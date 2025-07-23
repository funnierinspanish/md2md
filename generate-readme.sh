#!/bin/bash

# Generate README.md using md2md itself
# This demonstrates how md2md can be used to maintain its own documentation

echo "🔄 Generating README.md from partials..."

# Build the md2md binary

cargo build --release

# Generate the README

./target/release/md2md examples/source-documents/README-base.md -p examples/partials -o README.md --ci --verbose --force
