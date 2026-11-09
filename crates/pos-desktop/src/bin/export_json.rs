use directories::ProjectDirs;
use pos_db::init_database;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proj_dirs = ProjectDirs::from("com", "distribuidora", "pos")
        .expect("No se pudo determinar el directorio de datos");
    let db_path = proj_dirs.data_dir().join("pos.db");

    let db = init_database(&db_path)?;

    // Export products (limit to 300 for reasonable size)
    let products = db.get_all_products(Some(300))?;
    let products_json = serde_json::to_string_pretty(&products)?;
    fs::write("/tmp/products_export.json", products_json)?;
    println!("✅ Exported {} products to /tmp/products_export.json", products.len());

    // Export categories
    let categories = db.get_categories()?;
    let categories_json = serde_json::to_string_pretty(&categories)?;
    fs::write("/tmp/categories_export.json", categories_json)?;
    println!("✅ Exported {} categories to /tmp/categories_export.json", categories.len());

    Ok(())
}
