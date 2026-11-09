use crate::Database;
use pos_core::Error;

const MIGRATIONS: &[&str] = &[
    // Migración 1: Tablas base
    r#"
    CREATE TABLE IF NOT EXISTS schema_version (
        version INTEGER PRIMARY KEY,
        applied_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
    );

    CREATE TABLE IF NOT EXISTS products (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL,
        name_normalized TEXT NOT NULL,
        description TEXT,
        price_professional_cents INTEGER,
        price_public_cents INTEGER,
        currency TEXT NOT NULL DEFAULT 'ARS',
        category TEXT,
        sku TEXT,
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL
    );

    CREATE UNIQUE INDEX IF NOT EXISTS idx_products_sku_unique ON products(sku) WHERE sku IS NOT NULL;
    CREATE INDEX IF NOT EXISTS idx_products_name ON products(name_normalized);
    CREATE INDEX IF NOT EXISTS idx_products_category ON products(category);

    CREATE VIRTUAL TABLE IF NOT EXISTS products_fts USING fts5(
        name,
        description,
        category,
        content='products',
        content_rowid='id',
        tokenize='unicode61 remove_diacritics 1'
    );

    CREATE TRIGGER IF NOT EXISTS products_fts_insert AFTER INSERT ON products BEGIN
        INSERT INTO products_fts(rowid, name, description, category)
        VALUES (new.id, new.name, new.description, new.category);
    END;

    CREATE TRIGGER IF NOT EXISTS products_fts_update AFTER UPDATE ON products BEGIN
        UPDATE products_fts SET
            name = new.name,
            description = new.description,
            category = new.category
        WHERE rowid = old.id;
    END;

    CREATE TRIGGER IF NOT EXISTS products_fts_delete AFTER DELETE ON products BEGIN
        DELETE FROM products_fts WHERE rowid = old.id;
    END;
    "#,
    // Migración 2: Ventas
    r#"
    CREATE TABLE IF NOT EXISTS sales (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        sale_number TEXT NOT NULL UNIQUE,
        timestamp TEXT NOT NULL,
        total_cents INTEGER NOT NULL,
        currency TEXT NOT NULL,
        payment_method TEXT NOT NULL,
        cash_received_cents INTEGER,
        cash_change_cents INTEGER,
        is_voided INTEGER NOT NULL DEFAULT 0,
        void_reason TEXT,
        void_timestamp TEXT
    );

    CREATE INDEX IF NOT EXISTS idx_sales_timestamp ON sales(timestamp);
    CREATE INDEX IF NOT EXISTS idx_sales_number ON sales(sale_number);
    CREATE INDEX IF NOT EXISTS idx_sales_voided ON sales(is_voided);

    CREATE TABLE IF NOT EXISTS sale_items (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        sale_id INTEGER NOT NULL,
        product_id INTEGER NOT NULL,
        product_name TEXT NOT NULL,
        quantity INTEGER NOT NULL,
        unit_price_cents INTEGER NOT NULL,
        price_type TEXT NOT NULL,
        subtotal_cents INTEGER NOT NULL,
        currency TEXT NOT NULL,
        FOREIGN KEY (sale_id) REFERENCES sales(id) ON DELETE CASCADE,
        FOREIGN KEY (product_id) REFERENCES products(id)
    );

    CREATE INDEX IF NOT EXISTS idx_sale_items_sale ON sale_items(sale_id);
    CREATE INDEX IF NOT EXISTS idx_sale_items_product ON sale_items(product_id);
    "#,
    // Migración 3: Configuración
    r#"
    CREATE TABLE IF NOT EXISTS config (
        key TEXT PRIMARY KEY,
        value TEXT NOT NULL,
        updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
    );
    "#,
    // Migración 4: Flags de precios calculados y markup por defecto
    r#"
    ALTER TABLE products ADD COLUMN price_professional_calculated INTEGER NOT NULL DEFAULT 0;
    ALTER TABLE products ADD COLUMN price_public_calculated INTEGER NOT NULL DEFAULT 0;
    
    INSERT OR IGNORE INTO config (key, value, updated_at) 
    VALUES ('price_markup_percentage', '30', CURRENT_TIMESTAMP);
    "#,
];

pub fn run_migrations(db: &Database) -> Result<(), Error> {
    let current_version = db.with_connection(|conn| {
        // First check if the schema_version table exists
        let table_exists: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='schema_version'",
                [],
                |row| row.get::<_, i32>(0),
            )
            .map(|count| count > 0)
            .unwrap_or(false);
        
        if !table_exists {
            return Ok(0);
        }
        
        // If it exists, get the max version
        conn.query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_version",
            [],
            |row| row.get::<_, i32>(0),
        )
        .or(Ok(0))
        .map_err(|e: rusqlite::Error| Error::Database(e.to_string()))
    })?;

    tracing::info!("Versión actual del esquema: {}", current_version);

    for (idx, migration) in MIGRATIONS.iter().enumerate() {
        let version = (idx + 1) as i32;
        if version > current_version {
            tracing::info!("Aplicando migración {}", version);
            db.transaction(|tx| {
                tx.execute_batch(migration)
                    .map_err(|e| Error::Database(format!("Migración {} falló: {}", version, e)))?;

                tx.execute(
                    "INSERT INTO schema_version (version) VALUES (?1)",
                    [version],
                )
                .map_err(|e| Error::Database(e.to_string()))?;

                Ok(())
            })?;
            tracing::info!("Migración {} aplicada exitosamente", version);
        }
    }

    Ok(())
}

pub fn normalize_text(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|c| match c {
            'á' => 'a',
            'é' => 'e',
            'í' => 'i',
            'ó' => 'o',
            'ú' | 'ü' => 'u',
            'ñ' => 'n',
            _ => c,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migrations() {
        let db = Database::open_in_memory().unwrap();
        run_migrations(&db).unwrap();

        db.with_connection(|conn| {
            let version: i32 =
                conn.query_row("SELECT MAX(version) FROM schema_version", [], |row| {
                    row.get(0)
                })
                .unwrap();

            assert_eq!(version, MIGRATIONS.len() as i32);
            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn test_normalize_text() {
        assert_eq!(normalize_text("Ácido"), "acido");
        assert_eq!(normalize_text("Niño"), "nino");
        assert_eq!(normalize_text("CAFÉ"), "cafe");
    }
}
