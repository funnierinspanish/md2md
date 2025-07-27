## Quick Start

### Installation

#### Direct Install from GitHub (Recommended)

```bash
cargo install --git https://github.com/funnierinspanish/md2md.git
```

#### From Source

```bash
git clone https://github.com/funnierinspanish/md2md.git
cd md2md
cargo build --release
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