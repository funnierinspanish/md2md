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
