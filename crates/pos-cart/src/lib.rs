use pos_core::{CartItem, Currency, Error, Money, PriceType, Product, ProductId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cart {
    pub items: Vec<CartItem>,
    pub default_price_type: PriceType,
}

impl Cart {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            default_price_type: PriceType::Professional,
        }
    }

    pub fn add_product(&mut self, product: &Product, quantity: u32) -> Result<(), Error> {
        if quantity == 0 {
            return Err(Error::Cart("La cantidad debe ser mayor a cero".to_string()));
        }

        let price_type = PriceType::Inherit;
        let unit_price = self.resolve_price(product, price_type)?;

        if let Some(existing) = self.items.iter_mut().find(|item| {
            item.product_id == product.id && item.price_type == price_type
        }) {
            existing.quantity += quantity;
        } else {
            self.items.push(CartItem {
                product_id: product.id,
                product_name: product.name.clone(),
                quantity,
                price_type,
                unit_price,
            });
        }

        Ok(())
    }

    pub fn update_quantity(&mut self, product_id: ProductId, quantity: u32) -> Result<(), Error> {
        if quantity == 0 {
            return self.remove_item(product_id);
        }

        let item = self
            .items
            .iter_mut()
            .find(|item| item.product_id == product_id)
            .ok_or_else(|| Error::Cart("Producto no encontrado en el carrito".to_string()))?;

        item.quantity = quantity;
        Ok(())
    }

    pub fn remove_item(&mut self, product_id: ProductId) -> Result<(), Error> {
        let initial_len = self.items.len();
        self.items.retain(|item| item.product_id != product_id);

        if self.items.len() == initial_len {
            return Err(Error::Cart("Producto no encontrado en el carrito".to_string()));
        }

        Ok(())
    }

    pub fn set_item_price_type(
        &mut self,
        product_id: ProductId,
        price_type: PriceType,
        product: &Product,
    ) -> Result<(), Error> {
        let new_price = self.resolve_price(product, price_type)?;
        
        let item = self
            .items
            .iter_mut()
            .find(|item| item.product_id == product_id)
            .ok_or_else(|| Error::Cart("Producto no encontrado en el carrito".to_string()))?;

        item.price_type = price_type;
        item.unit_price = new_price;

        Ok(())
    }

    pub fn set_default_price_type(&mut self, price_type: PriceType, products: &[Product]) -> Result<(), Error> {
        if price_type == PriceType::Inherit {
            return Err(Error::Cart(
                "El tipo de precio predeterminado no puede ser 'Heredar'".to_string(),
            ));
        }

        self.default_price_type = price_type;

        let mut updates = Vec::new();
        for item in &self.items {
            if item.price_type == PriceType::Inherit {
                if let Some(product) = products.iter().find(|p| p.id == item.product_id) {
                    let new_price = self.resolve_price(product, PriceType::Inherit)?;
                    updates.push((item.product_id, new_price));
                }
            }
        }

        for (product_id, new_price) in updates {
            if let Some(item) = self.items.iter_mut().find(|i| i.product_id == product_id) {
                item.unit_price = new_price;
            }
        }

        Ok(())
    }

    pub fn reset_all_to_default(&mut self, products: &[Product]) -> Result<(), Error> {
        let mut updates = Vec::new();
        for item in &self.items {
            if let Some(product) = products.iter().find(|p| p.id == item.product_id) {
                let new_price = self.resolve_price(product, PriceType::Inherit)?;
                updates.push((item.product_id, new_price));
            }
        }

        for item in &mut self.items {
            item.price_type = PriceType::Inherit;
            if let Some((_, new_price)) = updates.iter().find(|(id, _)| *id == item.product_id) {
                item.unit_price = *new_price;
            }
        }

        Ok(())
    }

    pub fn total(&self) -> Money {
        self.items
            .iter()
            .map(|item| item.total())
            .fold(Money::zero(Currency::ARS), |acc, money| acc + money)
    }

    pub fn item_count(&self) -> usize {
        self.items.len()
    }

    pub fn unit_count(&self) -> u32 {
        self.items.iter().map(|item| item.quantity).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn clear(&mut self) {
        self.items.clear();
    }

    pub fn get_effective_price_type(&self, item_price_type: PriceType) -> PriceType {
        match item_price_type {
            PriceType::Inherit => self.default_price_type,
            other => other,
        }
    }

    fn resolve_price(&self, product: &Product, price_type: PriceType) -> Result<Money, Error> {
        let effective_type = self.get_effective_price_type(price_type);

        let price = match effective_type {
            PriceType::Professional => product
                .price_professional
                .ok_or_else(|| {
                    Error::Price(format!(
                        "El producto '{}' no tiene precio profesional",
                        product.name
                    ))
                })?,
            PriceType::Public => product
                .price_public
                .ok_or_else(|| {
                    Error::Price(format!(
                        "El producto '{}' no tiene precio público",
                        product.name
                    ))
                })?,
            PriceType::Inherit => {
                return Err(Error::Price(
                    "Error interno: no se debería resolver 'Heredar'".to_string(),
                ))
            }
        };

        Ok(price)
    }
}

