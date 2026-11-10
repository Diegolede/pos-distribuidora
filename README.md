# Sistema POS - Punto de Venta para Distribuidoras

Sistema de punto de venta (POS) completo desarrollado en **Rust + Tauri v2 + React/TypeScript** para distribuidoras argentinas. Interfaz 100% en español, diseño blanco y negro profesional, con soporte para catálogo masivo, importación de Excel, gestión de precios diferenciados (público/profesional), y reportes detallados.

## ✨ Características Principales

### 🛒 Gestión de Ventas
- **Catálogo de productos responsivo** con grid de tarjetas compactas
- **Búsqueda instantánea** con FTS5 (Full-Text Search) y eliminación de diacríticos
- **Filtros por categoría** con píldoras horizontales scrollables
- **Precios diferenciados**: Público y Profesional por producto
- **Carrito inteligente** con control de precios mixtos (Auto/Público/Profesional)
- **Atajos de teclado** para operación rápida (F2, F12, Alt+1-9)
- **Modal de cobro rápido** con montos sugeridos y cálculo de vuelto en tiempo real
- Métodos de pago: Efectivo, Tarjeta de débito, Tarjeta de crédito

### 📦 Importación de Catálogo
- **Asistente de importación** de Excel (.xlsx) con vista previa detallada
- **Cálculo automático de precios** con markup configurable (default 30%)
  - Precio faltante profesional → calculado desde público
  - Precio faltante público → calculado desde profesional
- **Productos sin precio** importados como "no vendibles" (bloqueados en carrito)
- **Actualización inteligente**: Reimportar preserva productos existentes (matching por SKU o nombre+categoría)
- **Categorías jerárquicas** de hasta 5 niveles (Categoría 1 > Categoría 2 > ...)
- **Validación robusta** con reporte de errores por fila

### 📊 Reportes y Gestión
- **Reportes mensuales** de ventas con totales por método de pago
- **Anulación de ventas** con trazabilidad completa
- **Exportación** de reportes (CSV/XLSX) *(próximamente)*
- **Configuración de empresa**: nombre, dirección, teléfono, zona horaria
- **Gestión de markup**: ajuste del porcentaje de recargo para recalcular precios

