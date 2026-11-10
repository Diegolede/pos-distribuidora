use pos_core::{Error, ImportError, ImportPreview, Money, Product};
use pos_db::Database;
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::str::FromStr;

use crate::import::ExcelImporter;

/// Clave de búsqueda normalizada para detectar duplicados
#[derive(Hash, Eq, PartialEq, Clone)]
struct ProductKey {
    name: String,
    category: Option<String>,
}

impl ProductKey {
    fn from_product(product: &Product) -> Self {
        Self {
            name: product.name.trim().to_lowercase(),
            category: product.category.as_ref().map(|c| c.trim().to_lowercase()),
        }
    }
}

pub struct OptimizedImporter {
    importer: ExcelImporter,
}

impl OptimizedImporter {
    pub fn new(importer: ExcelImporter) -> Self {
        Self { importer }
    }

    pub fn import_products_optimized<F>(
        &self,
        sheet_name: &str,
        header_row: usize,
        db: &Database,
        mut progress_callback: F,
    ) -> Result<ImportPreview, Error>
    where
        F: FnMut(usize, usize) + Send,
    {
        // Leer todas las filas del Excel
        let (_mapping, raw_rows) = self.importer.read_sheet(sheet_name, header_row)?;
        let total_rows = raw_rows.len();

        // Configurar SQLite para máximo rendimiento
        db.with_connection(|conn| {
            conn.execute_batch(
                "PRAGMA journal_mode=WAL;
                 PRAGMA synchronous=NORMAL;
                 PRAGMA cache_size=10000;
                 PRAGMA temp_store=MEMORY;"
            )
            .map_err(|e| Error::Database(e.to_string()))?;
            Ok(())
        })?;

        // Precargar productos existentes en un HashMap
        let existing_products = db.get_all_products(None)
            .map_err(|e| Error::Database(e.to_string()))?;

        let mut existing_by_sku: HashMap<String, Product> = HashMap::new();
        let mut existing_by_key: HashMap<ProductKey, Product> = HashMap::new();

        for product in existing_products {
            if let Some(ref sku) = product.sku {
                existing_by_sku.insert(sku.clone(), product.clone());
            }
            existing_by_key.insert(ProductKey::from_product(&product), product.clone());
        }

        tracing::info!(
            "Precargados {} productos existentes ({} con SKU)",
            existing_by_key.len(),
            existing_by_sku.len()
        );

        // Validar y procesar todas las filas
        let mut products_to_insert = Vec::new();
        let mut products_to_update = Vec::new();
        let mut errors = Vec::new();
        let mut empty_rows_skipped = 0;

        for (idx, row) in raw_rows.iter().enumerate() {
            if idx % 100 == 0 {
                progress_callback(idx, total_rows);
            }

            // Ignorar filas completamente vacías
            if ExcelImporter::is_empty_row(row) {
                empty_rows_skipped += 1;
                continue;
            }

            match self.importer.validate_row(row) {
                Ok(mut product) => {
                    // Calcular precios faltantes
                    self.importer.calculate_missing_prices(&mut product, db);

                    // Buscar producto existente por SKU o por nombre+categoría
                    let existing = if let Some(ref sku) = product.sku {
                        existing_by_sku.get(sku)
                    } else {
                        let key = ProductKey::from_product(&product);
                        existing_by_key.get(&key)
                    };

                    if let Some(existing_product) = existing {
                        product.id = existing_product.id;
                        product.created_at = existing_product.created_at;
                        products_to_update.push(product);
                    } else {
                        products_to_insert.push(product);
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

        progress_callback(total_rows, total_rows);

        let new_count = products_to_insert.len();
        let update_count = products_to_update.len();
        let valid_count = new_count + update_count;

        tracing::info!(
            "Procesadas {} filas: {} nuevos, {} actualizaciones, {} errores, {} vacías",
            total_rows,
            new_count,
            update_count,
            errors.len(),
            empty_rows_skipped
        );

        // Ejecutar todo en UNA transacción con prepared statements
        let start = std::time::Instant::now();

        db.transaction(|tx| {
            // Deshabilitar triggers de FTS5 temporalmente
            tx.execute("DELETE FROM products_fts WHERE 1=1", [])
                .map_err(|e| Error::Database(e.to_string()))?;

            // Prepared statement para INSERT
            let mut insert_stmt = tx
                .prepare(
                    "INSERT INTO products (name, description, price_professional, price_public, 
                     price_professional_calculated, price_public_calculated, category, sku, 
                     created_at, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, datetime('now'), datetime('now'))"
                )
                .map_err(|e| Error::Database(e.to_string()))?;

            // Insertar productos nuevos
            for product in &products_to_insert {
                insert_stmt
                    .execute((
                        &product.name,
                        &product.description,
                        product.price_professional.as_ref().map(|m| m.amount().to_string()),
                        product.price_public.as_ref().map(|m| m.amount().to_string()),
                        product.price_professional_calculated,
                        product.price_public_calculated,
                        &product.category,
                        &product.sku,
                    ))
                    .map_err(|e| Error::Database(e.to_string()))?;
            }

            drop(insert_stmt);

            // Prepared statement para UPDATE
            let mut update_stmt = tx
                .prepare(
                    "UPDATE products SET name = ?1, description = ?2, price_professional = ?3, 
                     price_public = ?4, price_professional_calculated = ?5, 
                     price_public_calculated = ?6, category = ?7, sku = ?8, 
                     updated_at = datetime('now')
                     WHERE id = ?9"
                )
                .map_err(|e| Error::Database(e.to_string()))?;

            // Actualizar productos existentes
            for product in &products_to_update {
                update_stmt
                    .execute((
                        &product.name,
                        &product.description,
                        product.price_professional.as_ref().map(|m| m.amount().to_string()),
                        product.price_public.as_ref().map(|m| m.amount().to_string()),
                        product.price_professional_calculated,
                        product.price_public_calculated,
                        &product.category,
                        &product.sku,
                        product.id,
                    ))
                    .map_err(|e| Error::Database(e.to_string()))?;
            }

            drop(update_stmt);

            // Reconstruir índice FTS5 completo UNA SOLA VEZ
            tx.execute(
                "INSERT INTO products_fts (rowid, name, category)
                 SELECT id, name, COALESCE(category, '') FROM products",
                [],
            )
            .map_err(|e| Error::Database(e.to_string()))?;

            Ok(())
        })?;

        let elapsed = start.elapsed();
        tracing::info!(
            "✓ Importación completada en {:.2}s ({:.0} filas/s)",
            elapsed.as_secs_f64(),
            total_rows as f64 / elapsed.as_secs_f64()
        );

        Ok(ImportPreview {
            total_rows: total_rows - empty_rows_skipped,
            valid_rows: valid_count,
            new_products: new_count,
            updates: update_count,
            errors,
            calculated_professional_prices: 0,
            calculated_public_prices: 0,
            products_without_prices: 0,
        })
    }
}
