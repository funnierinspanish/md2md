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

!codesnippet(../test-code/hello.rs, lang="rust")

## What happens to surrounding content?

As you can see, nothing breaks!
