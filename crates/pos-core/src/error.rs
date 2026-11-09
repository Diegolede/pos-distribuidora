use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Error de base de datos: {0}")]
    Database(String),

    #[error("Producto no encontrado: {0}")]
    ProductNotFound(String),

    #[error("Error de validación: {0}")]
    Validation(String),

    #[error("Error de importación: {0}")]
    Import(String),

    #[error("Error de carrito: {0}")]
    Cart(String),

    #[error("Error de venta: {0}")]
    Sale(String),

    #[error("Error de impresión: {0}")]
    Print(String),

    #[error("Error de actualización: {0}")]
    Update(String),

    #[error("Error de precio: {0}")]
    Price(String),

    #[error("Error de IO: {0}")]
    Io(#[from] std::io::Error),

    #[error("Error de serialización: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
