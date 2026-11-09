use crate::money::Money;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub type ProductId = i64;
pub type SaleId = i64;
pub type CategoryId = i64;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Product {
    pub id: ProductId,
    pub name: String,
    pub description: Option<String>,
    pub price_professional: Option<Money>,
    pub price_public: Option<Money>,
    /// True si el precio profesional fue calculado automáticamente (no vino del Excel)
    #[serde(default)]
    pub price_professional_calculated: bool,
    /// True si el precio público fue calculado automáticamente (no vino del Excel)
    #[serde(default)]
    pub price_public_calculated: bool,
    pub category: Option<String>,
    pub sku: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PriceType {
    Professional,
    Public,
    Inherit, // Hereda del selector general del carrito
}

impl PriceType {
    pub fn as_str(&self) -> &str {
        match self {
            PriceType::Professional => "Profesional",
            PriceType::Public => "Público",
            PriceType::Inherit => "Heredar",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CartItem {
    pub product_id: ProductId,
    pub product_name: String,
    pub quantity: u32,
    pub price_type: PriceType,
    pub unit_price: Money,
}

impl CartItem {
    pub fn total(&self) -> Money {
        self.unit_price * self.quantity
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaymentMethod {
    Cash,
    Card,
    Transfer,
}

impl PaymentMethod {
    pub fn as_str(&self) -> &str {
        match self {
            PaymentMethod::Cash => "Efectivo",
            PaymentMethod::Card => "Tarjeta",
            PaymentMethod::Transfer => "Transferencia",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Efectivo" => Some(PaymentMethod::Cash),
            "Tarjeta" => Some(PaymentMethod::Card),
            "Transferencia" => Some(PaymentMethod::Transfer),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sale {
    pub id: SaleId,
    pub sale_number: String,
    pub timestamp: DateTime<Utc>,
    pub items: Vec<SaleItem>,
    pub total: Money,
    pub payment_method: PaymentMethod,
    pub cash_received: Option<Money>,
    pub cash_change: Option<Money>,
    pub is_voided: bool,
    pub void_reason: Option<String>,
    pub void_timestamp: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SaleItem {
    pub product_id: ProductId,
    pub product_name: String,
    pub quantity: u32,
    pub unit_price: Money,
    pub price_type: PriceType,
    pub subtotal: Money,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthlySummary {
    pub year: i32,
    pub month: u32,
    pub total_sales: Money,
    pub sales_count: u32,
    pub units_sold: u32,
    pub voided_sales: u32,
    pub voided_total: Money,
    pub net_total: Money,
    pub by_payment_method: Vec<(PaymentMethod, Money, u32)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportPreview {
    pub total_rows: usize,
    pub valid_rows: usize,
    pub new_products: usize,
    pub updates: usize,
    pub errors: Vec<ImportError>,
    #[serde(default)]
    pub calculated_professional_prices: usize,
    #[serde(default)]
    pub calculated_public_prices: usize,
    #[serde(default)]
    pub products_without_prices: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportError {
    pub row: usize,
    pub column: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanyInfo {
    pub name: String,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub tax_id: Option<String>,
}

impl Default for CompanyInfo {
    fn default() -> Self {
        Self {
            name: "Distribuidora".to_string(),
            address: None,
            phone: None,
            email: None,
            tax_id: None,
        }
    }
}
