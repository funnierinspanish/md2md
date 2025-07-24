## Examples

See the `examples/` directory for comprehensive demonstrations of md2md features:

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
