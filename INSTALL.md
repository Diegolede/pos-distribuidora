# Guía de Instalación - POS Distribuidora

## Requisitos del Sistema

### Linux (Ubuntu/Debian)

```bash
# Dependencias del sistema requeridas
sudo apt update
sudo apt install -y \
    libwebkit2gtk-4.0-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev \
    patchelf \
    libssl-dev \
    pkg-config

# Rust (si no está instalado)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup update stable

# Node.js 20+ (si no está instalado)
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt install -y nodejs
```

### Arch Linux

```bash
sudo pacman -S webkit2gtk gtk3 libayatana-appindicator librsvg base-devel openssl pkg-config
```

### Fedora

```bash
sudo dnf install webkit2gtk4.0-devel gtk3-devel libappindicator-gtk3-devel librsvg2-devel openssl-devel pkg-config
```

## Compilación

### 1. Clonar el repositorio

```bash
git clone <repo-url>
cd pos-distribuidora
```

### 2. Compilar backend (verificar tests)

```bash
cargo test --workspace --exclude pos-desktop
```

Debe mostrar: **19/19 tests pasando**.

### 3. Compilar frontend

```bash
cd crates/pos-desktop/ui
npm install
npm run build
cd ../../..
```

### 4. Compilar aplicación completa

```bash
cargo build --release -p pos-desktop
```

El ejecutable estará en: `target/release/pos-desktop`

## Ejecución

### Modo desarrollo

```bash
# Terminal 1: Frontend dev server (con hot reload)
cd crates/pos-desktop/ui
npm run dev

# Terminal 2: Backend Tauri
cd crates/pos-desktop
cargo run
```

### Modo producción

```bash
./target/release/pos-desktop
```

## Ubicación de Datos

- **Linux:** `~/.local/share/pos/pos.db`
- **Tickets guardados:** `~/.local/share/pos/tickets/`

Los datos persisten entre actualizaciones de la aplicación.

## Respaldo y Restauración

### Respaldo manual

```bash
# Copiar la base de datos
cp ~/.local/share/pos/pos.db ~/respaldo-pos-$(date +%Y%m%d).db

# O usar SQLite dump
sqlite3 ~/.local/share/pos/pos.db .dump > ~/respaldo-pos-$(date +%Y%m%d).sql
```

### Restaurar desde respaldo

```bash
# Desde archivo .db
cp ~/respaldo-pos-YYYYMMDD.db ~/.local/share/pos/pos.db

# Desde dump SQL
sqlite3 ~/.local/share/pos/pos.db < ~/respaldo-pos-YYYYMMDD.sql
```

## Solución de Problemas

### Error: "cannot find -lwebkit2gtk"

Falta instalar `libwebkit2gtk-4.0-dev`:
```bash
sudo apt install libwebkit2gtk-4.0-dev
```

### Error: "no such file or directory: pos.db"

La aplicación crea la BD automáticamente en el primer arranque. Si no se crea, verificar permisos en `~/.local/share/pos/`.

### La aplicación no inicia en Wayland

Forzar X11:
```bash
GDK_BACKEND=x11 ./target/release/pos-desktop
```

### Importación de Excel falla

Verificar que el archivo tiene:
- Al menos una hoja con datos
- Encabezados en la primera fila (Nombre, Precio Profesional, Precio Público, etc.)
- Filas con al menos un nombre Y un precio (profesional O público)

Filas completamente vacías se ignoran automáticamente.

## Generar Datos de Prueba

Para testing con 5,000 productos:

1. Abrir la aplicación
2. Click en "Generar Prueba" (botón superior derecho)
3. Confirmar

Los productos generados tienen SKU con prefijo `TEST-` para identificarlos.

## Desinstalación

```bash
# Eliminar ejecutable
rm /usr/local/bin/pos-desktop  # o donde lo hayas instalado

# Eliminar datos (¡cuidado, esto borra todo!)
rm -rf ~/.local/share/pos/
```

## Notas de Seguridad

- La base de datos SQLite **no está encriptada**
- Los tickets **no son comprobantes fiscales válidos**
- Para uso fiscal, se requiere módulo de facturación certificado (futuro)
- Hacer respaldos regulares antes de actualizaciones

## Soporte

Ver `README.md` para documentación completa y arquitectura del proyecto.
