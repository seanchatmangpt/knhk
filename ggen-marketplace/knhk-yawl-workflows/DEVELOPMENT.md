# Development Guide

Local development setup and contribution guidelines for the KNHK YAWL marketplace template.

## Quick Start

```bash
# Clone repository
git clone https://github.com/seanchatmangpt/knhk.git
cd knhk/ggen-marketplace/knhk-yawl-workflows

# Setup development environment
make dev-setup

# Run validation checks
make check-all
```

## Development Workflow

### 1. Making Changes

```bash
# Create feature branch
git checkout -b feature/your-feature-name

# Make changes to files
# - Edit templates in template/
# - Add examples in examples/
# - Add SPARQL queries in queries/
# - Update documentation in docs/

# Run validation
make validate

# Test changes
make test
```

### 2. Testing Changes

```bash
# Validate entire template
make validate

# Test example workflows
make test

# Test YAWL generation (requires ggen)
make test-generate

# Verify structure
make verify-structure

# Run all checks
make check-all
```

### 3. Documentation

Update documentation for:
- **New SPARQL queries**: Add to `docs/EXAMPLES.md`
- **New patterns**: Document in `docs/ARCHITECTURE.md`
- **API changes**: Update `docs/USAGE.md`
- **Examples**: Add `.ttl` files to `examples/`

```bash
# Build documentation site (if configured)
make docs
```

### 4. Commit and Push

```bash
# Review changes
git status
git diff

# Commit with descriptive message
git add .
git commit -m "feat: add new YAWL pattern support

- Add extract_custom_pattern.sparql query
- Add custom pattern example
- Update documentation"

# Push to feature branch
git push origin feature/your-feature-name

# Create Pull Request on GitHub
```

## File Structure

```
knhk-yawl-workflows/
├── ggen.yaml                    # Marketplace metadata (READ ONLY)
├── README.md                    # Main documentation
├── PUBLISH.md                   # Publishing guide
├── DEVELOPMENT.md               # This file
├── Makefile                     # Development commands
│
├── template/
│   ├── yawl-workflow.xml.j2    # YAWL XML generation
│   └── yawl-workflow.json.j2   # YAWL JSON generation
│
├── queries/
│   ├── extract_workflows.sparql
│   ├── extract_tasks.sparql
│   ├── extract_conditions.sparql
│   ├── extract_flows.sparql
│   ├── extract_patterns.sparql
│   └── extract_metadata.sparql
│
├── examples/
│   ├── simple-sequence.ttl
│   ├── parallel-split.ttl
│   └── exclusive-choice.ttl
│
├── docs/
│   ├── USAGE.md
│   ├── ARCHITECTURE.md
│   └── EXAMPLES.md
│
└── scripts/
    ├── validate-template.sh     # Validation script
    └── test-examples.sh         # Integration tests
```

## Key Files to Understand

### ggen.yaml
Marketplace template metadata. Defines:
- Template ID and version
- Input/output formats
- SPARQL queries to execute
- Required namespaces
- Supported patterns

**Edit when**: Adding new SPARQL queries, changing template behavior

### Jinja2 Templates
Template files that generate YAWL output:
- `template/yawl-workflow.xml.j2` - YAWL 2.2 XML format
- `template/yawl-workflow.json.j2` - JSON format

**Edit when**: Changing YAWL output structure, adding new fields

### SPARQL Queries
Extract semantic structure from RDF:
- `queries/extract_workflows.sparql` - Find workflow specifications
- `queries/extract_tasks.sparql` - Extract task definitions
- `queries/extract_conditions.sparql` - Extract places/conditions
- `queries/extract_flows.sparql` - Extract control flows
- `queries/extract_patterns.sparql` - Extract routing patterns
- `queries/extract_metadata.sparql` - Extract metadata

**Edit when**: Need to extract different information from RDF

### Examples
Turtle RDF example workflows:
- `examples/simple-sequence.ttl` - Basic sequential pattern
- `examples/parallel-split.ttl` - AND split/join pattern
- `examples/exclusive-choice.ttl` - XOR conditional pattern

**Edit when**: Adding new pattern examples, testing RDF ontology

## Common Tasks

### Add a New YAWL Pattern

1. **Create Turtle example** in `examples/`:
```turtle
@prefix yawl: <http://bitflow.ai/ontology/yawl/v2#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

<http://example.org/workflow/my-pattern> a yawl:WorkflowSpecification ;
    rdfs:label "My Pattern" ;
    yawl:hasTask <http://example.org/task/example> .

<http://example.org/task/example> a yawl:Task ;
    rdfs:label "Example Task" ;
    yawl:hasSplitType yawl:CustomPattern .
```

