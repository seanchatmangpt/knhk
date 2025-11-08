# Building the Book

To build the KNHK + simdjson mdbook:

```bash
cd docs/knhk-simdjson-book
mdbook build
```

The built book will be in `docs/knhk-simdjson-book-build/`.

## Serving Locally

To serve the book locally for development:

```bash
cd docs/knhk-simdjson-book
mdbook serve
```

The book will be available at `http://localhost:3000`.

## Prerequisites

Install mdbook:

```bash
cargo install mdbook
```

## Structure

The book is organized into:

- `book.toml`: Configuration file
- `src/SUMMARY.md`: Table of contents
- `src/*.md`: Chapter files

## Next Steps

- Read the [Introduction](intro.md)
- Explore [Part I: Foundations](part1/what-is-knhk.md)
- Review [Part IV: Applied Optimizations](part4/80-20-implementation.md)


