## Include Syntax

Use the following syntax in your markdown files to include partials:

```
!include (filename.md)
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
!include (document-header.md)

# API Documentation

Your main content here...

!include (document-footer.md)
```

Process with:

```bash
md2md docs/api.md -p partials -o output/api.md
```

## What happens to surrounding content?

As you can see, nothing breaks!
