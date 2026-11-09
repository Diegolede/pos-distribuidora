use chrono::{DateTime, Datelike, Utc};
use chrono_tz::Tz;
use pos_core::{CartItem, Currency, Error, Money, MonthlySummary, PaymentMethod, Sale, SaleItem};
use pos_db::Database;
use rusqlite::params;

pub struct SalesManager {
    db: Database,
    timezone: Tz,
}

impl SalesManager {
    pub fn new(db: Database, timezone: Tz) -> Self {
        Self { db, timezone }
    }

    pub fn create_sale(
        &self,
        items: Vec<CartItem>,
        payment_method: PaymentMethod,
        cash_received: Option<Money>,
    ) -> Result<Sale, Error> {
        if items.is_empty() {
            return Err(Error::Sale("El carrito está vacío".to_string()));
        }

        if payment_method == PaymentMethod::Cash && cash_received.is_none() {
            return Err(Error::Sale(
                "Debe especificar el efectivo recibido".to_string(),
            ));
        }

        let total: Money = items
            .iter()
            .map(|item| item.total())
            .fold(Money::zero(Currency::ARS), |acc, m| acc + m);

        let cash_change = if let Some(received) = cash_received {
            if received < total {
                return Err(Error::Sale(
                    "El efectivo recibido es menor al total".to_string(),
                ));
            }
            Some(received - total)
        } else {
            None
        };

        let timestamp = Utc::now();
        let sale_number = self.generate_sale_number(&timestamp)?;

        let sale = self.db.transaction(|tx| {
            tx.execute(
                "INSERT INTO sales (sale_number, timestamp, total_cents, currency, 
                 payment_method, cash_received_cents, cash_change_cents, is_voided)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 0)",
                params![
                    sale_number,
                    timestamp.to_rfc3339(),
                    total.to_cents(),
                    "ARS",
                    payment_method.as_str(),
                    cash_received.map(|m| m.to_cents()),
                    cash_change.map(|m| m.to_cents()),
                ],
            )
            .map_err(|e| Error::Database(e.to_string()))?;

            let sale_id = tx.last_insert_rowid();

            for item in &items {
                tx.execute(
                    "INSERT INTO sale_items (sale_id, product_id, product_name, quantity,
                     unit_price_cents, price_type, subtotal_cents, currency)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                    params![
                        sale_id,
                        item.product_id,
                        item.product_name,
                        item.quantity,
                        item.unit_price.to_cents(),
                        item.price_type.as_str(),
                        item.total().to_cents(),
                        "ARS",
                    ],
                )
                .map_err(|e| Error::Database(e.to_string()))?;
            }

