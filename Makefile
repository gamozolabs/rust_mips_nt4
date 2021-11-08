all:
	cargo build --release
	elfloader target/mipsel-unknown-none/release/mipstest out.felf

objdump: all
	objdump --demangle -d target/mipsel-unknown-none/release/mipstest
