pub mod migrations;
pub mod connection;
pub mod queries;

pub use connection::Database;
pub use queries::*;

use pos_core::Error;
use std::path::Path;

pub fn init_database(db_path: &Path) -> Result<Database, Error> {
    let db = Database::open(db_path)?;
    migrations::run_migrations(&db)?;
    Ok(db)
}