### 🎨 Diseño y UX
- **Paleta estrictamente blanco y negro** (#000000, #ffffff, grises neutros)
- **Controles segmentados** para selección de precios (estilo iOS/macOS)
- **Grid responsivo** auto-fill minmax(220px, 1fr)
- **Tipografía Inter** con jerarquía clara
- **Animaciones sutiles** (≤150ms)
- **Accesibilidad**: focus visible, contraste alto, atajos de teclado

## 🚀 Instalación y Ejecución

### Requisitos Previos

- **Rust** 1.70+ (con Cargo)
- **Node.js** 18+ (con npm)
- **Dependencias de sistema** (Linux):
  ```bash
  sudo apt-get update
  sudo apt-get install -y \
    libwebkit2gtk-4.1-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev \
    patchelf \
    libssl-dev
  ```

### Desarrollo

```bash
# 1. Clonar el repositorio
git clone <repo-url>
cd <repo-dir>

# 2. Instalar dependencias del frontend
cd crates/pos-desktop/ui
npm install
cd ../../..

# 3. Ejecutar la aplicación en modo desarrollo
cargo tauri dev
```

### Compilación Release

```bash
# Compilar el ejecutable optimizado
cargo tauri build

# El binario estará en:
# - Linux: target/release/pos-desktop
# - AppImage: target/release/bundle/appimage/pos-desktop_X.X.X_amd64.AppImage
```

## 📁 Estructura del Proyecto

```
workspace/
├── crates/
│   ├── pos-core/          # Tipos core (Product, Cart, Sale, Money)
│   ├── pos-db/            # SQLite + migraciones + queries
│   ├── pos-catalog/       # Importación Excel + búsqueda
│   ├── pos-cart/          # Lógica de carrito + precios mixtos
│   ├── pos-sales/         # Creación de ventas + anulaciones
│   ├── pos-printer/       # Impresión de tickets (58/80mm)
│   ├── pos-updater/       # Sistema de actualizaciones (ed25519)
│   └── pos-desktop/       # App Tauri + UI React
│       └── ui/            # Frontend (Vite + React + TS)
├── docs/                  # Documentación técnica
├── uploads/               # Catálogo Excel de ejemplo
└── README.md
```

## 📥 Importación de Productos

### Formato de Excel Esperado

| Columna | Requerido | Descripción |
|---------|-----------|-------------|
| **Nombre** | ✅ Sí | Nombre del producto |
| **Precio Profesional** | ⚠️ Opcional* | Precio mayorista (formato: `1234.56` o `1.234,56`) |
| **Precio Público** | ⚠️ Opcional* | Precio minorista |
| **Categoría 1..5** | ❌ No | Hasta 5 niveles de categorías |
| **SKU** | ❌ No | Código único del producto |
| **Descripción** | ❌ No | Descripción detallada |

*\* Al menos uno de los dos precios es recomendado. Si faltan ambos, el producto se importa como "no vendible".*

### Reglas de Importación

1. **Precio faltante** → Calculado automáticamente con markup del 30%
   - Público = Profesional × (1 + 0.30)
   - Profesional = Público ÷ (1 + 0.30)
2. **Ambos precios faltantes** → Producto importado como "no vendible"
3. **Filas vacías** → Ignoradas silenciosamente
4. **Duplicados** → Identificados por SKU o (nombre + categoría), se actualizan en lugar de crear nuevos
5. **Precios calculados** → Marcados con `*` en la UI, recalculables desde Ajustes

### Ejemplo de Uso

```bash
# Ver resumen de importación real del catálogo incluido:
cargo test --test real_excel_import -- --ignored --nocapture

# Resultado esperado:
# - 2.721 filas válidas procesadas
# - 2.677 productos únicos en base de datos
# - 42 categorías distintas
# - 873 filas vacías ignoradas
# - 48 productos consolidados (duplicados)
```

## ⌨️ Atajos de Teclado

| Atajo | Acción |
|-------|--------|
| **F2** | Focus en buscador |
| **F12** | Abrir modal de cobro |
| **Alt + 1** | Ver todas las categorías |
| **Alt + 2-9** | Seleccionar categoría específica (posiciones 2-9) |
| **Enter** | Confirmar venta (en modal de pago) |
| **Clic / Enter** | Agregar producto al carrito (desde grid) |

## 🧪 Testing

```bash
# Ejecutar todos los tests unitarios
cargo test

# Tests de integración con Excel real
cargo test --test real_excel_import -- --ignored

# Benchmarks de performance (Criterion)
cargo bench

# Resultados esperados:
# - Búsqueda con 5.000 productos: <150ms
# - Operaciones de carrito: <100ms
```

## 🖨️ Impresión de Tickets

- Soporte para impresoras térmicas de **58mm** y **80mm**
- **Vista previa** en pantalla antes de imprimir
- **Exportación a PDF** para envío digital
- **Prueba de fallo de impresión**: venta no se pierde/duplica si falla la impresora

## 🔧 Configuración (Ajustes)

- **Información de empresa**: Nombre, dirección, teléfono
- **Markup de precios**: Porcentaje de recargo (default 30%)
- **Moneda**: ARS (Peso Argentino), USD (Dólar)
- **Zona horaria**: Buenos Aires, Córdoba, Mendoza
- **Ancho de ticket**: 58mm / 80mm

## 🔐 Sistema de Actualizaciones

- Firmado criptográfico con **ed25519/minisign**
- Verificación de integridad con **SHA-256**
- **Backup automático** de base de datos antes de actualizar
- **Rollback** en caso de fallo de migración

## 📦 Empaquetado

```bash
# Generar AppImage para Linux
cargo tauri build --bundles appimage

# Generar .deb para Debian/Ubuntu
cargo tauri build --bundles deb

# Los paquetes quedan en: target/release/bundle/
```

## 📊 Base de Datos

- **SQLite** con migraciones versionadas
- **FTS5** (Full-Text Search) con eliminación de diacríticos
- **Índices optimizados** para búsquedas por categoría y SKU
- **Timestamps UTC** convertidos a zona horaria local para reportes
- **Aritmética exacta** con `rust_decimal` (sin errores de redondeo)

## 🎯 Próximas Funcionalidades

- [ ] Exportación de reportes a CSV/XLSX
- [ ] Gráficos de ventas mensuales
- [ ] Gestión de inventario (stock)
- [ ] Múltiples usuarios con permisos
- [ ] Integración con AFIP (facturas electrónicas)
- [ ] Modo offline con sincronización

## 🛠️ Tecnologías Utilizadas

- **Backend**: Rust 1.70+ (Tauri v2, SQLite, calamine, rust_decimal)
- **Frontend**: React 18, TypeScript 5, Vite 5
- **Estilos**: CSS custom (paleta B&N, Inter font)
- **Testing**: Cargo test, Criterion (benchmarks)
- **CI/CD**: GitHub Actions (release workflow)

## 📄 Licencia

MIT License - ver archivo `LICENSE` para más detalles.

## 🤝 Contribuciones

Las contribuciones son bienvenidas! Por favor:
1. Fork el repositorio
2. Crea una rama para tu feature (`git checkout -b feature/AmazingFeature`)
3. Commit tus cambios (`git commit -m 'Add some AmazingFeature'`)
4. Push a la rama (`git push origin feature/AmazingFeature`)
5. Abre un Pull Request

## 🔧 Solución de Problemas

### Ventana en blanco en Wayland/Hyprland (GPUs NVIDIA)

Si la aplicación se abre pero muestra una ventana completamente en blanco, especialmente en entornos Wayland como Hyprland con tarjetas NVIDIA, la aplicación aplica automáticamente los siguientes workarounds:

- `WEBKIT_DISABLE_DMABUF_RENDERER=1` - Deshabilita el renderizador DMA-BUF
- `WEBKIT_DISABLE_COMPOSITING_MODE=1` - Deshabilita el modo compositing en Wayland

Estos valores se configuran automáticamente al iniciar la aplicación si no están definidos previamente. Si el problema persiste, puedes exportar manualmente estas variables antes de ejecutar:

```bash
export WEBKIT_DISABLE_DMABUF_RENDERER=1
export WEBKIT_DISABLE_COMPOSITING_MODE=1
./pos-distribuidora
```

### Error Boundary visible

Si ves una pantalla roja con un mensaje de error, la aplicación ha capturado un problema inesperado. Posibles soluciones:

1. Hacer clic en "Recargar aplicación" para reintentar
2. Verificar que la base de datos SQLite no esté corrupta
3. Verificar los permisos de escritura en `~/.local/share/pos-distribuidora/`
4. Revisar los logs en la consola del sistema

### Frontend no se carga

Si tras compilar la aplicación el frontend no se carga, asegúrate de:

1. Haber compilado el frontend antes: `cd crates/pos-desktop/ui && npm run build`
2. Verificar que existe el directorio `crates/pos-desktop/ui/dist/` con los assets JS
3. El `beforeBuildCommand` en `tauri.conf.json` debe estar configurado correctamente

## 📞 Soporte

Para reportar bugs o sugerir mejoras, abre un issue en GitHub.

---

**Desarrollado con ❤️ en Argentina para distribuidoras argentinas.**
