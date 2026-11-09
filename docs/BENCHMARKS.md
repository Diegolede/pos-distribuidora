# Benchmarks de Rendimiento

## Hardware de Prueba

- **CPU**: Intel(R) Xeon(R) Processor
- **Cores**: 4 cores, 1 thread per core
- **RAM**: 15 GiB total, ~14 GiB available
- **Storage**: SSD
- **OS**: Linux 6.12

## Metodología

Los benchmarks se ejecutaron con Criterion en modo `release` con optimizaciones completas.
Los tests incluyen datasets de 100, 1,000 y **5,000 productos** (real catalog + synthetic).

## Resultados

### Cart Operations (Carrito)

| Operación | Tiempo | Notas |
|-----------|--------|-------|
| `add_to_cart` | 23.36 ns | Agregar 1 producto |
| `add_10_products` | 359.71 ns | Agregar 10 productos |
| `update_quantity` | 809 ps | Actualizar cantidad |
| `set_item_price_type` | 6.43 ns | Cambiar tipo de precio |
| `calculate_total` | 66.51 ns | Calcular total de 10 items |
| `clear_cart` | 131.27 ns | Limpiar carrito con 5 items |

**Análisis**: Las operaciones del carrito son extremadamente rápidas (nanosegundos), manejando carros con múltiples items sin latencia perceptible.

### Catalog Search (Búsqueda con FTS5)

#### Term Search (Búsqueda por término)

| Dataset | Tiempo | Throughput |
|---------|--------|------------|
| 100 productos | 12.69 µs | ~78.8k búsquedas/seg |
| 1,000 productos | 17.14 µs | ~58.3k búsquedas/seg |
| **5,000 productos** | **17.43 µs** | **~57.4k búsquedas/seg** |

#### Category Filter (Filtro por categoría)

| Dataset | Tiempo | Throughput |
|---------|--------|------------|
| 100 productos | 13.88 µs | ~72.0k búsquedas/seg |
| 1,000 productos | 18.47 µs | ~54.1k búsquedas/seg |
| **5,000 productos** | **18.07 µs** | **~55.3k búsquedas/seg** |

#### Combined Search (Término + Categoría)

| Dataset | Tiempo | Throughput |
|---------|--------|------------|
| 100 productos | 13.78 µs | ~72.6k búsquedas/seg |
| 1,000 productos | 19.72 µs | ~50.7k búsquedas/seg |
| **5,000 productos** | **17.93 µs** | **~55.8k búsquedas/seg** |

**Análisis**: SQLite FTS5 con `remove_diacritics 1` mantiene búsquedas **sub-20 microsegundos** incluso con 5,000 productos. El rendimiento se mantiene prácticamente constante gracias a los índices FTS5, haciendo la búsqueda en tiempo real imperceptible para el usuario (< 0.02 ms).

### Database Operations (Operaciones de BD)

_(Pendiente de completar - benchmarks en ejecución)_

## Conclusiones

1. **Búsqueda instantánea**: FTS5 permite búsqueda en tiempo real con latencias imperceptibles
2. **Escalabilidad**: El rendimiento de búsqueda escala logarítmicamente con el tamaño del catálogo
3. **Carrito eficiente**: Operaciones del carrito en el rango de nanosegundos
4. **Diacríticos**: La configuración `remove_diacritics 1` funciona correctamente sin penalización de rendimiento

## Recomendaciones

- El sistema puede manejar catálogos de 10,000+ productos sin degradación significativa
- La búsqueda en tiempo real (keypress) es viable con actualizaciones cada 150-200ms
- No se requiere paginación agresiva en el frontend (el backend es suficientemente rápido)
