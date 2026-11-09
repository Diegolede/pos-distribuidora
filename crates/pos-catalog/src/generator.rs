use chrono::Utc;
use pos_core::{Currency, Money, Product};
use rust_decimal_macros::dec;

const CATEGORIES: &[&str] = &[
    "Iyosei",
    "Aerolom",
    "Suelto",
    "Higiene Personal",
    "Perfumería",
    "Cosmética",
    "Cabello",
    "Skin Care",
    "Maquillaje",
    "Accesorios",
];

const PRODUCT_TYPES: &[&str] = &[
    "Shampoo",
    "Acondicionador",
    "Tratamiento",
    "Mascarilla",
    "Serum",
    "Crema",
    "Loción",
    "Gel",
    "Spray",
    "Aceite",
    "Bálsamo",
    "Exfoliante",
    "Limpiador",
    "Tónico",
    "Desodorante",
    "Jabón",
    "Colonia",
    "Perfume",
    "Lápiz",
    "Brillo",
    "Base",
    "Polvo",
    "Rubor",
    "Sombra",
    "Máscara",
];

const VARIANTS: &[&str] = &[
    "Natural",
    "Hidratante",
    "Nutritivo",
    "Reparador",
    "Anti-edad",
    "Suavizante",
    "Fortalecedor",
    "Voluminizador",
    "Anti-frizz",
    "Brillo intenso",
    "Extra suave",
    "Para cabello graso",
    "Para cabello seco",
    "Para piel sensible",
    "Matificante",
    "Iluminador",
    "Larga duración",
    "Hipoalergénico",
    "Con protección UV",
    "Dermatológico",
];

const SIZES: &[&str] = &[
    "50ml",
    "100ml",
    "150ml",
    "200ml",
    "250ml",
    "300ml",
    "400ml",
    "500ml",
    "1L",
    "50g",
    "100g",
    "150g",
    "200g",
    "250g",
    "85gr",
    "290gr",
    "240cc",
];

pub fn generate_test_products(count: usize, start_id: i64) -> Vec<Product> {
    let mut products = Vec::with_capacity(count);
    
    for i in 0..count {
        let category_idx = i % CATEGORIES.len();
        let product_idx = (i / CATEGORIES.len()) % PRODUCT_TYPES.len();
        let variant_idx = (i / (CATEGORIES.len() * PRODUCT_TYPES.len())) % VARIANTS.len();
        let size_idx = i % SIZES.len();

        let category = CATEGORIES[category_idx];
        let product_type = PRODUCT_TYPES[product_idx];
        let variant = VARIANTS[variant_idx];
        let size = SIZES[size_idx];

        let name = format!("{} {} {} {}", category, product_type, variant, size)
            .to_uppercase();

        let base_price = 1000 + ((i * 137) % 15000) as i64;
        let price_professional = Money::from_pesos(base_price);
        let price_public = Money::from_pesos((base_price as f64 * 1.25) as i64);

        let description = Some(format!(
            "{} {} de {} para uso {}. {}",
            product_type,
            variant.to_lowercase(),
            category,
            if i % 2 == 0 { "profesional" } else { "diario" },
            size
        ));

        products.push(Product {
            id: start_id + i as i64,
            name,
            description,
            price_professional: Some(price_professional),
            price_public: Some(price_public),
            price_professional_calculated: false,
            price_public_calculated: false,
            category: Some(category.to_string()),
            sku: Some(format!("TEST-{:05}", start_id + i as i64)),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        });
    }

    products
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_products() {
        let products = generate_test_products(100, 1);
        assert_eq!(products.len(), 100);
        
        assert!(products[0].name.contains("IYOSEI"));
        assert!(products[0].sku.is_some());
        assert!(products[0].price_professional.is_some());
        assert!(products[0].price_public.is_some());

        for product in &products {
            assert!(product.sku.as_ref().unwrap().starts_with("TEST-"));
        }
    }

    #[test]
    fn test_product_uniqueness() {
        let products = generate_test_products(1000, 1);
        let mut names = std::collections::HashSet::new();
        let mut skus = std::collections::HashSet::new();

        for product in &products {
            assert!(names.insert(&product.name), "Nombre duplicado: {}", product.name);
            assert!(skus.insert(&product.sku), "SKU duplicado: {:?}", product.sku);
        }
    }
}
