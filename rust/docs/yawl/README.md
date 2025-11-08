# YAWL Artifacts Documentation

**Last Updated**: January 2025  
**Source**: Extracted from `yawl.txt` (21,649 lines)

## Overview

This directory contains extracted artifacts from the YAWL workflow engine documentation, including C4 architecture diagrams and code examples. These artifacts provide reference material for understanding the KNHK Workflow Engine architecture and implementation patterns.

## Contents

### Diagrams (`diagrams/`)
- **9 C4 PlantUML diagrams** covering architecture from context to code level
- See [DIAGRAMS_README.md](DIAGRAMS_README.md) for detailed documentation

### Code Files (`code/`)
- **407 code files** extracted from documentation examples
- Includes Rust, C, TOML, YAML, JSON, XML, Turtle, and shell scripts
- See [CODE_INDEX.md](CODE_INDEX.md) for categorized listings

## Quick Start

### Viewing Diagrams

C4 diagrams are in PlantUML format. To render them:

1. **Online**: Use [PlantUML Web Server](http://www.plantuml.com/plantuml/uml/)
   - Copy the `.puml` file contents
   - Paste into the web editor

2. **VS Code**: Install the "PlantUML" extension
   - Open any `.puml` file
   - Press `Alt+D` to preview

3. **Command Line**: Install PlantUML and Java
   ```bash
   brew install plantuml  # macOS
   plantuml diagrams/*.puml
   ```

### Finding Code Examples

1. **By Language**: See [CODE_INDEX.md](CODE_INDEX.md) for file type breakdown
2. **By Pattern**: Search for pattern numbers (e.g., `P01`, `P16`)
3. **By Component**: Look for component names (e.g., `PatternRegistry`, `ExecutionEngine`)

### Common Use Cases

- **Architecture Review**: Start with `diagrams/C1_Context.puml` and work through C2-C4
- **Pattern Implementation**: Check `code/rust_*.rs` files for pattern executors
- **Configuration Examples**: See `code/*.toml` and `code/*.yaml` files
- **Hot Path Code**: Look for `code/*.c` files for C implementations

## File Organization

```
yawl/
├── README.md              # This file
├── CODE_INDEX.md          # Code file index and categorization
├── DIAGRAMS_README.md     # Diagram documentation
├── yawl.txt              # Source documentation file
├── diagrams/             # C4 architecture diagrams
│   ├── C1_Context.puml
│   ├── C2_Container.puml
│   ├── C3_Components_*.puml
│   ├── C4_Code_Level.puml
│   └── Reflex_Enterprise_Container_View.puml
└── code/                  # Extracted code examples
    ├── rust_*.rs         # Rust code examples
    ├── c_*.c             # C code examples
    ├── *.toml            # Configuration files
    ├── *.yaml            # YAML configurations
    └── ...               # Other file types
```

## Extraction Process

Code blocks and diagrams were extracted from `yawl.txt` using automated parsing that identified:
- Language markers (e.g., `rust`, `c`, `toml`)
- File path markers (e.g., `src/lib.rs`, `Cargo.toml`)
- PlantUML diagram blocks (`@startuml` ... `@enduml`)

## Related Documentation

- [Architecture Improvements](ARCHITECTURE_IMPROVEMENTS.md) - Workflow engine architecture
- [Main Documentation Index](../../docs/INDEX.md) - Complete KNHK documentation
- [Workflow Patterns](../../docs/architecture.md) - Pattern documentation

## Contributing

When adding new artifacts:
1. Update the appropriate index file (`CODE_INDEX.md` or `DIAGRAMS_README.md`)
2. Maintain consistent naming conventions
3. Include brief descriptions for new files

