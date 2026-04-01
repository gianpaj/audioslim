APP_NAME   := audioslim
CARGO      := cargo
DIST_DIR   := dist

# ── Host detection ────────────────────────────────────────────────────────────
# Detect whether cross-compilation tooling is available for the current host.
# Override any variable on the command line, e.g.:
#   make build-windows CROSS=cross

CROSS ?= cross

# ── Default target ────────────────────────────────────────────────────────────

.PHONY: all
all: build

# ── Native build (current host) ───────────────────────────────────────────────

.PHONY: build
build:
	$(CARGO) build --release
	@echo "Binary: target/release/$(APP_NAME)"

# ── macOS targets ─────────────────────────────────────────────────────────────

.PHONY: build-macos-arm
build-macos-arm:
	$(CARGO) build --release --target aarch64-apple-darwin
	@mkdir -p $(DIST_DIR)
	cp target/aarch64-apple-darwin/release/$(APP_NAME) \
	   $(DIST_DIR)/$(APP_NAME)-macos-arm64

.PHONY: build-macos-intel
build-macos-intel:
	$(CARGO) build --release --target x86_64-apple-darwin
	@mkdir -p $(DIST_DIR)
	cp target/x86_64-apple-darwin/release/$(APP_NAME) \
	   $(DIST_DIR)/$(APP_NAME)-macos-x86_64

# Universal binary (requires both targets above to already be built)
.PHONY: build-macos-universal
build-macos-universal: build-macos-arm build-macos-intel
	@mkdir -p $(DIST_DIR)
	lipo -create \
	     target/aarch64-apple-darwin/release/$(APP_NAME) \
	     target/x86_64-apple-darwin/release/$(APP_NAME) \
	     -output $(DIST_DIR)/$(APP_NAME)-macos-universal
	@echo "Universal binary: $(DIST_DIR)/$(APP_NAME)-macos-universal"

# ── Linux targets ─────────────────────────────────────────────────────────────
# Requires the target to be installed:
#   rustup target add x86_64-unknown-linux-gnu
#   rustup target add aarch64-unknown-linux-gnu
# When cross-compiling from macOS/Windows, install 'cross':
#   cargo install cross

.PHONY: build-linux-x86
build-linux-x86:
	$(CROSS) build --release --target x86_64-unknown-linux-gnu
	@mkdir -p $(DIST_DIR)
	cp target/x86_64-unknown-linux-gnu/release/$(APP_NAME) \
	   $(DIST_DIR)/$(APP_NAME)-linux-x86_64

.PHONY: build-linux-arm
build-linux-arm:
	$(CROSS) build --release --target aarch64-unknown-linux-gnu
	@mkdir -p $(DIST_DIR)
	cp target/aarch64-unknown-linux-gnu/release/$(APP_NAME) \
	   $(DIST_DIR)/$(APP_NAME)-linux-arm64

# ── Windows target ────────────────────────────────────────────────────────────
# Requires the target to be installed:
#   rustup target add x86_64-pc-windows-gnu
# When cross-compiling from macOS/Linux, install 'cross':
#   cargo install cross

.PHONY: build-windows
build-windows:
	$(CROSS) build --release --target x86_64-pc-windows-gnu
	@mkdir -p $(DIST_DIR)
	cp target/x86_64-pc-windows-gnu/release/$(APP_NAME).exe \
	   $(DIST_DIR)/$(APP_NAME)-windows-x86_64.exe

# ── Convenience group targets ─────────────────────────────────────────────────

.PHONY: build-macos
build-macos: build-macos-universal

.PHONY: build-all
build-all: build-macos-universal build-linux-x86 build-linux-arm build-windows
	@echo "All binaries written to $(DIST_DIR)/"
	@ls -lh $(DIST_DIR)/

# ── macOS .app bundle ─────────────────────────────────────────────────────────
# Produces dist/AudioSlim.app for the current host architecture.
# Run 'make bundle-macos-arm' or 'bundle-macos-intel' for a specific arch.

APP_BUNDLE   := $(DIST_DIR)/AudioSlim.app
BUNDLE_MACOS := $(APP_BUNDLE)/Contents/MacOS
BUNDLE_RES   := $(APP_BUNDLE)/Contents/Resources

# Build the .icns file from the repo's iconset (macOS only)
ICNS_SRC := ../icon.iconset
ICNS_OUT := $(DIST_DIR)/AppIcon.icns

$(ICNS_OUT):
	@mkdir -p $(DIST_DIR)
	iconutil -c icns $(ICNS_SRC) -o $(ICNS_OUT)

.PHONY: _copy-icon
_copy-icon: $(ICNS_OUT)
	@if [ -f $(ICNS_OUT) ]; then \
	    cp $(ICNS_OUT) $(BUNDLE_RES)/AppIcon.icns; \
	fi

