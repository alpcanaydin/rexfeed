#!/bin/sh
# Build the release tarball: rexfeed binary, install/uninstall scripts, agent hooks
# (scripts/install-hooks.sh, agents/) and slash commands.
# Usage: scripts/package-release.sh [vX.Y.Z]   (prints the tarball path)
set -e
cd "$(dirname "$0")/.."
version=${1:-v$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)}
target=aarch64-apple-darwin
cargo build --release --locked --target "$target" -q
name="rexfeed-$version-$target"
out=build/release
rm -rf "${out:?}/${name:?}" && mkdir -p "$out/$name"
cp "target/$target/release/rexfeed" install.sh uninstall.sh README.md LICENSE "$out/$name/"
cp -R commands agents "$out/$name/"
mkdir -p "$out/$name/scripts" && cp scripts/install-hooks.sh "$out/$name/scripts/"
tar -C "$out" -czf "$out/$name.tar.gz" "$name"
echo "$out/$name.tar.gz"
