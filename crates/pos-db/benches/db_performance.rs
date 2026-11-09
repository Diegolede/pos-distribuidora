use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use pos_core::{Currency, Money, Product};
use pos_db::Database;
use pos_catalog::generate_test_products;
use pos_cart::Cart;
use std::time::Duration;

fn setup_db_with_products(count: usize) -> (Database, Vec<Product>) {
    let db = Database::open_in_memory().unwrap();
    pos_db::migrations::run_migrations(&db).unwrap();
    
    let products = generate_test_products(count, 1);
    for product in &products {
        db.insert_product(product).unwrap();
    }
    
    (db, products)
}

fn benchmark_search_by_name(c: &mut Criterion) {
    let mut group = c.benchmark_group("search_by_name");
    group.measurement_time(Duration::from_secs(10));
    
    for size in [100, 1000, 5000] {
        let (db, _products) = setup_db_with_products(size);
        
        group.bench_with_input(
            BenchmarkId::new("exact", size),
            &size,
            |b, _| {
                b.iter(|| {
                    db.search_products(black_box("TEST"), None, 10)
                        .unwrap()
                });
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("partial", size),
            &size,
            |b, _| {
                b.iter(|| {
                    db.search_products(black_box("producto"), None, 10)
                        .unwrap()
                });
            },
        );
    }
    
    group.finish();
}

fn benchmark_search_by_category(c: &mut Criterion) {
    let mut group = c.benchmark_group("search_by_category");
    group.measurement_time(Duration::from_secs(10));
    
    for size in [100, 1000, 5000] {
        let (db, _products) = setup_db_with_products(size);
        
        group.bench_with_input(
            BenchmarkId::new("category_filter", size),
            &size,
            |b, _| {
                b.iter(|| {
                    db.search_products(
                        black_box("TEST"),
                        Some(black_box("Categoría-A")),
                        10
                    ).unwrap()
                });
            },
        );
    }
    
    group.finish();
}

fn benchmark_cart_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("cart_operations");
    group.measurement_time(Duration::from_secs(10));
    
    let (_db, products) = setup_db_with_products(100);
    
    group.bench_function("add_product", |b| {
        b.iter(|| {
            let mut cart = Cart::new();
            let product = &products[0];
            cart.add_product(black_box(product), black_box(1)).unwrap();
        });
    });
    
    group.bench_function("add_10_products", |b| {
        b.iter(|| {
            let mut cart = Cart::new();
            for i in 0..10 {
                let product = &products[i];
                cart.add_product(product, 1).unwrap();
            }
        });
    });
    
    group.bench_function("total_10_items", |b| {
        let mut cart = Cart::new();
        for i in 0..10 {
            let product = &products[i];
            cart.add_product(product, 2).unwrap();
        }
        
        b.iter(|| {
            black_box(cart.total());
        });
    });
    
    group.bench_function("remove_item", |b| {
        let mut cart = Cart::new();
        let product = &products[0];
        cart.add_product(product, 1).unwrap();
        
        b.iter(|| {
            let mut c = cart.clone();
            c.remove_item(black_box(product.id)).unwrap();
        });
    });
    
    group.finish();
}

fn benchmark_product_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("product_insert");
    group.measurement_time(Duration::from_secs(10));
    
    group.bench_function("insert_single", |b| {
        let db = Database::open_in_memory().unwrap();
        pos_db::migrations::run_migrations(&db).unwrap();
        
        let product = Product {
            id: 0,
            name: "Test Product".to_string(),
            description: Some("Description".to_string()),
            price_professional: Some(Money::from_pesos(1000)),
            price_public: Some(Money::from_pesos(1500)),
            price_professional_calculated: false,
            price_public_calculated: false,
            category: Some("Test".to_string()),
            sku: Some("TEST-001".to_string()),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        
        b.iter(|| {
            db.insert_product(black_box(&product)).unwrap();
        });
    });
    
    group.finish();
}

fn benchmark_full_text_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("fts_search");
    group.measurement_time(Duration::from_secs(10));
    
    for size in [100, 1000, 5000] {
        let (db, _products) = setup_db_with_products(size);
        
        // Búsqueda con diacríticos
        group.bench_with_input(
            BenchmarkId::new("with_diacritics", size),
            &size,
            |b, _| {
                b.iter(|| {
                    db.search_products(black_box("categoría"), None, 10)
                        .unwrap()
                });
            },
        );
        
        // Búsqueda sin diacríticos (debe encontrar resultados igual)
        group.bench_with_input(
            BenchmarkId::new("without_diacritics", size),
            &size,
            |b, _| {
                b.iter(|| {
                    db.search_products(black_box("categoria"), None, 10)
                        .unwrap()
                });
            },
        );
    }
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_search_by_name,
    benchmark_search_by_category,
    benchmark_cart_operations,
    benchmark_product_insert,
    benchmark_full_text_search
);

criterion_main!(benches);
