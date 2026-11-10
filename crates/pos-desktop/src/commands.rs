use crate::state::AppState;
use pos_cart::Cart;
use pos_catalog::{generate_test_products, ExcelImporter, OptimizedImporter};
use pos_core::{
    CartItem, CompanyInfo, Error, ImportPreview, Money, MonthlySummary, PaymentMethod, PriceType,
    Product, ProductId, Sale,
};
use pos_printer::{save_ticket_to_file, TicketFormatter, TicketWidth};
use rust_decimal::Decimal;
use std::str::FromStr;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};

#[tauri::command]
pub fn get_products(
    state: State<Arc<AppState>>,
    limit: Option<usize>,
) -> Result<Vec<Product>, String> {
    state
        .db
        .get_all_products(limit)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn search_products(
    state: State<Arc<AppState>>,
    query: String,
    category: Option<String>,
    limit: Option<usize>,
) -> Result<Vec<Product>, String> {
    state
        .db
        .search_products(&query, category.as_deref(), limit.unwrap_or(100))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_categories(state: State<Arc<AppState>>) -> Result<Vec<String>, String> {
    state.db.get_categories().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn import_excel_preview(
    state: State<Arc<AppState>>,
    file_path: String,
    sheet_name: String,
    header_row: usize,
) -> Result<ImportPreview, String> {
    let importer = ExcelImporter::open(&file_path).map_err(|e| e.to_string())?;
    importer
        .preview_import(&sheet_name, header_row, &state.db)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn import_excel(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    file_path: String,
    sheet_name: String,
    header_row: usize,
) -> Result<ImportPreview, String> {
    let db = state.db.clone();
    
    // Ejecutar importación en hilo bloqueante para no congelar la UI
    let result = tauri::async_runtime::spawn_blocking(move || {
        let importer = ExcelImporter::open(&file_path).map_err(|e| e.to_string())?;
        let optimized = OptimizedImporter::new(importer);
        
        // Emitir eventos de progreso cada N filas
        let result = optimized.import_products_optimized(
            &sheet_name,
            header_row,
            &db,
            |current, total| {
                let _ = app.emit("import-progress", serde_json::json!({
                    "current": current,
                    "total": total,
                    "percent": if total > 0 { (current * 100) / total } else { 0 }
                }));
            },
        );
        
        result.map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| format!("Error en spawn: {}", e))??;
    
    Ok(result)
}

#[tauri::command]
pub fn generate_test_data(
    state: State<Arc<AppState>>,
    count: usize,
) -> Result<usize, String> {
    let current_count = state.db.count_products().map_err(|e| e.to_string())?;
    let start_id = (current_count + 1) as i64;

    let products = generate_test_products(count, start_id);

    for product in products {
        state
            .db
            .insert_product(&product)
            .map_err(|e| e.to_string())?;
    }

    Ok(count)
}

#[tauri::command]
pub fn add_to_cart(
    state: State<Arc<AppState>>,
    product_id: ProductId,
    quantity: u32,
) -> Result<Cart, String> {
    let product = state
        .db
        .get_product(product_id)
        .map_err(|e| e.to_string())?;

    let mut cart = state.cart.lock().unwrap();
    cart.add_product(&product, quantity)
        .map_err(|e| e.to_string())?;

    Ok(cart.clone())
}

#[tauri::command]
pub fn update_cart_quantity(
    state: State<Arc<AppState>>,
    product_id: ProductId,
    quantity: u32,
) -> Result<Cart, String> {
    let mut cart = state.cart.lock().unwrap();
    cart.update_quantity(product_id, quantity)
        .map_err(|e| e.to_string())?;
    Ok(cart.clone())
}

#[tauri::command]
pub fn remove_from_cart(
    state: State<Arc<AppState>>,
    product_id: ProductId,
) -> Result<Cart, String> {
    let mut cart = state.cart.lock().unwrap();
    cart.remove_item(product_id).map_err(|e| e.to_string())?;
    Ok(cart.clone())
}

#[tauri::command]
pub fn set_cart_price_type(
    state: State<Arc<AppState>>,
    product_id: ProductId,
    price_type: String,
) -> Result<Cart, String> {
    let price_type = match price_type.as_str() {
        "Professional" => PriceType::Professional,
        "Public" => PriceType::Public,
        "Inherit" => PriceType::Inherit,
        _ => return Err("Tipo de precio inválido".to_string()),
    };

    let product = state
        .db
        .get_product(product_id)
        .map_err(|e| e.to_string())?;

    let mut cart = state.cart.lock().unwrap();
    cart.set_item_price_type(product_id, price_type, &product)
        .map_err(|e| e.to_string())?;

    Ok(cart.clone())
}

#[tauri::command]
pub fn set_default_price_type(
    state: State<Arc<AppState>>,
    price_type: String,
) -> Result<Cart, String> {
    let price_type = match price_type.as_str() {
        "Professional" => PriceType::Professional,
        "Public" => PriceType::Public,
        _ => return Err("Tipo de precio inválido".to_string()),
    };

    let mut cart = state.cart.lock().unwrap();
    let product_ids: Vec<ProductId> = cart.items.iter().map(|i| i.product_id).collect();

    let products: Vec<Product> = product_ids
        .into_iter()
        .filter_map(|id| state.db.get_product(id).ok())
        .collect();

    cart.set_default_price_type(price_type, &products)
        .map_err(|e| e.to_string())?;

    Ok(cart.clone())
}

#[tauri::command]
pub fn reset_cart_prices(state: State<Arc<AppState>>) -> Result<Cart, String> {
    let mut cart = state.cart.lock().unwrap();
    let product_ids: Vec<ProductId> = cart.items.iter().map(|i| i.product_id).collect();

    let products: Vec<Product> = product_ids
        .into_iter()
        .filter_map(|id| state.db.get_product(id).ok())
        .collect();

    cart.reset_all_to_default(&products)
        .map_err(|e| e.to_string())?;

    Ok(cart.clone())
}

#[tauri::command]
pub fn get_cart(state: State<Arc<AppState>>) -> Result<Cart, String> {
    let cart = state.cart.lock().unwrap();
    Ok(cart.clone())
}

#[tauri::command]
pub fn clear_cart(state: State<Arc<AppState>>) -> Result<(), String> {
    let mut cart = state.cart.lock().unwrap();
    cart.clear();
    Ok(())
}

#[tauri::command]
pub fn create_sale(
    state: State<Arc<AppState>>,
    payment_method: String,
    cash_received_pesos: Option<i64>,
) -> Result<Sale, String> {
    let payment_method = PaymentMethod::from_str(&payment_method)
        .ok_or_else(|| "Método de pago inválido".to_string())?;

    let cash_received = cash_received_pesos.map(|p| Money::from_pesos(p));

    let cart = state.cart.lock().unwrap();
    let items = cart.items.clone();
    drop(cart);

    let sale = state
        .sales
        .create_sale(items, payment_method, cash_received)
        .map_err(|e| e.to_string())?;

    let mut cart = state.cart.lock().unwrap();
    cart.clear();

    Ok(sale)
}

#[tauri::command]
pub fn void_sale(
    state: State<Arc<AppState>>,
    sale_id: i64,
    reason: String,
) -> Result<(), String> {
    state
        .sales
        .void_sale(sale_id, reason)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_sale(state: State<Arc<AppState>>, sale_id: i64) -> Result<Sale, String> {
    state.sales.get_sale(sale_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_monthly_summary(
    state: State<Arc<AppState>>,
    year: i32,
    month: u32,
) -> Result<MonthlySummary, String> {
    state
        .sales
        .get_monthly_summary(year, month)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_sales_for_month(
    state: State<Arc<AppState>>,
    year: i32,
    month: u32,
) -> Result<Vec<Sale>, String> {
    state
        .sales
        .get_sales_for_month(year, month)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn format_ticket(
    state: State<Arc<AppState>>,
    sale_id: i64,
    width_mm: u32,
) -> Result<String, String> {
    let sale = state.sales.get_sale(sale_id).map_err(|e| e.to_string())?;

    let width = match width_mm {
        58 => TicketWidth::Mm58,
        80 => TicketWidth::Mm80,
        _ => return Err("Ancho de ticket inválido (58 o 80)".to_string()),
    };

    let company = state.company_info.lock().unwrap().clone();
    let formatter = TicketFormatter::new(width, company);

    formatter.format_sale(&sale).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_ticket_file(
    state: State<Arc<AppState>>,
    ticket_content: String,
    filename: String,
) -> Result<String, String> {
    let tickets_dir = state.data_dir.join("tickets");
    std::fs::create_dir_all(&tickets_dir).map_err(|e| e.to_string())?;

    let file_path = tickets_dir.join(filename);
    save_ticket_to_file(&ticket_content, &file_path).map_err(|e| e.to_string())?;

    Ok(file_path.to_string_lossy().to_string())
}

#[tauri::command]
pub fn get_company_info(state: State<Arc<AppState>>) -> Result<CompanyInfo, String> {
    let info = state.company_info.lock().unwrap().clone();
    Ok(info)
}

#[tauri::command]
pub fn update_company_info(
    state: State<Arc<AppState>>,
    info: CompanyInfo,
) -> Result<(), String> {
    state.save_company_info(&info).map_err(|e| e.to_string())
}
