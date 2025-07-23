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
