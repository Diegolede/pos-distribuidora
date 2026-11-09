use criterion::{black_box, criterion_group, criterion_main, Criterion};
use pos_cart::Cart;
use pos_catalog::generate_test_products;
use pos_core::{Money, PriceType};
use pos_db::Database;

fn create_test_products(n: usize) -> Vec<pos_core::Product> {
    generate_test_products(n, 1)
}

fn bench_cart_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("cart_operations");
    
    let products = create_test_products(100);
    
    group.bench_function("add_to_cart", |b| {
        b.iter(|| {
            let mut cart = Cart::new();
            cart.add_product(black_box(&products[0]), black_box(1)).unwrap();
        });
    });
    
    group.bench_function("add_10_products", |b| {
        b.iter(|| {
            let mut cart = Cart::new();
            for i in 0..10 {
                cart.add_product(&products[i], 1).unwrap();
            }
        });
    });
    
    group.bench_function("update_quantity", |b| {
        let mut cart = Cart::new();
        cart.add_product(&products[0], 1).unwrap();
        b.iter(|| {
            cart.update_quantity(black_box(products[0].id), black_box(5)).unwrap();
        });
    });
    
    group.bench_function("set_item_price_type", |b| {
        let mut cart = Cart::new();
        cart.add_product(&products[0], 1).unwrap();
        b.iter(|| {
            cart.set_item_price_type(black_box(products[0].id), black_box(PriceType::Professional), black_box(&products[0])).unwrap();
        });
    });
    
    group.bench_function("calculate_total", |b| {
        let mut cart = Cart::new();
        for i in 0..10 {
            cart.add_product(&products[i], 2).unwrap();
        }
        b.iter(|| {
            black_box(cart.total())
        });
    });
    
    group.bench_function("clear_cart", |b| {
        b.iter(|| {
            let mut cart = Cart::new();
            for i in 0..5 {
                cart.add_product(&products[i], 1).ok();
            }
            cart.clear();
        });
    });
    
    group.finish();
}

criterion_group!(benches, bench_cart_operations);
criterion_main!(benches);
