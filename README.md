# md2md

A Markdown to Markdown processor that supports include directives and code snippet inclusion for reusable content composition... A templating system for Markdown that outputs to Markdown.

## Overview

md2md allows you to compose documents from reusable markdown partials and code snippets using include directives. This enables you to:

- **Reuse content** across multiple documents
- **Include code files** with syntax highlighting and line selection
- **Maintain consistency** in documentation
- **Organize content** into modular, reusable pieces
- **Process files individually** or in batch mode

## Features

- **Include directives** - Compose documents from reusable partials
- **Code snippet inclusion** - Include code files with syntax highlighting
- **Batch processing** - Process entire directories at once
- **Interactive TUI** - Beautiful terminal interface for monitoring progress
- **CI/automation mode** - Non-interactive processing for pipelines
- **Force mode** - Automatic overwrite and directory creation
- **Detailed reporting** - Comprehensive processing statistics
- **Input/Output validation** - Enforces consistent file/directory types

## Two Main Directive Types

- **`!include()`** - Include markdown partials with optional variables and titles
- **`!codesnippet()`** - Include code files with syntax highlighting and line selection

## Quick Start

### Installation

#### Option 1: Install from Cargo

```bash
cargo install --git https://github.com/funnierinspanish/md2md.git
```

#### Option 2: Download Pre-built Binaries

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


## Directive Syntax

md2md supports two types of directives for content inclusion:

### Include Directives

Use include directives to include markdown partials:

#### Basic Include

```markdown
!include(your-partial.md)
```

#### Include with Title

Add an automatic title to the included content:

```markdown
!include(your-partial.md, title="Section Title")
!include(your-partial.md, title="Section Title", title-level=2)
```

#### Include with Variables

Pass variables to be substituted in the included content:

```markdown
!include(your-partial.md, values=[variable_name="Value", another_var="Another Value"])
```

#### Combined Usage

```markdown
!include(your-partial.md, title="Getting Started", title-level=2, values=[project_name="MyProject", author="John Doe"])
```

#### Variable Syntax in Partials

Within your partial files, use this syntax for variables:

```markdown
# Welcome to {% project_name %}!

Created by: {% author %}

Optional with default: {% optional_var || "default value" %}
```

### Code Snippet Directives

Use codesnippet directives to include code files with syntax highlighting:

#### Basic Code Inclusion

```markdown
!codesnippet(src/main.rs, lang="rust")
```

#### Line Range Selection

```markdown
!codesnippet(utils.py, lang="python", start=10, end=25)
!codesnippet(config.js, lang="javascript", end=15)
!codesnippet(helpers.py, lang="python", start=20)
```

#### Without Language

```markdown
!codesnippet(data.txt)
```

### Path Resolution

Both include and codesnippet directives follow the same path resolution rules:

1. **Partials directory** - Plain filenames are resolved relative to the partials directory (`-p` flag)
2. **Relative paths** - Paths starting with `../` are resolved relative to the current file
3. **Absolute paths** - Paths starting with `/` are used as-is

## Example

Given this file structure:

```text
examples/
├── source-documents/
│   ├── api.md
│   └── mixed-demo.md
├── partials/
│   ├── header.md
│   └── footer.md
└── test-code/
    ├── hello.rs
    └── example.py
```

Your `docs/api.md` can include partials and code:

```markdown
!include(header.md)

# API Documentation

!codesnippet(../test-code/hello.rs, lang="rust")

Your main content here...

!include(footer.md)
```

Process with:

```bash
md2md docs/api.md -p partials -o output/api.md
```

Yielding:

```rust
fn main() {
    println!("Hello, world!");
    let x = 42;
    let y = x * 2;
    println!("Result: {}", y);

    if x > 0 {
        println!("x is positive");
    }
}
```

## What happens to surrounding content?

As you can see, nothing breaks!

## CLI Reference

```bash
md2md [OPTIONS] <INPUT_PATH>

Arguments:
  <INPUT_PATH>  The source file or directory to be processed

Options:
  -p, --partials-path <PARTIALS>    The directory containing the partials. Default: `partials` [default: partials]
  -o, --output-path <OUTPUT>        Output path (file or directory) [default: out]
  -b, --batch                       Process directories recursively (batch mode)
  -v, --verbose                     Verbose output
  -c, --ci                          Disable TUI interface (use simple console output)
  -f, --force                       Force overwrite existing files and create directories without prompting
      --fix-code-fences <LANGUAGE>  Fix code fences that don't specify a language by adding a default language [default: text]
  -h, --help                        Print help (see more with '--help')
  -V, --version                     Print version
```

## Input/Output Validation

md2md enforces consistent input/output types:

- **File input** → **File output**: `input.md` → `output.md`
- **Directory input** → **Directory output**: `src-docs/` → `output-docs/`
- Use trailing slash (`/`) to explicitly indicate directory output
- Files without extensions are allowed as output for file input

## Validation Examples

```bash
# ✅ Valid: File → File
md2md input.md -p partials -o output.md

# ✅ Valid: Directory → Directory
md2md src-docs -p partials -o output-docs --batch

# ✅ Valid: Directory → Directory (explicit)
md2md src-docs -p partials -o output-docs/ --batch

# ❌ Invalid: File → Directory
md2md input.md -p partials -o output-dir/

# ❌ Invalid: Directory → File
md2md src-docs -p partials -o output.md --batch
```

## Examples

See the `examples/` directory for demonstrations of md2md features:

```bash
# Process include examples
md2md examples/source-documents/demo.md -p examples/partials -o output/demo.md

# Process code snippet examples
md2md examples/source-documents/codesnippet-demo.md -p examples/partials -o output/codesnippet-demo.md

# Process mixed directives
md2md examples/source-documents/mixed-directives.md -p examples/partials -o output/mixed.md

# Batch process directory
md2md examples/source-documents -p examples/partials -o output-docs --batch
```

## Architecture

md2md processes files through these steps:

1. **Parse** input files for include and codesnippet directives
2. **Resolve** partial and code file paths according to resolution rules
3. **Include** partial content and code snippets recursively (supports nested includes)
4. **Process** variable substitution in partials
5. **Write** processed output to destination

The tool supports both single-file processing and batch directory processing with comprehensive error handling and progress reporting.

## Terminology Guide

### Partials vs Templates

**md2md** uses the term "partials" to describe reusable content pieces, although the CLI flag remains `--templates-path` for backward compatibility.

### Partials

- **Definition**: Reusable pieces of content that can be included in multiple documents
- **Examples**: Headers, footers, common sections, shared content blocks
- **Usage**: Include directives like `!include (header.md)` pull in the content of partials

### Templates (in broader context)

- **Definition**: Structural layouts that define the overall format of documents
- **Examples**: HTML page layouts, document skeletons with placeholders
- **md2md approach**: We compose documents from partials rather than filling template placeholders

## Why "Partials"?

The content pieces in md2md are:

1. **Self-contained** - Complete markdown content that can stand alone
1. **Reusable** - Used across multiple documents
1. **Composable** - Combined to build larger documents
1. **Content-focused** - Contain actual content rather than structural placeholders

This aligns with the "partial" concept used in many templating systems where partials are reusable content components.

## CLI Compatibility

The CLI flag remains `--templates-path` for backward compatibility, but conceptually these are partials directories containing reusable content pieces.
