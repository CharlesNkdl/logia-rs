BINARY     := logia-rs
INSTALL_AS := logia
INSTALL_DIR := $(HOME)/.local/bin

TARGETS := \
	aarch64-apple-darwin


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
	cargo run --profile dev

all: format lint test run

release:
	cargo build --release

install: release
	@mkdir -p $(INSTALL_DIR)
	@cp target/release/$(BINARY) $(INSTALL_DIR)/$(INSTALL_AS)
	@chmod +x $(INSTALL_DIR)/$(INSTALL_AS)
	@echo "Installed $(INSTALL_AS) -> $(INSTALL_DIR)/$(INSTALL_AS)"
	@if ! echo "$$PATH" | tr ':' '\n' | grep -qx "$(INSTALL_DIR)"; then \
		echo ""; \
		echo "  Note: $(INSTALL_DIR) is not in your PATH."; \
		echo "  Add to your ~/.zshrc:  export PATH=\"\$$HOME/.local/bin:\$$PATH\""; \
	fi

uninstall:
	@rm -f $(INSTALL_DIR)/$(INSTALL_AS)
	@echo "Removed $(INSTALL_DIR)/$(INSTALL_AS)"

.PHONY: format lint docs clean test run all release install uninstall
