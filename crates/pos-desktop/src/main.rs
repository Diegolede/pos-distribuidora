// Prevents additional console window on Windows in release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod state;

use crate::state::AppState;
use directories::ProjectDirs;
use std::sync::Arc;
use tracing_subscriber::{fmt, EnvFilter};

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let state = Arc::new(AppState::new().expect("Error al inicializar la aplicación"));

    tauri::Builder::default()
        .manage(state)
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            commands::get_products,
            commands::search_products,
            commands::get_categories,
            commands::import_excel_preview,
            commands::import_excel,
            commands::generate_test_data,
            commands::add_to_cart,
            commands::update_cart_quantity,
            commands::remove_from_cart,
            commands::set_cart_price_type,
            commands::set_default_price_type,
            commands::reset_cart_prices,
            commands::get_cart,
            commands::clear_cart,
            commands::create_sale,
            commands::void_sale,
            commands::get_sale,
            commands::get_monthly_summary,
            commands::get_sales_for_month,
            commands::format_ticket,
            commands::save_ticket_file,
            commands::get_company_info,
            commands::update_company_info,
        ])
        .run(tauri::generate_context!())
        .expect("Error al ejecutar la aplicación");
}
