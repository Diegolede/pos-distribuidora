use chrono_tz::America::Argentina::Buenos_Aires;
use directories::ProjectDirs;
use pos_cart::Cart;
use pos_catalog::ExcelImporter;
use pos_core::{CompanyInfo, Error};
use pos_db::Database;
use pos_sales::SalesManager;
use std::path::PathBuf;
use std::sync::Mutex;

pub struct AppState {
    pub db: Database,
    pub sales: SalesManager,
    pub cart: Mutex<Cart>,
    pub company_info: Mutex<CompanyInfo>,
    pub data_dir: PathBuf,
}

impl AppState {
    pub fn new() -> Result<Self, Error> {
        let proj_dirs = ProjectDirs::from("com", "distribuidora", "pos")
            .ok_or_else(|| Error::Other(anyhow::anyhow!("No se pudo determinar el directorio de datos")))?;

        let data_dir = proj_dirs.data_dir().to_path_buf();
        std::fs::create_dir_all(&data_dir)?;

        let db_path = data_dir.join("pos.db");
        let db = pos_db::init_database(&db_path)?;
        
        // Auto-importar catálogo si la DB está vacía
        let product_count = db.get_all_products(Some(1)).map(|p| p.len()).unwrap_or(0);
        if product_count == 0 {
            tracing::info!("Base de datos vacía, importando catálogo...");
            if let Ok(excel_path) = std::env::var("POS_CATALOG_PATH") {
                if std::path::Path::new(&excel_path).exists() {
                    match ExcelImporter::open(&excel_path) {
                        Ok(importer) => {
                            match importer.import_products("Productos", 1, &db) {
                                Ok(result) => {
                                    tracing::info!(
                                        "✓ Catálogo importado: {} productos, {} categorías",
                                        result.new_products + result.updates,
                                        db.get_categories().unwrap_or_default().len()
                                    );
                                }
                                Err(e) => tracing::warn!("Error al importar catálogo: {}", e),
                            }
                        }
                        Err(e) => tracing::warn!("Error al abrir Excel: {}", e),
                    }
                }
            }
        }

        let sales = SalesManager::new(db.clone(), Buenos_Aires);

        let company_info = Self::load_company_info(&db)?;

        Ok(Self {
            db,
            sales,
            cart: Mutex::new(Cart::new()),
            company_info: Mutex::new(company_info),
            data_dir,
        })
    }

    fn load_company_info(db: &Database) -> Result<CompanyInfo, Error> {
        let info = db.with_connection(|conn| {
            let name: String = conn
                .query_row(
                    "SELECT value FROM config WHERE key = 'company_name'",
                    [],
                    |row| row.get(0),
                )
                .unwrap_or_else(|_| "Distribuidora".to_string());

            let address: Option<String> = conn
                .query_row(
                    "SELECT value FROM config WHERE key = 'company_address'",
                    [],
                    |row| row.get(0),
                )
                .ok();

            let phone: Option<String> = conn
                .query_row(
                    "SELECT value FROM config WHERE key = 'company_phone'",
                    [],
                    |row| row.get(0),
                )
                .ok();

            let email: Option<String> = conn
                .query_row(
                    "SELECT value FROM config WHERE key = 'company_email'",
                    [],
                    |row| row.get(0),
                )
                .ok();

            let tax_id: Option<String> = conn
                .query_row(
                    "SELECT value FROM config WHERE key = 'company_tax_id'",
                    [],
                    |row| row.get(0),
                )
                .ok();

            Ok(CompanyInfo {
                name,
                address,
                phone,
                email,
                tax_id,
            })
        })?;

        Ok(info)
    }

    pub fn save_company_info(&self, info: &CompanyInfo) -> Result<(), Error> {
        self.db.transaction(|tx| {
            let save_field = |key: &str, value: Option<&String>| -> Result<(), Error> {
                if let Some(val) = value {
                    tx.execute(
                        "INSERT OR REPLACE INTO config (key, value, updated_at) VALUES (?1, ?2, datetime('now'))",
                        [key, val],
                    )
                    .map_err(|e| Error::Database(e.to_string()))?;
                }
                Ok(())
            };

            save_field("company_name", Some(&info.name))?;
            save_field("company_address", info.address.as_ref())?;
            save_field("company_phone", info.phone.as_ref())?;
            save_field("company_email", info.email.as_ref())?;
            save_field("company_tax_id", info.tax_id.as_ref())?;

            Ok(())
        })?;

        *self.company_info.lock().unwrap() = info.clone();

        Ok(())
    }
}
