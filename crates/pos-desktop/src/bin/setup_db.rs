use directories::ProjectDirs;
use pos_catalog::import::ExcelImporter;
use pos_db::init_database;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get the app data directory
    let proj_dirs = ProjectDirs::from("com", "distribuidora", "pos")
        .expect("No se pudo determinar el directorio de datos");
    let data_dir = proj_dirs.data_dir();
    std::fs::create_dir_all(data_dir)?;

    let db_path = data_dir.join("pos.db");
    println!("Inicializando base de datos en: {:?}", db_path);

    // Initialize database
    let db = init_database(&db_path)?;

    // Import the real catalog
    let excel_path = "/workspace/uploads/catalogo_09cd.xlsx";
    println!("Importando catálogo desde: {}", excel_path);
    
    let importer = ExcelImporter::open(excel_path)?;
    let result = importer.import_products("Productos", 0, &db)?;

    println!("\n✅ IMPORTACIÓN COMPLETADA:");
    println!("   Productos nuevos insertados: {}", result.new_products);
    println!("   Productos actualizados: {}", result.updates);
    println!("   Total exitosos: {}", result.valid_rows);
    println!("   Errores: {}", result.errors.len());

    // Verify import
    let products = db.get_all_products(None)?;
    println!("\n📦 Total de productos en DB: {}", products.len());

    let categories = db.get_categories()?;
    println!("🏷️  Total de categorías: {}", categories.len());

    Ok(())
}
