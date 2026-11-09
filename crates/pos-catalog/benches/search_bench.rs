use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use pos_catalog::generate_test_products;
use pos_db::Database;

fn create_test_db_with_products(n: usize) -> Database {
    let db = Database::open_in_memory().unwrap();
    pos_db::migrations::run_migrations(&db).unwrap();
    
    // Insertar N productos usando el generador
    let products = generate_test_products(n, 1);
    for product in products {
        db.insert_product(&product).unwrap();
    }
    
    db
}

fn bench_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("catalog_search");
    
    for size in [100, 1000, 5000].iter() {
        let db = create_test_db_with_products(*size);
        
        group.bench_with_input(
            BenchmarkId::new("term_search", size),
            size,
            |b, _| {
                b.iter(|| {
                    let results = db.search_products(
                        black_box("PRODUCTO"),
                        None,
                        100
                    ).unwrap();
                    black_box(results)
                });
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("category_filter", size),
            size,
            |b, _| {
                b.iter(|| {
                    // FTS5 requires a non-empty query, using a common term
                    let results = db.search_products(
                        black_box("TEST"),
                        Some("Categoría-A"),
                        100
                    ).unwrap();
                    black_box(results)
                });
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("combined_search", size),
            size,
            |b, _| {
                b.iter(|| {
                    let results = db.search_products(
                        black_box("TEST"),
                        Some("Categoría-B"),
                        50
                    ).unwrap();
                    black_box(results)
                });
            },
        );
    }
    
    group.finish();
}

criterion_group!(benches, bench_search);
criterion_main!(benches);
