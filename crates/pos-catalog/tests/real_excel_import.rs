use pos_catalog::ExcelImporter;
use pos_db::{init_database, Database};
use std::path::Path;

#[test]
#[ignore] // Ejecutar manualmente con: cargo test --test real_excel_import -- --ignored --nocapture
fn test_import_real_excel() {
    let excel_path = "/workspace/uploads/catalogo_09cd.xlsx";
    
    if !Path::new(excel_path).exists() {
        println!("⚠️  Excel no encontrado en: {}", excel_path);
        println!("   Este test requiere el archivo real del usuario");
        return;
    }

    // Base de datos en memoria para testing
    let db = Database::open_in_memory().unwrap();
    pos_db::migrations::run_migrations(&db).unwrap();

    println!("\n📊 IMPORTACIÓN DEL EXCEL REAL");
    println!("================================\n");

    // Abrir el Excel
    let importer = ExcelImporter::open(excel_path).unwrap();
    println!("✅ Excel abierto: {}", excel_path);
    println!("   Hojas disponibles: {:?}\n", importer.sheet_names);

    // Preview de la importación (hoja "Productos", encabezados en fila 0)
    println!("🔍 PREVIEW DE IMPORTACIÓN...");
    let preview = importer.preview_import("Productos", 0, &db).unwrap();

    println!("\n📈 RESULTADOS DEL PREVIEW:");
    println!("   Total de filas procesadas: {}", preview.total_rows);
    println!("   Filas válidas: {}", preview.valid_rows);
    println!("   Productos nuevos: {}", preview.new_products);
    println!("   Productos a actualizar: {}", preview.updates);
    println!("   Errores encontrados: {}\n", preview.errors.len());

    if !preview.errors.is_empty() {
        println!("❌ ERRORES DETECTADOS ({} filas con datos parciales inválidos):", preview.errors.len());
        for (i, error) in preview.errors.iter().take(10).enumerate() {
            println!("   {}. Fila {}: {}", i + 1, error.row, error.message);
        }
        if preview.errors.len() > 10 {
            println!("   ... y {} errores más", preview.errors.len() - 10);
        }
        println!();
    }

    // Importación real
    println!("💾 EJECUTANDO IMPORTACIÓN REAL...");
    let result = importer.import_products("Productos", 0, &db).unwrap();

    println!("\n✅ IMPORTACIÓN COMPLETADA:");
    println!("   Productos nuevos insertados: {}", result.new_products);
    println!("   Productos actualizados: {}", result.updates);
    println!("   Total exitosos: {}", result.valid_rows);
    println!("   Errores finales: {}\n", result.errors.len());

    // Verificar en la base de datos
    let total_count = db.count_products().unwrap();
    println!("📦 VERIFICACIÓN EN BASE DE DATOS:");
    println!("   Total de productos en DB: {}\n", total_count);

    // Listar categorías encontradas
    let categories = db.get_categories().unwrap();
    println!("🏷️  CATEGORÍAS ENCONTRADAS ({}):", categories.len());
    for (i, cat) in categories.iter().enumerate() {
        println!("   {}. {}", i + 1, cat);
    }
    println!();

    // Prueba de reimportación (no debe duplicar)
    println!("🔄 PRUEBA DE REIMPORTACIÓN (no debe duplicar)...");
    let reimport = importer.import_products("Productos", 0, &db).unwrap();
    let new_count = db.count_products().unwrap();

    println!("   Productos después de reimportar: {}", new_count);
    println!("   Nuevos (debe ser 0): {}", reimport.new_products);
    println!("   Actualizados: {}", reimport.updates);

    assert_eq!(
        total_count, new_count,
        "La reimportación NO debe crear productos duplicados"
    );
    println!("\n✅ REIMPORTACIÓN: Sin duplicados confirmado\n");

    // Muestra de productos importados
    println!("📋 MUESTRA DE PRODUCTOS IMPORTADOS (primeros 5):");
    let sample = db.get_all_products(Some(5)).unwrap();
    for (i, product) in sample.iter().enumerate() {
        println!("   {}. {} (Cat: {})", 
            i + 1,
            product.name,
            product.category.as_deref().unwrap_or("Sin categoría")
        );
        println!("      Precio Prof: {:?} | Público: {:?}",
            product.price_professional.map(|p| p.to_cents()),
            product.price_public.map(|p| p.to_cents())
        );
    }

    println!("\n================================");
    println!("✅ PRUEBA COMPLETADA");
    println!("================================\n");
}
