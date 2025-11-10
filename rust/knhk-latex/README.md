# knhk-latex

A CLI tool for compiling and managing LaTeX documents using clap-noun-verb v3.4.0 and Chicago TDD testing.

## Features

- **Compile LaTeX to PDF**: Supports Tectonic, pdflatex, and xelatex
- **Mermaid Diagram Support**: Automatically converts Mermaid diagrams to SVG/PNG for LaTeX inclusion
- **Syntax Checking**: Check LaTeX syntax without full compilation
- **Structure Validation**: Validate LaTeX document structure
- **Clean Auxiliary Files**: Remove auxiliary files generated during compilation
- **Compiler Information**: List available LaTeX compilers

## Installation

```bash
cd rust
cargo build --package knhk-latex --release
cargo install --path knhk-latex
```

## Usage

The CLI follows the noun-verb pattern:

```bash
knhk-latex <noun> <verb> [args]
```

### Commands

#### Compile LaTeX to PDF

```bash
knhk-latex latex compile <source.tex> [--output-dir <dir>] [--compiler <compiler>]
```

Examples:
```bash
# Auto-detect compiler
knhk-latex latex compile paper.tex

# Specify output directory
knhk-latex latex compile paper.tex --output-dir ./output

# Use specific compiler
knhk-latex latex compile paper.tex --compiler pdflatex
```

#### Check LaTeX Syntax

```bash
knhk-latex latex check <source.tex>
```

#### Validate LaTeX Structure

```bash
knhk-latex latex validate <source.tex>
```

#### Clean Auxiliary Files

```bash
knhk-latex latex clean <source.tex> [--keep-pdf]
```

#### Show Compiler Information

```bash
knhk-latex latex info
```

## Mermaid Diagram Support

The CLI automatically processes Mermaid diagrams during LaTeX compilation. It supports:

### Inline Mermaid Blocks

```latex
\begin{mermaid}
graph TD;
    A[Rust Code] --> B(Generate mmd files);
    B --> C{Run mermaid-cli};
    C --> D[Output SVG/PNG];
\end{mermaid}
```

### External Mermaid Files

Reference standalone `.mmd` or `.mermaid` files:

```latex
\includemermaid{diagram.mmd}
```

Or:

```latex
\inputmermaid{diagram.mmd}
```

Or:

```latex
\mermaidfile{diagram.mmd}
```

### Standalone Mermaid Files

Standalone `.mmd` or `.mermaid` files in the same directory as your LaTeX file are automatically detected and converted. They can be referenced in your LaTeX document using the commands above.

### Requirements

To use Mermaid diagram support, you need to have one of the following installed:

- **mermaid-cli** (recommended): `npm install -g @mermaid-js/mermaid-cli`
- **npx**: Automatically downloads and uses mermaid-cli if available

The CLI will automatically:
1. Detect Mermaid diagrams in your LaTeX file
2. Convert them to SVG (preferred) or PNG format
3. Include them in your LaTeX document using appropriate figure environments
4. Handle errors gracefully if Mermaid CLI is not available

### Output

Mermaid diagrams are converted to images in the `mermaid/` subdirectory of your output directory. The LaTeX file is automatically updated to include these images using `\includegraphics` or `\includesvg` commands.

## Output Format

All commands output JSON for easy integration with tools and scripts:

```json
{
  "source": "paper.tex",
  "output": "paper.pdf",
  "success": true,
  "pages": 9,
  "size_bytes": 322031
}
```

## Testing

Tests use Chicago TDD methodology:

```bash
cargo test --package knhk-latex
```

## Architecture

- **`compiler.rs`**: LaTeX compilation logic (Tectonic, pdflatex, xelatex)
- **`mermaid.rs`**: Mermaid diagram preprocessing and conversion
- **`validator.rs`**: Syntax checking and structure validation
- **`cleaner.rs`**: Auxiliary file cleanup
- **`latex.rs`**: CLI commands using clap-noun-verb

## Dependencies

- `clap-noun-verb` v3.4.0: Noun-verb CLI pattern
- `chicago-tdd-tools`: Testing framework
- `which`: Command path resolution
- `serde`: JSON serialization

## License

MIT OR Apache-2.0

