# —————————————————————————————
#   PROJECT-WIDE MAKE TARGETS
# —————————————————————————————

.PHONY: all clean build run ext1 test coverage docs

# Default: clean & build
all: clean build

# Build release binary
build:
	cargo build --release

# Wipe out target/
clean:
	cargo clean
	@rm -f tarpaulin-report.html
# CLI run: rows × cols
run: build
	cargo run --release -- 999 18278

# GUI run (your embedded::ui)
ext1: build
	cargo run --release -- ext1

# Run all Rust tests
test:
	cargo test --release

# Code‐coverage (Tarpaulin)
coverage:
	cargo tarpaulin --skip-clean --include-files src/commands.rs src/function.rs src/myparser.rs src/spreadsheet.rs --out Html

# Generate HTML docs + PDF
docs:
	cargo doc --no-deps --all-features --document-private-items
	@if [ -f target/doc/embedded/index.html ]; then \
		echo "Rust HTML documentation generated successfully."; \
	else \
		echo "Rust documentation failed."; \
		exit 1; \
	fi
	pdflatex main.tex