.PHONY: bundle-macos
bundle-macos: build _copy-icon
	@rm -rf $(APP_BUNDLE)
	@mkdir -p $(BUNDLE_MACOS) $(BUNDLE_RES)/bin
	cp target/release/$(APP_NAME) $(BUNDLE_MACOS)/$(APP_NAME)
	@if [ -f ../vendors/ffmpeg/macos-arm64/ffmpeg ]; then \
	    cp ../vendors/ffmpeg/macos-arm64/ffmpeg $(BUNDLE_RES)/bin/ffmpeg; \
	    chmod +x $(BUNDLE_RES)/bin/ffmpeg; \
	fi
	$(MAKE) _write-plist _copy-icon
	@echo "App bundle: $(APP_BUNDLE)"

.PHONY: bundle-macos-arm
bundle-macos-arm: build-macos-arm
	@rm -rf $(APP_BUNDLE)
	@mkdir -p $(BUNDLE_MACOS) $(BUNDLE_RES)/bin
	cp target/aarch64-apple-darwin/release/$(APP_NAME) $(BUNDLE_MACOS)/$(APP_NAME)
	@if [ -f ../vendors/ffmpeg/macos-arm64/ffmpeg ]; then \
	    cp ../vendors/ffmpeg/macos-arm64/ffmpeg $(BUNDLE_RES)/bin/ffmpeg; \
	    chmod +x $(BUNDLE_RES)/bin/ffmpeg; \
	fi
	$(MAKE) _write-plist _copy-icon
	@echo "App bundle (arm64): $(APP_BUNDLE)"

.PHONY: bundle-macos-intel
bundle-macos-intel: build-macos-intel
	@rm -rf $(APP_BUNDLE)
	@mkdir -p $(BUNDLE_MACOS) $(BUNDLE_RES)/bin
	cp target/x86_64-apple-darwin/release/$(APP_NAME) $(BUNDLE_MACOS)/$(APP_NAME)
	$(MAKE) _write-plist _copy-icon
	@echo "App bundle (x86_64): $(APP_BUNDLE)"

.PHONY: _write-plist
_write-plist:
	@printf '<?xml version="1.0" encoding="UTF-8"?>\n\
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"\n\
    "http://www.apple.com/DTDs/PropertyList-1.0.dtd">\n\
<plist version="1.0">\n<dict>\n\
    <key>CFBundleName</key>              <string>AudioSlim</string>\n\
    <key>CFBundleIdentifier</key>        <string>com.audioslim.app</string>\n\
    <key>CFBundleVersion</key>           <string>0.2.0</string>\n\
    <key>CFBundleShortVersionString</key><string>0.2.0</string>\n\
    <key>CFBundlePackageType</key>       <string>APPL</string>\n\
    <key>CFBundleExecutable</key>        <string>$(APP_NAME)</string>\n\
    <key>CFBundleIconFile</key>          <string>AppIcon</string>\n\
    <key>NSHighResolutionCapable</key>   <true/>\n\
    <key>LSMinimumSystemVersion</key>    <string>12.0</string>\n\
    <key>NSPrincipalClass</key>          <string>NSApplication</string>\n\
</dict>\n</plist>\n' \
	> $(APP_BUNDLE)/Contents/Info.plist

# ── Development ───────────────────────────────────────────────────────────────

.PHONY: run
run:
	$(CARGO) run

.PHONY: check
check:
	$(CARGO) check

.PHONY: clippy
clippy:
	$(CARGO) clippy -- -D warnings

.PHONY: fmt
fmt:
	$(CARGO) fmt

# ── Utilities ─────────────────────────────────────────────────────────────────

.PHONY: add-targets
add-targets:
	rustup target add \
	    aarch64-apple-darwin \
	    x86_64-apple-darwin \
	    x86_64-unknown-linux-gnu \
	    aarch64-unknown-linux-gnu \
	    x86_64-pc-windows-gnu

.PHONY: clean
clean:
	$(CARGO) clean
	rm -rf $(DIST_DIR)

.PHONY: help
help:
	@echo "AudioSlim – Freya desktop app build targets"
	@echo ""
	@echo "  make build                  Native release build"
	@echo "  make run                    Run in dev mode"
	@echo ""
	@echo "  make build-macos-arm        macOS arm64 (Apple Silicon)"
	@echo "  make build-macos-intel      macOS x86_64 (Intel)"
	@echo "  make build-macos-universal  macOS universal binary (lipo)"
	@echo ""
	@echo "  make build-linux-x86        Linux x86_64 (uses cross)"
	@echo "  make build-linux-arm        Linux arm64  (uses cross)"
	@echo "  make build-windows          Windows x86_64 (uses cross)"
	@echo ""
	@echo "  make build-all              All platforms → dist/"
	@echo ""
	@echo "  make bundle-macos           .app bundle (native arch)"
	@echo "  make bundle-macos-arm       .app bundle (arm64)"
	@echo "  make bundle-macos-intel     .app bundle (x86_64)"
	@echo ""
	@echo "  make add-targets            rustup add all cross-compile targets"
	@echo "  make clean                  Remove build artifacts and dist/"
