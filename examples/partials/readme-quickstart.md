## Quick Start

### Installation

```bash
cargo build --release
```

### Basic Usage

```bash
# Process a single file with partials
md2md input.md -p partials -o output.md

# Batch process directory
md2md src-docs -p partials -o output-docs --batch

# CI mode with automatic overwrite
md2md src-docs -p partials -o output-docs --batch --ci --force
```
