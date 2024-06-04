.PHONY build:
build:
	cargo build

.PHONY build_release:
build_release:
	cargo build --release

.PHONY deploy_release:
deploy_release:
	cp target/release/isconklive-rust ~/bin/isconklive-rust

