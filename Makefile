BINARY_NAME ?= md2md
BIN_DIR_PATH ?= $(HOME)/.local/bin
BUILD_FLAGS ?= --release

PROFILE := $(if $(findstring --release,$(BUILD_FLAGS)),release,debug)
BIN_PATH     := ./target/$(PROFILE)/$(BINARY_NAME)

.PHONY: all build install reinstall uninstall run watch view pathcheck

all: build

build:
	cargo build $(BUILD_FLAGS)
	@test -x "$(BIN_PATH)" || (echo "Error: couldn't find built binary at $(BIN_PATH). Check PROGRAM name." && exit 1)

install: build
	mkdir -p "$(BIN_DIR_PATH)"
	cp "$(BIN_PATH)" "$(BIN_DIR_PATH)/$(BINARY_NAME)"
	chmod +x "$(BIN_DIR_PATH)/$(BINARY_NAME)"
	@echo "✅ Installed: $(BIN_DIR_PATH)/$(BINARY_NAME)"
	@$(MAKE) -s pathcheck

reinstall: uninstall install

uninstall:
	@rm -f "$(BIN_DIR_PATH)/$(BINARY_NAME)" && echo "🗑️  Removed: $(BIN_DIR_PATH)/$(BINARY_NAME)" || true

run: build
	"$(BIN_PATH)"

# Rebuild on changes and re-copy to BIN_DIR_PATH/BINARY_NAME using cargo-watch (optional)
# Requires: cargo install cargo-watch
watch:
	@command -v cargo-watch >/dev/null 2>&1 || { echo "Install cargo-watch: cargo install cargo-watch"; exit 1; }
	cargo watch -x "build $(BUILD_FLAGS)" -s 'mkdir -p "$(BIN_DIR_PATH)"; cp "$(BIN_PATH)" "$(BIN_DIR_PATH)/$(BINARY_NAME)"; echo "🔁 Updated $(BIN_DIR_PATH)/$(BINARY_NAME)"'

# Quickly watch the installed program's output every second
view:
	@command -v watch >/dev/null 2>&1 || { echo "Install watch (procps-ng)."; exit 1; }
	watch -n 1 "$(BIN_DIR_PATH)/$(BINARY_NAME)"

# Warn if ~/.local/bin isn’t on PATH
pathcheck:
	@echo "$$PATH" | tr ':' '\n' | grep -qx "$(BIN_DIR_PATH)" || \
	  echo "⚠️  Note: $(BIN_DIR_PATH) is not on your PATH. Add this to your shell rc:\n  export PATH=\"$(BIN_DIR_PATH):$$PATH\""
