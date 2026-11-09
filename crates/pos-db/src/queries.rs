use crate::Database;
use chrono::{DateTime, Utc};
use pos_core::{Error, Money, Product, ProductId, Currency};
use rusqlite::params;
use rust_decimal::Decimal;

impl Database {
    pub fn insert_product(&self, product: &Product) -> Result<ProductId, Error> {
        self.with_connection(|conn| {
            let name_normalized = crate::migrations::normalize_text(&product.name);
            
            conn.execute(
                "INSERT INTO products (name, name_normalized, description, price_professional_cents, 
                 price_public_cents, price_professional_calculated, price_public_calculated,
                 currency, category, sku, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                params![
                    product.name,
                    name_normalized,
                    product.description,
                    product.price_professional.map(|m| m.to_cents()),
                    product.price_public.map(|m| m.to_cents()),
                    product.price_professional_calculated as i32,
                    product.price_public_calculated as i32,
                    "ARS",
                    product.category,
                    product.sku,
                    product.created_at.to_rfc3339(),
                    product.updated_at.to_rfc3339(),
                ],
            )
            .map_err(|e| Error::Database(e.to_string()))?;

            let id = conn.last_insert_rowid();
            Ok(id)
        })
    }

    pub fn update_product(&self, product: &Product) -> Result<(), Error> {
        self.with_connection(|conn| {
            let name_normalized = crate::migrations::normalize_text(&product.name);
            
            let rows = conn.execute(
                "UPDATE products 
                 SET name = ?1, name_normalized = ?2, description = ?3, 
                     price_professional_cents = ?4, price_public_cents = ?5,
                     price_professional_calculated = ?6, price_public_calculated = ?7,
                     category = ?8, sku = ?9, updated_at = ?10
                 WHERE id = ?11",
                params![
                    product.name,
                    name_normalized,
                    product.description,
                    product.price_professional.map(|m| m.to_cents()),
                    product.price_public.map(|m| m.to_cents()),
                    product.price_professional_calculated as i32,
                    product.price_public_calculated as i32,
                    product.category,
                    product.sku,
                    Utc::now().to_rfc3339(),
                    product.id,
                ],
            )
            .map_err(|e| Error::Database(e.to_string()))?;

            if rows == 0 {
                return Err(Error::ProductNotFound(product.id.to_string()));
            }

            Ok(())
        })
    }

