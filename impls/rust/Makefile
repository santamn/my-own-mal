all: build

step0:
	cargo build --release --bin step0_repl

step1:
	cargo build --release --bin step1_read_print

step2:
	cargo build --release --bin step2_eval

step3:
	cargo build --release --bin step3_env

build:
	cargo build --release

.PHONY: clean
clean:
	cargo clean