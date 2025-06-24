# Maintainer: instellate
pkgname='pest-editor'
pkgver='0.1.0'
pkgrel=1
pkgdesc='A editor for pest grammar files'
arch=('x86_64' 'aarm64')
url='https://github.com/Instellate/pest-editor'
license=('MIT')
depends=('gdk-pixbuf2' 'glibc' 'gtk3' 'cairo' 'libsoup3' 'gcc-libs' 'glib2' 'pango' 'hicolor-icon-theme' 'webkit2gtk-4.1')
options=('!lto' '!strip' '!emptydirs')
makedepends=('cargo' 'appmenu-gtk-module' 'libappindicator-gtk3' 'librsvg' 'pnpm' 'nodejs' 'git')
source=("git+https://github.com/Instellate/pest-editor.git")
b2sums=('SKIP')

build() {
    cd "${srcdir}/pest-editor"
    pnpm i
    pnpm tauri build -b deb
}

package() {
    cp -r -a -T "${srcdir}/pest-editor/src-tauri/target/release/bundle/deb/Pest Editor_${pkgver}"_*/data ${pkgdir}
    install -Dm644 "${srcdir}/pest-editor/LICENSE" "${pkgdir}/usr/share/licenses/${pkgname}/LICENSE"
}
