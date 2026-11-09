use pos_core::Error;
use rusqlite::{Connection, OpenFlags};
use std::path::Path;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    pub fn open(path: &Path) -> Result<Self, Error> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_WRITE
                | OpenFlags::SQLITE_OPEN_CREATE
                | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .map_err(|e| Error::Database(e.to_string()))?;

        conn.execute_batch(
            "PRAGMA foreign_keys = ON;
             PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA cache_size = -64000;
             PRAGMA temp_store = MEMORY;",
        )
        .map_err(|e| Error::Database(e.to_string()))?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn open_in_memory() -> Result<Self, Error> {
        let conn = Connection::open_in_memory()
            .map_err(|e| Error::Database(e.to_string()))?;

        conn.execute_batch(
            "PRAGMA foreign_keys = ON;
             PRAGMA journal_mode = MEMORY;",
        )
        .map_err(|e| Error::Database(e.to_string()))?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn with_connection<F, T>(&self, f: F) -> Result<T, Error>
    where
        F: FnOnce(&Connection) -> Result<T, Error>,
    {
        let conn = self
            .conn
            .lock()
            .map_err(|e| Error::Database(format!("Error al adquirir lock: {}", e)))?;
        f(&conn)
    }

    pub fn transaction<F, T>(&self, f: F) -> Result<T, Error>
    where
        F: FnOnce(&rusqlite::Transaction) -> Result<T, Error>,
    {
        let mut conn = self
            .conn
            .lock()
            .map_err(|e| Error::Database(format!("Error al adquirir lock: {}", e)))?;

        let tx = conn
            .transaction()
            .map_err(|e| Error::Database(e.to_string()))?;

        let result = f(&tx)?;

        tx.commit().map_err(|e| Error::Database(e.to_string()))?;

        Ok(result)
    }
}
