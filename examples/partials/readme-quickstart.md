## Quick Start

### Installation

#### Option 1: Download Pre-built Binaries (Recommended)

Download the latest release for your platform from the [GitHub releases page](https://github.com/funnierinspanish/md2md/releases):

**Linux (x86_64):**

```bash
# Download and extract
curl -L https://github.com/funnierinspanish/md2md/releases/latest/download/md2md-x86_64-unknown-linux-gnu.tar.gz | tar xz

# Make executable and move to PATH
chmod +x md2md-x86_64-unknown-linux-gnu
sudo mv md2md-x86_64-unknown-linux-gnu /usr/local/bin/md2md
```

**Linux (x86_64, static musl):**

```bash
# Download and extract (no glibc dependencies)
curl -L https://github.com/funnierinspanish/md2md/releases/latest/download/md2md-x86_64-unknown-linux-musl.tar.gz | tar xz

# Make executable and move to PATH
chmod +x md2md-x86_64-unknown-linux-musl
sudo mv md2md-x86_64-unknown-linux-musl /usr/local/bin/md2md
```

**macOS (Intel):**

```bash
# Download and extract
curl -L https://github.com/funnierinspanish/md2md/releases/latest/download/md2md-x86_64-apple-darwin.tar.gz | tar xz

# Make executable and move to PATH
chmod +x md2md-x86_64-apple-darwin
sudo mv md2md-x86_64-apple-darwin /usr/local/bin/md2md
```

**macOS (Apple Silicon):**

```bash
# Download and extract
curl -L https://github.com/funnierinspanish/md2md/releases/latest/download/md2md-aarch64-apple-darwin.tar.gz | tar xz

# Make executable and move to PATH
chmod +x md2md-aarch64-apple-darwin
sudo mv md2md-aarch64-apple-darwin /usr/local/bin/md2md
```

**Windows:**

1. Download `md2md-x86_64-pc-windows-msvc.zip` from the [releases page](https://github.com/funnierinspanish/md2md/releases)
2. Extract the ZIP file
3. Move `md2md.exe` to a directory in your PATH

#### Option 2: Install from Cargo

```bash
cargo install --git https://github.com/funnierinspanish/md2md.git
```

#### Option 3: Build from Source

```bash
git clone https://github.com/funnierinspanish/md2md.git
cd md2md
cargo build --release
# Binary will be in target/release/md2md
```

### Verify Installation

```bash
md2md --version
```

### Basic Usage

```bash
# Process a single file with partials
md2md input.md -p partials -o output.md

# Include code snippets with syntax highlighting
# Use !codesnippet(file.rs, lang="rust") in your markdown

# Batch process directory
md2md src-docs -p partials -o output-docs --batch

# CI mode with automatic overwrite
md2md src-docs -p partials -o output-docs --batch --ci --force
```
