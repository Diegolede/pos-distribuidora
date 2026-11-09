# Maintainer: Diego Lede <diego@example.com>
pkgname=pos-distribuidora-bin
pkgver=0.1.0
pkgrel=1
pkgdesc="Sistema POS para distribuidoras - versión binaria"
arch=('x86_64')
url="https://github.com/Diegolede/pos-distribuidora"
license=('MIT')
depends=('webkit2gtk' 'gtk3' 'libappindicator-gtk3')
provides=('pos-distribuidora')
conflicts=('pos-distribuidora')
source=("https://github.com/Diegolede/pos-distribuidora/releases/download/v${pkgver}/pos-distribuidora_${pkgver}_amd64.AppImage"
        "pos-distribuidora.desktop"
        "pos-distribuidora.png")
sha256sums=('SKIP'  # Se actualizará con el hash real de la release
            'SKIP'
            'SKIP')
options=('!strip')

prepare() {
    chmod +x "pos-distribuidora_${pkgver}_amd64.AppImage"
    "./pos-distribuidora_${pkgver}_amd64.AppImage" --appimage-extract > /dev/null 2>&1 || true
}

package() {
    # Instalar el binario
    install -Dm755 "squashfs-root/usr/bin/pos-desktop" "${pkgdir}/usr/bin/pos-distribuidora"
    
    # Copiar bibliotecas necesarias
    if [ -d "squashfs-root/usr/lib" ]; then
        cp -r "squashfs-root/usr/lib/"* "${pkgdir}/usr/lib/" 2>/dev/null || true
    fi
    
    # Instalar icono
    install -Dm644 "${srcdir}/pos-distribuidora.png" \
        "${pkgdir}/usr/share/pixmaps/pos-distribuidora.png"
    
    # Instalar .desktop
    install -Dm644 "${srcdir}/pos-distribuidora.desktop" \
        "${pkgdir}/usr/share/applications/pos-distribuidora.desktop"
    
    # Crear directorio de datos
    install -dm755 "${pkgdir}/usr/share/pos-distribuidora"
}
