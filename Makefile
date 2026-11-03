# Developer shortcuts. Installing doesn't need any of this: see README.md.
.PHONY: build release test lint install uninstall clean

build:            ## release binary in target/release (what ~/.local/bin/rexfeed points to from a checkout)
	cargo build --release

release:          ## release tarball in build/release
	scripts/package-release.sh

test:
	cargo test

lint:
	cargo fmt --check
	cargo clippy --release -- -D warnings
	shellcheck -s sh install.sh uninstall.sh scripts/*.sh

install:
	./install.sh

uninstall:
	./uninstall.sh

clean:
	cargo clean
	rm -rf build
