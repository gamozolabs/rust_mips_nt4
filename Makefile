all:
	cargo build --release
	elfloader target/mipsel-unknown-none/release/mipstest out.felf
	nc -w 0 127.0.0.1 5555

clippy:
	cargo clippy -- -F clippy::missing_docs_in_private_items

objdump: all
	objdump --demangle -d target/mipsel-unknown-none/release/mipstest
