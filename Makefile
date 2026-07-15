# Makefile for ezMerge

PREFIX ?= /usr/local
BINDIR ?= $(PREFIX)/bin
CARGO  ?= cargo
PYTHON ?= python3

.PHONY: all build install uninstall clean test doctor web help

all: build

build:
	@echo "Building ezMerge CLI in release mode..."
	$(CARGO) build --release

install: build
	@echo "Installing ezmerge binary to $(DESTDIR)$(BINDIR)..."
	install -d $(DESTDIR)$(BINDIR)
	install -m 0755 target/release/ezmerge-cli $(DESTDIR)$(BINDIR)/ezmerge
	@echo "ezmerge installed successfully!"

uninstall:
	@echo "Removing ezmerge binary from $(DESTDIR)$(BINDIR)..."
	rm -f $(DESTDIR)$(BINDIR)/ezmerge
	@echo "ezmerge uninstalled successfully."

clean:
	@echo "Cleaning build artifacts..."
	$(CARGO) clean

test:
	@echo "Running tests..."
	$(CARGO) test

doctor:
	@echo "Running ezmerge doctor..."
	@if [ -f target/release/ezmerge-cli ]; then \
		target/release/ezmerge-cli doctor; \
	elif [ -f target/debug/ezmerge-cli ]; then \
		target/debug/ezmerge-cli doctor; \
	else \
		$(CARGO) run --bin ezmerge-cli -- doctor; \
	fi

web:
	@echo "Starting package search portal web server..."
	$(PYTHON) server.py

help:
	@echo "ezMerge Makefile targets:"
	@echo "  build (or all) - Compile the CLI in release mode"
	@echo "  install       - Install the binary to $(BINDIR) (requires root/sudo)"
	@echo "  uninstall     - Remove the binary from $(BINDIR)"
	@echo "  clean         - Clean build target directory"
	@echo "  test          - Run test suite"
	@echo "  doctor        - Run CLI system diagnostics"
	@echo "  web           - Launch the web portal"