impl Default for Cart {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use rust_decimal_macros::dec;

    fn create_test_product(id: i64, name: &str, prof: i64, public: i64) -> Product {
        Product {
            id,
            name: name.to_string(),
            description: None,
            price_professional: Some(Money::from_pesos(prof)),
            price_public: Some(Money::from_pesos(public)),
            price_professional_calculated: false,
            price_public_calculated: false,
            category: None,
            sku: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn test_cart_basic_operations() {
        let mut cart = Cart::new();
        let product = create_test_product(1, "Producto A", 100, 150);

        cart.add_product(&product, 2).unwrap();
        assert_eq!(cart.item_count(), 1);
        assert_eq!(cart.unit_count(), 2);

        cart.update_quantity(1, 5).unwrap();
        assert_eq!(cart.unit_count(), 5);

        cart.remove_item(1).unwrap();
        assert!(cart.is_empty());
    }

    #[test]
    fn test_cart_price_inheritance() {
        let mut cart = Cart::new();
        cart.default_price_type = PriceType::Professional;

        let product = create_test_product(1, "Producto A", 100, 150);
        cart.add_product(&product, 1).unwrap();

        let total = cart.total();
        assert_eq!(total.to_cents(), 10000);

        cart.set_default_price_type(PriceType::Public, &[product.clone()])
            .unwrap();
        let total = cart.total();
        assert_eq!(total.to_cents(), 15000);
    }

    #[test]
    fn test_cart_mixed_prices() {
        let mut cart = Cart::new();
        cart.default_price_type = PriceType::Professional;

        let product1 = create_test_product(1, "Producto A", 100, 150);
        let product2 = create_test_product(2, "Producto B", 200, 250);

        cart.add_product(&product1, 1).unwrap();
        cart.add_product(&product2, 1).unwrap();

        cart.set_item_price_type(1, PriceType::Public, &product1)
            .unwrap();

        let item1 = cart.items.iter().find(|i| i.product_id == 1).unwrap();
        let item2 = cart.items.iter().find(|i| i.product_id == 2).unwrap();

        assert_eq!(item1.unit_price.to_cents(), 15000);
        assert_eq!(item2.unit_price.to_cents(), 20000);

        cart.set_default_price_type(PriceType::Public, &[product1.clone(), product2.clone()])
            .unwrap();

        let item1 = cart.items.iter().find(|i| i.product_id == 1).unwrap();
        let item2 = cart.items.iter().find(|i| i.product_id == 2).unwrap();

        assert_eq!(item1.unit_price.to_cents(), 15000);
        assert_eq!(item2.unit_price.to_cents(), 25000);
    }

    #[test]
    fn test_cart_case_from_requirements() {
        let mut cart = Cart::new();
        cart.default_price_type = PriceType::Professional;

        let product = create_test_product(1, "Producto A", 100, 150);
        
        cart.add_product(&product, 1).unwrap();
        
        cart.set_item_price_type(1, PriceType::Public, &product)
            .unwrap();

        let item = cart.items.iter().find(|i| i.product_id == 1).unwrap();
        assert_eq!(item.price_type, PriceType::Public);
        assert_eq!(item.unit_price.to_cents(), 15000);

        cart.set_default_price_type(PriceType::Public, &[product.clone()])
            .unwrap();

        let item = cart.items.iter().find(|i| i.product_id == 1).unwrap();
        assert_eq!(item.price_type, PriceType::Public);
        assert_eq!(item.unit_price.to_cents(), 15000);
    }
}
