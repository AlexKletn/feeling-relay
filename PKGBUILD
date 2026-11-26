pkgbase=feeling-relay
pkgname=('feeling-relay' 'feeling-relay-dkms')
pkgver=0.1.0
pkgrel=1
pkgdesc='Linux bridge for Qualcomm SLPI sensor measurements and events'
arch=('x86_64' 'aarch64')
url='https://github.com/AlexKletn/feeling-relay'
license=('GPL-2.0-only')
makedepends=('rust')
source=("$pkgbase-$pkgver.tar.gz")
sha256sums=('SKIP')

prepare() {
    cd "$srcdir/$pkgbase-$pkgver"
    cargo fetch --locked --target "$CARCH-unknown-linux-gnu"
}

build() {
    cd "$srcdir/$pkgbase-$pkgver"
    cargo build --frozen --release --bin feeling-relay --target-dir "$srcdir/target"
}

check() {
    cd "$srcdir/$pkgbase-$pkgver"
    cargo test --frozen --workspace --target-dir "$srcdir/target"
}

package_feeling-relay() {
    depends=('glibc' 'libgcc')
    optdepends=('systemd: service management' 'feeling-relay-dkms: Linux IIO devices')

    cd "$srcdir/$pkgbase-$pkgver"
    install -Dm755 "$srcdir/target/release/feeling-relay" "$pkgdir/usr/bin/feeling-relay"
    install -Dm644 systemd/feeling-relay.service "$pkgdir/usr/lib/systemd/system/feeling-relay.service"
    install -Dm644 LICENSE "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
    install -Dm644 README.md "$pkgdir/usr/share/doc/$pkgname/README.md"
    install -Dm644 README.ru.md "$pkgdir/usr/share/doc/$pkgname/README.ru.md"
}

package_feeling-relay-dkms() {
    pkgdesc='IIO kernel module for feeling-relay'
    arch=('any')
    depends=('dkms')

    cd "$srcdir/$pkgbase-$pkgver"
    local module_dir="$pkgdir/usr/src/feelrel_iio-$pkgver"
    install -Dm644 -t "$module_dir" kernel/Kbuild kernel/feelrel_iio.c kernel/dkms.conf
    sed -i "s/@VERSION@/$pkgver/g" "$module_dir/dkms.conf"
    install -Dm644 LICENSE "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
}
