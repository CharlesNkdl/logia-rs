format:
	cargo fmt --quiet

lint:
	cargo clippy --quiet

docs:
	cargo doc

clean:
	cargo fmt
	cargo clippy

test:
	cargo test --quiet

run:
	cargo run --package logia --bin main --profile dev

release:
	cargo build --release

all: format lint test run