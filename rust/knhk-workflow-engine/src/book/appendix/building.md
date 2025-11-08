# Building the Book

Instructions for building the KNHK Workflow Engine documentation book.

## Prerequisites

Install mdbook:

```bash
cargo install mdbook
```

## Build

Build the book:

```bash
cd rust/knhk-workflow-engine
mdbook build
```

## Serve

Serve the book locally:

```bash
mdbook serve
```

Then open http://localhost:3000 in your browser.

## Watch

Watch for changes and rebuild:

```bash
mdbook watch
```

## Output

The built book is in the `book/` directory.

## Next Steps

- [Contributing](appendix/contributing.md) - Contribute to documentation
- [README](../README.md) - Project README