    pub fn get_product(&self, id: ProductId) -> Result<Product, Error> {
        self.with_connection(|conn| {
            conn.query_row(
                "SELECT id, name, description, price_professional_cents, price_public_cents,
                        price_professional_calculated, price_public_calculated,
                        currency, category, sku, created_at, updated_at
                 FROM products WHERE id = ?1",
                params![id],
                |row| {
                    let price_prof_cents: Option<i64> = row.get(3)?;
                    let price_pub_cents: Option<i64> = row.get(4)?;
                    let price_prof_calculated: i32 = row.get(5)?;
                    let price_pub_calculated: i32 = row.get(6)?;
                    let currency_str: String = row.get(7)?;
                    let currency = match currency_str.as_str() {
                        "ARS" => Currency::ARS,
                        "USD" => Currency::USD,
                        _ => Currency::ARS,
                    };

                    Ok(Product {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        description: row.get(2)?,
                        price_professional: price_prof_cents
                            .map(|cents| Money::from_cents(cents, currency)),
                        price_public: price_pub_cents
                            .map(|cents| Money::from_cents(cents, currency)),
                        price_professional_calculated: price_prof_calculated != 0,
                        price_public_calculated: price_pub_calculated != 0,
                        category: row.get(8)?,
                        sku: row.get(9)?,
                        created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(10)?)
                            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?
                            .with_timezone(&Utc),
                        updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(11)?)
                            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?
                            .with_timezone(&Utc),
                    })
                },
            )
            .map_err(|e| Error::Database(e.to_string()))
        })
    }

    pub fn search_products(&self, query: &str, category: Option<&str>, limit: usize) -> Result<Vec<Product>, Error> {
        self.with_connection(|conn| {
            let sql = if let Some(cat) = category {
                "SELECT p.id, p.name, p.description, p.price_professional_cents, p.price_public_cents,
                        p.price_professional_calculated, p.price_public_calculated,
                        p.currency, p.category, p.sku, p.created_at, p.updated_at
                 FROM products p
                 WHERE p.id IN (SELECT rowid FROM products_fts WHERE products_fts MATCH ?1)
                   AND p.category = ?2
                 LIMIT ?3"
            } else {
                "SELECT p.id, p.name, p.description, p.price_professional_cents, p.price_public_cents,
                        p.price_professional_calculated, p.price_public_calculated,
                        p.currency, p.category, p.sku, p.created_at, p.updated_at
                 FROM products p
                 WHERE p.id IN (SELECT rowid FROM products_fts WHERE products_fts MATCH ?1)
                 LIMIT ?2"
            };

            let mut stmt = conn.prepare(sql)
                .map_err(|e| Error::Database(e.to_string()))?;

            let products = if let Some(cat) = category {
                stmt.query_map(params![query, cat, limit as i64], Self::row_to_product)
                    .map_err(|e| Error::Database(e.to_string()))?
            } else {
                stmt.query_map(params![query, limit as i64], Self::row_to_product)
                    .map_err(|e| Error::Database(e.to_string()))?
            };

            products
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| Error::Database(e.to_string()))
        })
    }

    pub fn get_all_products(&self, limit: Option<usize>) -> Result<Vec<Product>, Error> {
        self.with_connection(|conn| {
            let sql = if let Some(lim) = limit {
                format!("SELECT id, name, description, price_professional_cents, price_public_cents,
                                price_professional_calculated, price_public_calculated,
                                currency, category, sku, created_at, updated_at
                         FROM products ORDER BY name LIMIT {}", lim)
            } else {
                "SELECT id, name, description, price_professional_cents, price_public_cents,
                        price_professional_calculated, price_public_calculated,
                        currency, category, sku, created_at, updated_at
                 FROM products ORDER BY name".to_string()
            };

            let mut stmt = conn.prepare(&sql)
                .map_err(|e| Error::Database(e.to_string()))?;

            let products = stmt.query_map([], Self::row_to_product)
                .map_err(|e| Error::Database(e.to_string()))?;

            products
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| Error::Database(e.to_string()))
        })
    }

    pub fn get_categories(&self) -> Result<Vec<String>, Error> {
        self.with_connection(|conn| {
            let mut stmt = conn
                .prepare("SELECT DISTINCT category FROM products WHERE category IS NOT NULL ORDER BY category")
                .map_err(|e| Error::Database(e.to_string()))?;

            let categories = stmt.query_map([], |row| row.get(0))
                .map_err(|e| Error::Database(e.to_string()))?;

            categories
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| Error::Database(e.to_string()))
        })
    }

    pub fn count_products(&self) -> Result<usize, Error> {
        self.with_connection(|conn| {
            conn.query_row("SELECT COUNT(*) FROM products", [], |row| row.get(0))
                .map_err(|e| Error::Database(e.to_string()))
        })
    }

    pub fn find_by_sku(&self, sku: &str) -> Result<Option<Product>, Error> {
        self.with_connection(|conn| {
            match conn.query_row(
                "SELECT id, name, description, price_professional_cents, price_public_cents,
                        price_professional_calculated, price_public_calculated,
                        currency, category, sku, created_at, updated_at
                 FROM products WHERE sku = ?1",
                params![sku],
                Self::row_to_product,
            ) {
                Ok(product) => Ok(Some(product)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(Error::Database(e.to_string())),
            }
        })
    }

    fn row_to_product(row: &rusqlite::Row) -> rusqlite::Result<Product> {
        let price_prof_cents: Option<i64> = row.get(3)?;
        let price_pub_cents: Option<i64> = row.get(4)?;
        let price_prof_calculated: i32 = row.get(5)?;
        let price_pub_calculated: i32 = row.get(6)?;
        let currency_str: String = row.get(7)?;
        let currency = match currency_str.as_str() {
            "ARS" => Currency::ARS,
            "USD" => Currency::USD,
            _ => Currency::ARS,
        };

        Ok(Product {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            price_professional: price_prof_cents.map(|cents| Money::from_cents(cents, currency)),
            price_public: price_pub_cents.map(|cents| Money::from_cents(cents, currency)),
            price_professional_calculated: price_prof_calculated != 0,
            price_public_calculated: price_pub_calculated != 0,
            category: row.get(8)?,
            sku: row.get(9)?,
            created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(10)?)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?
                .with_timezone(&Utc),
            updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(11)?)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?
                .with_timezone(&Utc),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::migrations::run_migrations;
    use rust_decimal_macros::dec;

    #[test]
    fn test_product_crud() {
        let db = Database::open_in_memory().unwrap();
        run_migrations(&db).unwrap();

        let mut product = Product {
            id: 0,
            name: "Producto Test".to_string(),
            description: Some("Descripción test".to_string()),
            price_professional: Some(Money::new(dec!(100.50), Currency::ARS)),
            price_public: Some(Money::new(dec!(150.75), Currency::ARS)),
            price_professional_calculated: false,
            price_public_calculated: false,
            category: Some("Test".to_string()),
            sku: Some("SKU001".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let id = db.insert_product(&product).unwrap();
        product.id = id;

        let retrieved = db.get_product(id).unwrap();
        assert_eq!(retrieved.name, product.name);
        assert_eq!(retrieved.price_professional, product.price_professional);

        let count = db.count_products().unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_search_products() {
        let db = Database::open_in_memory().unwrap();
        run_migrations(&db).unwrap();

        let product1 = Product {
            id: 0,
            name: "Shampoo Anticaspa".to_string(),
            description: Some("Para cabello graso".to_string()),
            price_professional: Some(Money::from_pesos(100)),
            price_public: Some(Money::from_pesos(150)),
            price_professional_calculated: false,
            price_public_calculated: false,
            category: Some("Higiene".to_string()),
            sku: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        db.insert_product(&product1).unwrap();

        let results = db.search_products("shampoo", None, 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Shampoo Anticaspa");
    }
}
