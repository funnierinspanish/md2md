# md2md

A powerful Markdown processor that supports include directives for reusable content composition.

## Overview

md2md allows you to compose documents from reusable markdown partials using include directives. This enables you to:

- **Reuse content** across multiple documents
- **Maintain consistency** in documentation
- **Organize content** into modular, reusable pieces
- **Process files individually** or in batch mode

## Features

- 🔗 **Include directives** - Compose documents from reusable partials
- 📁 **Batch processing** - Process entire directories at once  
- 🖥️  **Interactive TUI** - Beautiful terminal interface for monitoring progress
- 🤖 **CI/automation mode** - Non-interactive processing for pipelines
- 🛡️ **Force mode** - Automatic overwrite and directory creation
- 📊 **Detailed reporting** - Comprehensive processing statistics
- ✅ **Input/Output validation** - Enforces consistent file/directory types

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

## Include Syntax

Use the following syntax in your markdown files to include partials:

### Basic Include

```markdown
!include (partial-file.md)
```

### Include with Title

Add an automatic title to the included content:

```markdown
!include (partial-file.md, title="Section Title")
!include (partial-file.md, title="Section Title", title-level=2)
```

### Include with Variables

Pass variables to be substituted in the included content:

```markdown
!include (partial-file.md, values=[variable_name="Value", another_var="Another Value"])
```

### Combined Usage

```markdown
!include (partial-file.md, title="Getting Started", title-level=2, values=[project_name="MyProject", author="John Doe"])
```

### Variable Syntax in Partials

Within your partial files, use this syntax for variables:

```markdown
# Welcome to {% project_name %}!

Created by: {% author %}

Optional with default: {% optional_var || "default value" %}
```

### Path Resolution

1. **Partials directory** - Plain filenames are resolved relative to the partials directory (`-p` flag)
1. **Relative paths** - Paths starting with `../` are resolved relative to the current file
1. **Absolute paths** - Paths starting with `/` are used as-is

## Example

Given this file structure:

```text
examples/
├── source-documents/
│   └── ...
├── partials/
│   ├── ...
│   └── ...
```

Your `docs/api.md` can include partials:

```
Common header content here

# API Documentation

Your main content here...

Common footer content here
```

Process with:

```bash
md2md docs/api.md -p partials -o output/api.md
```

## What happens to surrounding content?

As you can see, nothing breaks!

## CLI Reference

```bash
md2md [OPTIONS] <INPUT_PATH>

Arguments:
  <INPUT_PATH>  The source file or directory to be processed

Options:
  -p, --partials-path <PARTIALS>    The directory containing partials [default: partials]
  -o, --output-path <OUTPUT>        Output path (file or directory) [default: out]
  -b, --batch                       Process directories recursively (batch mode)
  -v, --verbose                     Verbose output
      --ci                          Disable TUI interface (use simple console output)
  -f, --force                       Force overwrite existing files and create directories
  -h, --help                        Print help
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

See the `examples/` directory for comprehensive demonstrations of md2md features:

```bash
# Run all examples
./examples/run-examples.sh

# Individual examples
md2md examples/basic-include.md -p examples/partials -o output.md
md2md examples/documentation -p examples/partials -o output-docs --batch
```

## Architecture

md2md processes files through these steps:

1. **Parse** input files for include directives
1. **Resolve** partial paths according to resolution rules  
1. **Include** partial content recursively (supports nested includes)
1. **Write** processed output to destination

The tool supports both single-file processing and batch directory processing with comprehensive error handling and progress reporting.

## Terminology Guide

### Partials vs Templates

**md2md** uses the term "partials" to describe reusable content pieces, although the CLI flag remains `--templates-path` for backward compatibility.

### Partials

- **Definition**: Reusable pieces of content that can be included in multiple documents
- **Examples**: Headers, footers, common sections, shared content blocks
- **Usage**: Include directives like `# Header

Example header content.` pull in the content of partials

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
