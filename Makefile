.PHONY build:
build:
    cargo build

.PHONY build_release:
build_release:
    cargo build --release

.PHONY deploy_release:
deploy_release:
    scp target/release/isconklive-rust joeferguson@server.joeferguson.me:"~/bin/isconklive-rust"

