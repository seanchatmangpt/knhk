# Publishing to ggen Marketplace

Complete guide to publishing the KNHK YAWL template to the ggen marketplace.

## Prerequisites

1. **ggen CLI installed**: https://github.com/seanchatmangpt/ggen#installation
2. **ggen marketplace account**: Register at the marketplace platform
3. **Template validated**: Run `make check-all` to ensure template is production-ready

## Pre-Publication Checklist

Before publishing, ensure:

```bash
# 1. Validate template structure
make validate

# 2. Test all examples
make test

# 3. Verify documentation completeness
make docs

# 4. Check file structure
make verify-structure

# 5. Run all checks
make check-all
```

All checks must pass with zero errors.

## Publishing Steps

### 1. Version Management

Update version in `ggen.yaml`:

```yaml
version: "X.Y.Z"  # Semantic versioning: MAJOR.MINOR.PATCH
```

Update version in `README.md` and relevant documentation.

### 2. Test in Marketplace

Use local installation for testing:

```bash
# Install locally for development
make install-local

# Test installation
ggen template list | grep io.knhk.yawl-workflows
```

### 3. Prepare for Publishing

```bash
# Ensure git is clean
git status

# Commit all changes
git add .
git commit -m "release: prepare KNHK YAWL marketplace template v$(grep '^version:' ggen.yaml | cut -d' ' -f2)"

# Create git tag
VERSION=$(grep '^version:' ggen.yaml | cut -d' ' -f2)
git tag -a "marketplace/knhk-yawl-workflows/$VERSION" \
  -m "KNHK YAWL Marketplace Template v$VERSION"

# Push tag
git push origin "marketplace/knhk-yawl-workflows/$VERSION"
```

### 4. Publish to Marketplace

Use ggen CLI to publish:

```bash
# Option 1: Publish from template directory
cd ggen-marketplace/knhk-yawl-workflows
ggen marketplace publish --registry https://marketplace.ggen.io

# Option 2: Publish with authentication
ggen marketplace publish \
  --registry https://marketplace.ggen.io \
  --token <YOUR_MARKETPLACE_TOKEN>

# Option 3: Publish with custom name
ggen marketplace publish \
  --registry https://marketplace.ggen.io \
  --id io.knhk.yawl-workflows \
  --version 1.0.0
```

### 5. Verify Publication

```bash
# Search marketplace
ggen marketplace search "knhk yawl"

# Check template info
ggen marketplace info io.knhk.yawl-workflows

# Try installation from marketplace
ggen marketplace install io.knhk.yawl-workflows

# Verify installation
ggen template list | grep io.knhk.yawl-workflows
```

## Publishing Channels

### Official ggen Marketplace

**URL**: https://marketplace.ggen.io/
**Process**: Use `ggen marketplace publish` command
**Requirements**:
- Template passes validation
- Documentation complete
- Examples provided
- Valid ggen.yaml metadata

### Community Registry (Optional)

**URL**: https://registry.knhk.dev/ (KNHK community registry)
**Process**: Submit PR to KNHK repository
**Requirements**:
- Same as official marketplace
- KNHK compatibility documented
- Integration tests passing

## Continuous Integration

The CI/CD pipeline automatically validates on:

- **Push to main**: Full validation suite runs
- **Pull request**: Template validation without publishing
- **Version tags**: Automated deployment to marketplace

Review `.github/workflows/marketplace-template-validate.yml` for CI configuration.

## Update Process

For subsequent releases:

1. **Increment version** in `ggen.yaml` (semantic versioning)
2. **Update CHANGELOG** (if maintained)
3. **Update documentation** for new features
4. **Run all tests**: `make check-all`
5. **Commit and tag**: Follow "Publish to Marketplace" step 3
6. **Publish**: Follow step 4

## Troubleshooting

### Template not found on marketplace

```bash
# Check registration
ggen marketplace info io.knhk.yawl-workflows

# Try re-publishing
ggen marketplace publish --registry https://marketplace.ggen.io --force
```

### Installation fails

```bash
# Verify template locally
ggen template list

# Check template validation
make validate

# Re-install from marketplace
ggen marketplace install io.knhk.yawl-workflows --force
```

### Version conflicts

```bash
# Check available versions
ggen marketplace versions io.knhk.yawl-workflows

# Install specific version
ggen marketplace install io.knhk.yawl-workflows@1.0.0
```

## Documentation for End Users

After publishing, users can install with:

```bash
ggen marketplace install io.knhk.yawl-workflows
```

They should then see:
- Template available in `ggen template list`
- Usage: `ggen template generate-rdf --template io.knhk.yawl-workflows ...`
- Documentation via `ggen marketplace info io.knhk.yawl-workflows`

## Marketplace Metadata

The marketplace uses these fields from `ggen.yaml`:

| Field | Purpose | Example |
|-------|---------|---------|
| `id` | Unique identifier | `io.knhk.yawl-workflows` |
| `version` | Semantic version | `1.0.0` |
| `name` | Display name | `KNHK YAWL Workflow Generator` |
| `description` | Short description | `Generate YAWL specifications from RDF ontologies` |
| `author` | Author name | `Sean Chatman` |
| `repository` | Source repository | `https://github.com/seanchatmangpt/knhk` |
| `license` | License identifier | `MIT` |

## Support

- **Documentation**: See [README.md](README.md) and `docs/`
- **Issues**: https://github.com/seanchatmangpt/knhk/issues
- **Marketplace Help**: https://marketplace.ggen.io/docs

## Quality Standards for Marketplace

All published templates must meet:

✓ **Code Quality**
- Syntax validation passing
- No hardcoded errors
- Proper error handling

✓ **Documentation**
- Complete README.md
- Usage guide
- Working examples
- API documentation

✓ **Testing**
- All examples tested
- CI/CD passing
- Integration tests provided

✓ **Maintenance**
- Version tracking
- Changelog
- Security updates
- Community support

---

**Remember**: The marketplace is built on trust. Maintain quality standards and respond to user feedback promptly.