            Ok(Sale {
                id: sale_id,
                sale_number: sale_number.clone(),
                timestamp,
                items: items
                    .into_iter()
                    .map(|cart_item| {
                        let subtotal = cart_item.total();
                        SaleItem {
                            product_id: cart_item.product_id,
                            product_name: cart_item.product_name,
                            quantity: cart_item.quantity,
                            unit_price: cart_item.unit_price,
                            price_type: cart_item.price_type,
                            subtotal,
                        }
                    })
                    .collect(),
                total,
                payment_method,
                cash_received,
                cash_change,
                is_voided: false,
                void_reason: None,
                void_timestamp: None,
            })
        })?;

        Ok(sale)
    }

    pub fn void_sale(&self, sale_id: i64, reason: String) -> Result<(), Error> {
        let rows = self.db.with_connection(|conn| {
            conn.execute(
                "UPDATE sales SET is_voided = 1, void_reason = ?1, void_timestamp = ?2
                 WHERE id = ?3 AND is_voided = 0",
                params![reason, Utc::now().to_rfc3339(), sale_id],
            )
            .map_err(|e| Error::Database(e.to_string()))
        })?;

        if rows == 0 {
            return Err(Error::Sale(
                "La venta no existe o ya está anulada".to_string(),
            ));
        }

        Ok(())
    }

    pub fn get_sale(&self, sale_id: i64) -> Result<Sale, Error> {
        self.db.with_connection(|conn| {
            let mut stmt = conn
                .prepare(
                    "SELECT id, sale_number, timestamp, total_cents, currency, payment_method,
                            cash_received_cents, cash_change_cents, is_voided, void_reason, void_timestamp
                     FROM sales WHERE id = ?1",
                )
                .map_err(|e| Error::Database(e.to_string()))?;

            let sale = stmt
                .query_row(params![sale_id], |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, i64>(3)?,
                        row.get::<_, String>(4)?,
                        row.get::<_, String>(5)?,
                        row.get::<_, Option<i64>>(6)?,
                        row.get::<_, Option<i64>>(7)?,
                        row.get::<_, bool>(8)?,
                        row.get::<_, Option<String>>(9)?,
                        row.get::<_, Option<String>>(10)?,
                    ))
                })
                .map_err(|e| Error::Database(e.to_string()))?;

            let mut items_stmt = conn
                .prepare(
                    "SELECT product_id, product_name, quantity, unit_price_cents, price_type, subtotal_cents
                     FROM sale_items WHERE sale_id = ?1",
                )
                .map_err(|e| Error::Database(e.to_string()))?;

            let items = items_stmt
                .query_map(params![sale_id], |row| {
                    Ok(SaleItem {
                        product_id: row.get(0)?,
                        product_name: row.get(1)?,
                        quantity: row.get(2)?,
                        unit_price: Money::from_cents(row.get(3)?, Currency::ARS),
                        price_type: match row.get::<_, String>(4)?.as_str() {
                            "Profesional" => pos_core::PriceType::Professional,
                            "Público" => pos_core::PriceType::Public,
                            _ => pos_core::PriceType::Inherit,
                        },
                        subtotal: Money::from_cents(row.get(5)?, Currency::ARS),
                    })
                })
                .map_err(|e| Error::Database(e.to_string()))?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| Error::Database(e.to_string()))?;

            Ok(Sale {
                id: sale.0,
                sale_number: sale.1,
                timestamp: DateTime::parse_from_rfc3339(&sale.2)
                    .map_err(|e| Error::Database(e.to_string()))?
                    .with_timezone(&Utc),
                items,
                total: Money::from_cents(sale.3, Currency::ARS),
                payment_method: PaymentMethod::from_str(&sale.5).unwrap_or(PaymentMethod::Cash),
                cash_received: sale.6.map(|cents| Money::from_cents(cents, Currency::ARS)),
                cash_change: sale.7.map(|cents| Money::from_cents(cents, Currency::ARS)),
                is_voided: sale.8,
                void_reason: sale.9,
                void_timestamp: sale
                    .10
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Utc)),
            })
        })
    }

    pub fn get_monthly_summary(&self, year: i32, month: u32) -> Result<MonthlySummary, Error> {
        self.db.with_connection(|conn| {
            let start = format!("{:04}-{:02}-01T00:00:00Z", year, month);
            let end = if month == 12 {
                format!("{:04}-01-01T00:00:00Z", year + 1)
            } else {
                format!("{:04}-{:02}-01T00:00:00Z", year, month + 1)
            };

            let mut stmt = conn
                .prepare(
                    "SELECT 
                        COUNT(*) as sales_count,
                        COALESCE(SUM(CASE WHEN is_voided = 0 THEN total_cents ELSE 0 END), 0) as total_cents,
                        COALESCE(SUM(CASE WHEN is_voided = 1 THEN total_cents ELSE 0 END), 0) as voided_cents,
                        COALESCE(SUM(CASE WHEN is_voided = 1 THEN 1 ELSE 0 END), 0) as voided_count
                     FROM sales
                     WHERE timestamp >= ?1 AND timestamp < ?2",
                )
                .map_err(|e| Error::Database(e.to_string()))?;

            let summary = stmt
                .query_row(params![start, end], |row| {
                    Ok((
                        row.get::<_, u32>(0)?,
                        row.get::<_, i64>(1)?,
                        row.get::<_, i64>(2)?,
                        row.get::<_, u32>(3)?,
                    ))
                })
                .map_err(|e| Error::Database(e.to_string()))?;

            let mut units_stmt = conn
                .prepare(
                    "SELECT COALESCE(SUM(si.quantity), 0)
                     FROM sale_items si
                     JOIN sales s ON si.sale_id = s.id
                     WHERE s.timestamp >= ?1 AND s.timestamp < ?2 AND s.is_voided = 0",
                )
                .map_err(|e| Error::Database(e.to_string()))?;

            let units_sold: u32 = units_stmt
                .query_row(params![start, end], |row| row.get(0))
                .map_err(|e| Error::Database(e.to_string()))?;

            let mut payment_stmt = conn
                .prepare(
                    "SELECT payment_method, 
                            COALESCE(SUM(total_cents), 0) as total,
                            COUNT(*) as count
                     FROM sales
                     WHERE timestamp >= ?1 AND timestamp < ?2 AND is_voided = 0
                     GROUP BY payment_method",
                )
                .map_err(|e| Error::Database(e.to_string()))?;

            let by_payment: Vec<(PaymentMethod, Money, u32)> = payment_stmt
                .query_map(params![start, end], |row| {
                    let method_str: String = row.get(0)?;
                    let method = PaymentMethod::from_str(&method_str).unwrap_or(PaymentMethod::Cash);
                    let total = Money::from_cents(row.get(1)?, Currency::ARS);
                    let count: u32 = row.get(2)?;
                    Ok((method, total, count))
                })
                .map_err(|e| Error::Database(e.to_string()))?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| Error::Database(e.to_string()))?;

            let total_sales = Money::from_cents(summary.1, Currency::ARS);
            let voided_total = Money::from_cents(summary.2, Currency::ARS);
            let net_total = total_sales;

            Ok(MonthlySummary {
                year,
                month,
                total_sales,
                sales_count: summary.0 - summary.3,
                units_sold,
                voided_sales: summary.3,
                voided_total,
                net_total,
                by_payment_method: by_payment,
            })
        })
    }

    pub fn get_sales_for_month(&self, year: i32, month: u32) -> Result<Vec<Sale>, Error> {
        let start = format!("{:04}-{:02}-01T00:00:00Z", year, month);
        let end = if month == 12 {
            format!("{:04}-01-01T00:00:00Z", year + 1)
        } else {
            format!("{:04}-{:02}-01T00:00:00Z", year, month + 1)
        };

        self.db.with_connection(|conn| {
            let mut stmt = conn
                .prepare(
                    "SELECT id FROM sales 
                     WHERE timestamp >= ?1 AND timestamp < ?2
                     ORDER BY timestamp DESC",
                )
                .map_err(|e| Error::Database(e.to_string()))?;

            let ids: Vec<i64> = stmt
                .query_map(params![start, end], |row| row.get(0))
                .map_err(|e| Error::Database(e.to_string()))?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| Error::Database(e.to_string()))?;

            ids.into_iter()
                .map(|id| self.get_sale(id))
                .collect::<Result<Vec<_>, _>>()
        })
    }

    fn generate_sale_number(&self, timestamp: &DateTime<Utc>) -> Result<String, Error> {
        let local = timestamp.with_timezone(&self.timezone);
        let date_prefix = format!("{:04}{:02}{:03}", local.year(), local.month(), local.ordinal());

        let count: i64 = self.db.with_connection(|conn| {
            conn.query_row(
                "SELECT COUNT(*) FROM sales WHERE sale_number LIKE ?1 || '%'",
                params![date_prefix],
                |row| row.get(0),
            )
            .map_err(|e| Error::Database(e.to_string()))
        })?;

        Ok(format!("{}-{:04}", date_prefix, count + 1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono_tz::America::Argentina::Buenos_Aires;
    use pos_core::PriceType;
    use pos_db::{init_database, Database};
    use std::path::PathBuf;

    #[test]
    fn test_create_sale() {
        let db = Database::open_in_memory().unwrap();
        pos_db::migrations::run_migrations(&db).unwrap();

        // Crear un producto de prueba
        use chrono::Utc;
        use pos_core::Product;
        let product = Product {
            id: 0,
            name: "Producto Test".to_string(),
            description: None,
            price_professional: Some(Money::from_pesos(100)),
            price_public: Some(Money::from_pesos(150)),
            price_professional_calculated: false,
            price_public_calculated: false,
            category: None,
            sku: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let product_id = db.insert_product(&product).unwrap();

        let manager = SalesManager::new(db, Buenos_Aires);

        let items = vec![CartItem {
            product_id,
            product_name: "Producto Test".to_string(),
            quantity: 2,
            price_type: PriceType::Professional,
            unit_price: Money::from_pesos(100),
        }];

        let sale = manager
            .create_sale(
                items,
                PaymentMethod::Cash,
                Some(Money::from_pesos(300)),
            )
            .unwrap();

        assert_eq!(sale.total.to_cents(), 20000);
        assert_eq!(sale.items.len(), 1);
        assert!(sale.cash_change.is_some());
        assert_eq!(sale.cash_change.unwrap().to_cents(), 10000);
    }

    #[test]
    fn test_void_sale() {
        let db = Database::open_in_memory().unwrap();
        pos_db::migrations::run_migrations(&db).unwrap();

        // Crear un producto de prueba
        use chrono::Utc;
        use pos_core::Product;
        let product = Product {
            id: 0,
            name: "Producto Test".to_string(),
            description: None,
            price_professional: Some(Money::from_pesos(100)),
            price_public: Some(Money::from_pesos(150)),
            price_professional_calculated: false,
            price_public_calculated: false,
            category: None,
            sku: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let product_id = db.insert_product(&product).unwrap();

        let manager = SalesManager::new(db, Buenos_Aires);

        let items = vec![CartItem {
            product_id,
            product_name: "Producto Test".to_string(),
            quantity: 1,
            price_type: PriceType::Professional,
            unit_price: Money::from_pesos(100),
        }];

        let sale = manager
            .create_sale(items, PaymentMethod::Cash, Some(Money::from_pesos(100)))
            .unwrap();

        manager
            .void_sale(sale.id, "Error en el precio".to_string())
            .unwrap();

        let retrieved = manager.get_sale(sale.id).unwrap();
        assert!(retrieved.is_voided);
        assert_eq!(retrieved.void_reason, Some("Error en el precio".to_string()));
    }

    #[test]
    fn test_monthly_summary() {
        let db = Database::open_in_memory().unwrap();
        pos_db::migrations::run_migrations(&db).unwrap();

        // Crear un producto de prueba
        use chrono::Utc;
        use pos_core::Product;
        let product = Product {
            id: 0,
            name: "Producto Test".to_string(),
            description: None,
            price_professional: Some(Money::from_pesos(100)),
            price_public: Some(Money::from_pesos(150)),
            price_professional_calculated: false,
            price_public_calculated: false,
            category: None,
            sku: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let product_id = db.insert_product(&product).unwrap();

        let manager = SalesManager::new(db, Buenos_Aires);

        for _ in 0..5 {
            let items = vec![CartItem {
                product_id,
                product_name: "Producto Test".to_string(),
                quantity: 1,
                price_type: PriceType::Professional,
                unit_price: Money::from_pesos(100),
            }];

            manager
                .create_sale(items, PaymentMethod::Cash, Some(Money::from_pesos(100)))
                .unwrap();
        }

        let now = Utc::now();
        let summary = manager
            .get_monthly_summary(now.year(), now.month())
            .unwrap();

        assert_eq!(summary.sales_count, 5);
        assert_eq!(summary.total_sales.to_cents(), 50000);
    }
}