2. **Update SPARQL queries** if new properties needed:
```sparql
# Add to queries/extract_patterns.sparql
OPTIONAL {
  ?element yawl:hasCustomPattern ?customProp .
  BIND("custom-pattern" AS ?patternType)
}
```

3. **Update ggen.yaml** metadata:
```yaml
patterns:
  new_patterns:
    - "my-custom-pattern"
```

4. **Update documentation** in `docs/EXAMPLES.md`

5. **Test**:
```bash
make validate
make test
make test-generate
```

### Add a New Output Format

1. **Create Jinja2 template** in `template/`:
```
template/yawl-workflow.pnml.j2
```

2. **Update ggen.yaml**:
```yaml
output_formats:
  - "yawl-xml"
  - "yawl-json"
  - "pnml"  # New format
```

3. **Update README** with format information

4. **Test generation**:
```bash
make test-generate
```

### Fix a Bug

1. **Create test case** demonstrating the bug
2. **Fix the code** (template, query, or script)
3. **Verify fix**:
```bash
make check-all
```
4. **Commit** with description of fix

### Improve Documentation

1. **Edit markdown** in `docs/`
2. **Test rendering** (local markdown viewer)
3. **Verify links** and examples
4. **Run linter**:
```bash
make lint
```
5. **Commit** documentation changes

## Testing Workflow

### Manual Testing

```bash
# 1. Generate YAWL from example
ggen template generate-rdf \
  --ontology examples/simple-sequence.ttl \
  --template io.knhk.yawl-workflows \
  --output test-output.yawl

# 2. Inspect output
cat test-output.yawl

# 3. Validate YAWL (if tools available)
xmllint test-output.yawl
```

### Automated Testing

```bash
# Run all tests
make check-all

# Individual test categories
make validate        # Template structure validation
make test           # Example workflow tests
```

### CI/CD Testing

Tests run automatically on:
- **Push**: Full validation suite
- **Pull Request**: Template validation
- **Release tags**: Publication validation

View results in GitHub Actions: `.github/workflows/`

## Debugging

### Template Issues

```bash
# Test SPARQL queries in isolation
ggen graph load examples/simple-sequence.ttl
ggen graph query --ontology examples/simple-sequence.ttl --sparql \
  'PREFIX yawl: <http://bitflow.ai/ontology/yawl/v2#>
   SELECT ?task WHERE { ?task a yawl:Task }'

# Generate with verbose output
ggen template generate-rdf \
  --ontology examples/simple-sequence.ttl \
  --template io.knhk.yawl-workflows \
  --verbose
```

### RDF/Turtle Issues

```bash
# Validate Turtle syntax
ggen graph load your-file.ttl

# Query to find errors
ggen graph query --ontology your-file.ttl --sparql \
  'SELECT * WHERE { ?s ?p ?o } LIMIT 10'
```

### YAWL Output Issues

```bash
# Pretty-print YAWL XML
xmllint --format output.yawl

# Validate YAWL schema (if schema available)
xmllint --schema yawl.xsd output.yawl
```

## Code Style Guidelines

### Jinja2 Templates

- Use `{%- --%}` to control whitespace
- Comments use `{#- comment -#}`
- Variables: `{{ variable | filter }}`
- Prefer filters over logic loops
- Test for variable existence: `{% if var %}`

### SPARQL Queries

- PREFIX declarations at top
- Clear SELECT clause
- Use OPTIONAL for non-mandatory fields
- Order results with `ORDER BY`
- Add comments explaining each section

### Turtle Examples

- Use meaningful URIs (not generic ids)
- Include rdfs:label and rdfs:comment
- Complete workflow definitions (start + end)
- Document patterns used

### Documentation

- Clear headings (# H1, ## H2, ### H3)
- Code blocks with language specification
- Examples with expected output
- Links to related documentation
- Tables for reference material

## Release Process

See [PUBLISH.md](PUBLISH.md) for:
- Version management
- Testing before release
- Publishing to marketplace
- Documentation updates
- Maintenance procedures

## Getting Help

- **Issues**: https://github.com/seanchatmangpt/knhk/issues
- **Documentation**: [README.md](README.md), `docs/`
- **ggen Help**: https://github.com/seanchatmangpt/ggen
- **YAWL Spec**: http://www.yawlfoundation.org/

## Contributing

1. Fork the repository
2. Create feature branch
3. Make changes and test locally
4. Submit pull request with description
5. Address review feedback
6. Merge when approved

All contributions are welcome! See [CONTRIBUTING](../../CONTRIBUTING.md) for more details.
