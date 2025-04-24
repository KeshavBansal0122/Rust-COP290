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

# CLI run: rows × cols
run: build
	cargo run --release -- 999 18278

# GUI run (your embedded::ui)
ext1: build
	cargo run --release -- ext1

# Run all Rust tests
test:
	cargo test

# Code‐coverage (Tarpaulin)
coverage:
	cargo tarpaulin --out Html

# Generate HTML docs + PDF
docs:
	cargo doc --no-deps
	# convert the HTML index to PDF (requires wkhtmltopdf or pandoc)
	@if [ -f target/doc/$(shell basename `pwd`)/index.html ]; then \
	  wkhtmltopdf \
	    target/doc/$(shell basename `pwd`)/index.html \
	    target/doc/$(shell basename `pwd`).pdf; \
	else \
	  echo "Cannot find HTML docs; did 'cargo doc' run successfully?"; \
	  exit 1; \
	fi
