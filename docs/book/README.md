# KNHK mdbook

This is the mdbook (Markdown book) for KNHK documentation.

## Building

```bash
cd docs/book
mdbook build
```

The built book will be in `docs/book/book/` directory.

## Serving

To serve the book locally for preview:

```bash
cd docs/book
mdbook serve
```

Then open http://localhost:3000 in your browser.

## Structure

The book is organized into sections:

- **Getting Started**: Installation, building, testing
- **Architecture**: System design and components
- **API Reference**: C, Rust, and Erlang APIs
- **Integration**: Integration guides
- **Development**: Development guides and best practices
- **Reference**: Detailed reference documentation
- **Project Management**: Status, DoD, policies
- **Appendices**: Changelog, release notes, indexes

## Content Organization

The book follows the 80/20 principle:
- Links to canonical documentation files
- Organized by topic for easy navigation
- Cross-references between sections

## Updating

To update the book:
1. Edit files in `src/` directory
2. Update `src/SUMMARY.md` if adding new sections
3. Run `mdbook build` to rebuild
4. Run `mdbook serve` to preview changes
