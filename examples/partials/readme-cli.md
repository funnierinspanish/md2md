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