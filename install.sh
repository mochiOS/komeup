#!/bin/sh

set -eu

version="${KOME_SDK_VERSION:-27.0-dp.1}"
case "$(uname -s)" in
    Linux) ;;
    *) echo "komeup: this installer currently supports Linux only" >&2; exit 1 ;;
esac
case "$(uname -m)" in
    x86_64) arch=x86_64 ;;
    aarch64|arm64) arch=aarch64 ;;
    *) echo "komeup: unsupported architecture: $(uname -m)" >&2; exit 1 ;;
esac

asset="$arch-komeup-$version.tar.zst"
base="https://github.com/mochiOS/komeup/releases/download/$version"
temporary="$(mktemp -d)"
trap 'rm -rf "$temporary"' EXIT INT TERM

curl -fsSL "$base/$asset" -o "$temporary/$asset"
curl -fsSL "$base/SHA256SUMS" -o "$temporary/SHA256SUMS"
(
    cd "$temporary"
    grep "  $asset\$" SHA256SUMS | sha256sum -c -
    tar --zstd -xf "$asset"
)
"$temporary/komeup" install "$version"

printf '%s\n' "Kome SDK $version was installed."
printf '%s\n' 'Add $HOME/.kome/bin to PATH if it is not already present.'
