# Welcome to the md2md documentation!

This is a simple Markdown to Markdown converter that allows you to use templates to reuse content across multiple Markdown files. It is designed to be easy to use and flexible, allowing you to customize the templates to fit your needs.

## Usage

Specify the input file or directory, the template file or directory, and optionally, the output directory path.

Use the syntax in your input Markdown files to include the content from the specified template files. The template files can be in any format, including Markdown, HTML, or plain text.

## Example

```bash
cargo run -i examples/input_file_referencing_a_template.md -t examples/templates -o output
```

The above command will read the `input_file_referencing_a_template.md` file, find the template syntax below, and replace it line with the contents of the referenced `examples/templates/lorem` file.

{% include "examples/templates/lorem.md" %}

## What happens to surrounding content?

As you can see, nothing breaks!

## ToDo

- [ ] Template path resolution

