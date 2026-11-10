use calamine::{open_workbook, Reader, Xlsx, DataType};
use chrono::Utc;
use pos_core::{Currency, Error, ImportError, ImportPreview, Money, Product};
use pos_db::Database;
use regex::Regex;
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::path::Path;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct ExcelImporter {
    pub file_path: String,
    pub sheet_names: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ColumnMapping {
    pub name: Option<usize>,
    pub price_professional: Option<usize>,
    pub price_public: Option<usize>,
    pub description: Option<usize>,
    pub category: Option<usize>,
    pub category_columns: Vec<usize>, // Categorías jerárquicas (Categoría 1, 2, 3...)
    pub sku: Option<usize>,
}

impl ColumnMapping {
    pub fn detect_from_headers(headers: &[String]) -> Self {
        let normalize = |s: &str| s.to_lowercase().trim().replace(" ", "");

        let mut mapping = ColumnMapping {
            name: None,
            price_professional: None,
            price_public: None,
            description: None,
            category: None,
            category_columns: Vec::new(),
            sku: None,
        };

        for (idx, header) in headers.iter().enumerate() {
            let normalized = normalize(header);
            
            if normalized.contains("nombre") {
                mapping.name = Some(idx);
            } else if normalized.contains("precioprofesional") || normalized.contains("profesional") {
                mapping.price_professional = Some(idx);
            } else if normalized.contains("preciopublico") || normalized.contains("publico") || normalized.contains("público") {
                mapping.price_public = Some(idx);
            } else if normalized.contains("descripcion") || normalized.contains("descripción") {
                mapping.description = Some(idx);
            } else if normalized.contains("categoria") || normalized.contains("categoría") {
                // Detectar tanto "Categoría" simple como "Categoría 1", "Categoría 2", etc.
                mapping.category_columns.push(idx);
                // Usar la primera columna de categoría como category principal si no está definida
                if mapping.category.is_none() {
                    mapping.category = Some(idx);
                }
            } else if normalized.contains("sku") || normalized.contains("codigo") || normalized.contains("código") {
                mapping.sku = Some(idx);
            }
        }

        mapping
    }
}

#[derive(Debug, Clone)]
pub struct RawProductRow {
    pub row_number: usize,
    pub name: Option<String>,
    pub price_professional: Option<String>,
    pub price_public: Option<String>,
    pub description: Option<String>,
    pub category: Option<String>,
    pub sku: Option<String>,
}

impl ExcelImporter {
    pub fn open(file_path: &str) -> Result<Self, Error> {
        let workbook: Xlsx<_> = open_workbook(file_path)
            .map_err(|e| Error::Import(format!("No se pudo abrir el archivo: {}", e)))?;

        let sheet_names = workbook.sheet_names().to_vec();

        if sheet_names.is_empty() {
            return Err(Error::Import(
                "El archivo no contiene hojas".to_string(),
            ));
        }

        Ok(Self {
            file_path: file_path.to_string(),
            sheet_names,
        })
    }

    pub fn read_sheet(&self, sheet_name: &str, header_row: usize) -> Result<(ColumnMapping, Vec<RawProductRow>), Error> {
        let mut workbook: Xlsx<_> = open_workbook(&self.file_path)
            .map_err(|e| Error::Import(format!("No se pudo abrir el archivo: {}", e)))?;

        let range = workbook
            .worksheet_range(sheet_name)
            .map_err(|e| Error::Import(format!("Error al leer hoja: {}", e)))?;

        let rows: Vec<Vec<_>> = range.rows()
            .map(|r| r.to_vec())
            .collect();

        if rows.len() <= header_row {
            return Err(Error::Import(
                "El archivo no tiene suficientes filas".to_string(),
            ));
        }

        let headers: Vec<String> = rows[header_row]
            .iter()
            .map(|cell| cell.to_string())
            .collect();

        let mapping = ColumnMapping::detect_from_headers(&headers);

        let mut raw_rows = Vec::new();

        for (idx, row) in rows.iter().enumerate().skip(header_row + 1) {
            let get_cell = |col: Option<usize>| -> Option<String> {
                col.and_then(|c| {
                    row.get(c).and_then(|cell| {
                        let s = cell.to_string().trim().to_string();
                        if s.is_empty() { None } else { Some(s) }
                    })
                })
            };

            // Combinar categorías jerárquicas (Categoría 1 > Categoría 2 > ...)
            let combined_category = if !mapping.category_columns.is_empty() {
                let cats: Vec<String> = mapping.category_columns
                    .iter()
                    .filter_map(|&col_idx| get_cell(Some(col_idx)))
                    .collect();
                if cats.is_empty() {
                    None
                } else {
                    Some(cats.join(" > "))
                }
            } else {
                get_cell(mapping.category)
            };

            raw_rows.push(RawProductRow {
                row_number: idx + 1,
                name: get_cell(mapping.name),
                price_professional: get_cell(mapping.price_professional),
                price_public: get_cell(mapping.price_public),
                description: get_cell(mapping.description),
                category: combined_category,
                sku: get_cell(mapping.sku),
            });
        }

        Ok((mapping, raw_rows))
    }

    pub fn preview_import(
        &self,
        sheet_name: &str,
        header_row: usize,
        db: &Database,
    ) -> Result<ImportPreview, Error> {
        let (_mapping, raw_rows) = self.read_sheet(sheet_name, header_row)?;

        let mut errors = Vec::new();
        let mut valid_count = 0;
        let mut new_count = 0;
        let mut update_count = 0;
        let mut empty_rows_skipped = 0;

        for row in &raw_rows {
            // Verificar si la fila está completamente vacía
            if Self::is_empty_row(row) {
                empty_rows_skipped += 1;
                continue;
            }

            match self.validate_row(row) {
                Ok(product) => {
                    valid_count += 1;
                    
                    // Intentar encontrar por SKU o por nombre+categoría
                    let existing = if let Some(ref sku) = product.sku {
                        db.find_by_sku(sku).ok().flatten()
                    } else {
                        Self::find_by_name_and_category(db, &product.name, product.category.as_deref()).ok().flatten()
                    };

                    if existing.is_some() {
                        update_count += 1;
                    } else {
                        new_count += 1;
                    }
                }
                Err(e) => {
                    errors.push(ImportError {
                        row: row.row_number,
                        column: None,
                        message: e.to_string(),
                    });
                }
            }
        }

        tracing::info!(
            "Importación: {} válidas, {} vacías omitidas, {} errores",
            valid_count,
            empty_rows_skipped,
            errors.len()
        );

        Ok(ImportPreview {
            total_rows: raw_rows.len() - empty_rows_skipped,
            valid_rows: valid_count,
            new_products: new_count,
            updates: update_count,
            errors,
            calculated_professional_prices: 0, // TODO: implementar contador
            calculated_public_prices: 0, // TODO: implementar contador  
            products_without_prices: 0, // TODO: implementar contador
        })
    }

    pub fn is_empty_row(row: &RawProductRow) -> bool {
        row.name.is_none()
            && row.price_professional.is_none()
            && row.price_public.is_none()
            && row.description.is_none()
            && row.category.is_none()
            && row.sku.is_none()
    }

    /// Buscar producto existente por nombre y categoría (normalizado, case-insensitive)
    fn find_by_name_and_category(
        db: &Database,
        name: &str,
        category: Option<&str>,
    ) -> Result<Option<Product>, Error> {
        let normalized_name = name.trim().to_lowercase();
        let normalized_category = category.map(|c| c.trim().to_lowercase());

        // Obtener todos los productos y buscar coincidencia
        // TODO: optimizar con índice cuando el dataset crezca mucho
        let all_products = db.get_all_products(None)
            .map_err(|e| Error::Database(e.to_string()))?;

        for product in all_products {
            let product_name = product.name.trim().to_lowercase();
            let product_category = product.category.as_ref().map(|c| c.trim().to_lowercase());

            if product_name == normalized_name && product_category == normalized_category {
                return Ok(Some(product));
            }
        }

        Ok(None)
    }

    pub fn import_products(
        &self,
        sheet_name: &str,
        header_row: usize,
        db: &Database,
    ) -> Result<ImportPreview, Error> {
        let (_mapping, raw_rows) = self.read_sheet(sheet_name, header_row)?;

        let mut errors = Vec::new();
        let mut valid_count = 0;
        let mut new_count = 0;
        let mut update_count = 0;
        let mut empty_rows_skipped = 0;

        for row in &raw_rows {
            // Ignorar filas completamente vacías
            if Self::is_empty_row(row) {
                empty_rows_skipped += 1;
                continue;
            }

            match self.validate_row(row) {
                Ok(mut product) => {
                    // Calcular precios faltantes antes de guardar
                    self.calculate_missing_prices(&mut product, db);
                    
                    valid_count += 1;

                    // Intentar encontrar producto existente por SKU o por nombre+categoría
                    let existing = if let Some(ref sku) = product.sku {
                        db.find_by_sku(sku)?
                    } else {
                        // Sin SKU: buscar por nombre y categoría normalizada
                        Self::find_by_name_and_category(db, &product.name, product.category.as_deref())?
                    };

                    let result = if let Some(existing_product) = existing {
                        product.id = existing_product.id;
                        product.created_at = existing_product.created_at;
                        
                        // Al actualizar, solo reemplazar precios calculados si ahora vienen del Excel
                        // (un precio del Excel siempre reemplaza uno calculado)
                        db.update_product(&product).map(|_| true)
                    } else {
                        db.insert_product(&product).map(|_| false)
                    };

                    match result {
                        Ok(is_update) => {
                            if is_update {
                                update_count += 1;
                            } else {
                                new_count += 1;
                            }
                        }
                        Err(e) => {
                            errors.push(ImportError {
                                row: row.row_number,
                                column: None,
                                message: format!("Error al guardar: {}", e),
                            });
                        }
                    }
                }
                Err(e) => {
                    errors.push(ImportError {
                        row: row.row_number,
                        column: None,
                        message: e.to_string(),
                    });
                }
            }
        }

        tracing::info!(
            "Importación completada: {} nuevos, {} actualizados, {} errores, {} filas vacías omitidas",
            new_count,
            update_count,
            errors.len(),
            empty_rows_skipped
        );

        Ok(ImportPreview {
            total_rows: raw_rows.len() - empty_rows_skipped,
            valid_rows: valid_count,
            new_products: new_count,
            updates: update_count,
            errors,
            calculated_professional_prices: 0, // TODO: implementar contador
            calculated_public_prices: 0, // TODO: implementar contador
            products_without_prices: 0, // TODO: implementar contador
        })
    }

    /// Obtener markup percentage de la configuración (por defecto 30%)
    fn get_markup_percentage(&self, db: &Database) -> Decimal {
        db.with_connection(|conn| {
            let value = conn.query_row(
                "SELECT value FROM config WHERE key = 'price_markup_percentage'",
                [],
                |row| row.get::<_, String>(0)
            )
            .ok()
            .and_then(|v| Decimal::from_str(&v).ok())
            .unwrap_or(Decimal::from(30)); // 30% por defecto
            
            Ok(value)
        })
        .unwrap_or(Decimal::from(30))
    }

    pub fn validate_row(&self, row: &RawProductRow) -> Result<Product, Error> {
        let name = row.name.as_ref().ok_or_else(|| {
            Error::Validation(format!("Fila {}: falta el nombre", row.row_number))
        })?.clone();

        if name.is_empty() {
            return Err(Error::Validation(format!(
                "Fila {}: el nombre está vacío",
                row.row_number
            )));
        }

        // Parsear precios del Excel (pueden ser None)
        let price_professional_from_excel = row
            .price_professional
            .as_ref()
            .and_then(|p| parse_price(p).ok())
            .map(|amount| Money::new(amount, Currency::ARS));

        let price_public_from_excel = row
            .price_public
            .as_ref()
            .and_then(|p| parse_price(p).ok())
            .map(|amount| Money::new(amount, Currency::ARS));

        // NUEVA REGLA: Productos sin ningún precio son válidos (quedan como "no vendibles")
        // No lanzamos error, simplemente devolvemos el producto con ambos precios None

        Ok(Product {
            id: 0,
            name,
            description: row.description.clone().filter(|s| !s.is_empty()),
            price_professional: price_professional_from_excel,
            price_public: price_public_from_excel,
            price_professional_calculated: false,
            price_public_calculated: false,
            category: row.category.clone().filter(|s| !s.is_empty()),
            sku: row.sku.clone().filter(|s| !s.is_empty()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    /// Calcular precios faltantes usando el markup configurado
    pub fn calculate_missing_prices(&self, product: &mut Product, db: &Database) {
        let markup_percent = self.get_markup_percentage(db);
        let markup_multiplier = Decimal::ONE + (markup_percent / Decimal::from(100));

        match (&product.price_professional, &product.price_public) {
            // Caso 1: Falta precio público → calcularlo como profesional × (1 + markup%)
            (Some(prof), None) => {
                let calculated_public = Money::new(
                    prof.amount() * markup_multiplier,
                    prof.currency()
                );
                product.price_public = Some(calculated_public);
                product.price_public_calculated = true;
                
                tracing::debug!(
                    "Precio público calculado: {} → {} (markup {}%)",
                    prof.amount(),
                    calculated_public.amount(),
                    markup_percent
                );
            }
            
            // Caso 2: Falta precio profesional → calcularlo como público ÷ (1 + markup%)
            (None, Some(pub_price)) => {
                let calculated_prof = Money::new(
                    pub_price.amount() / markup_multiplier,
                    pub_price.currency()
                );
                product.price_professional = Some(calculated_prof);
                product.price_professional_calculated = true;
                
                tracing::debug!(
                    "Precio profesional calculado: {} → {} (markup {}%)",
                    pub_price.amount(),
                    calculated_prof.amount(),
                    markup_percent
                );
            }
            
            // Caso 3: Ambos precios presentes o ambos ausentes → no calcular nada
            _ => {}
        }
    }
}

fn parse_price(price_str: &str) -> Result<Decimal, Error> {
    let re = Regex::new(r"[^\d,.-]").unwrap();
    let cleaned = re.replace_all(price_str.trim(), "").to_string();
    
    let has_comma = cleaned.contains(',');
    let has_dot = cleaned.contains('.');
    
    let normalized = if has_comma && has_dot {
        let comma_pos = cleaned.rfind(',').unwrap();
        let dot_pos = cleaned.rfind('.').unwrap();
        
        if comma_pos > dot_pos {
            cleaned.replace(".", "").replace(",", ".")
        } else {
            cleaned.replace(",", "")
        }
    } else if has_comma {
        cleaned.replace(",", ".")
    } else {
        cleaned
    };

    Decimal::from_str(&normalized)
        .map_err(|_| Error::Validation(format!("Precio inválido: '{}'", price_str)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_price() {
        assert_eq!(parse_price("1234").unwrap().to_string(), "1234");
        assert_eq!(parse_price("$ 1.234,56").unwrap().to_string(), "1234.56");
        assert_eq!(parse_price("1,234.56").unwrap().to_string(), "1234.56"); // Formato USA
        assert_eq!(parse_price("1234,56").unwrap().to_string(), "1234.56");   // Formato ARG
    }

    #[test]
    fn test_column_detection() {
        let headers = vec![
            "Nombre".to_string(),
            "Precio Profesional".to_string(),
            "Precio Público".to_string(),
            "Categoría".to_string(),
        ];

        let mapping = ColumnMapping::detect_from_headers(&headers);
        assert_eq!(mapping.name, Some(0));
        assert_eq!(mapping.price_professional, Some(1));
        assert_eq!(mapping.price_public, Some(2));
        assert_eq!(mapping.category, Some(3));
    }
}
