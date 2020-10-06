# usage
usage:
	@just -l

# prepare
prepare:
	@which sccache 2>&1 > /dev/null || cargo install sccache
	@cargo watch --help > /dev/null 2>&1 || cargo install cargo-watch
	@cargo audit --help > /dev/null 2>&1 || cargo install cargo-audit

# run
run: prepare
	cargo run

sccache_path := `which sccache`
# build
build: prepare
	RUSTC_WRAPPER={{sccache_path}} cargo build

# format
fmt: prepare
	@cargo fmt

# lint
lint: fmt
	@cargo clippy

# audit
audit: lint
	@cargo audit

# test
test: prepare
	cargo test

# watch
watch +COMMAND='test': prepare
	cargo watch --clear --exec "{{COMMAND}}"

# clean
clean:
	@rm -rf examples
	@cargo clean

# vim: set noexpandtab :
