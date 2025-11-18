# Contributing to YAWL Process Editor

Thank you for your interest in contributing! This guide will help you understand our development process and conventions.

## DOCTRINE Compliance

All contributions MUST adhere to **DOCTRINE 2027** and the covenants defined in `DOCTRINE_COVENANT.md`.

### Before Contributing

Ask yourself:
1. **Which doctrine principle does this embody?** (O, Σ, Q, Π, MAPE-K, or Chatman constant)
2. **What would violate this covenant?**
3. **How is this validated?** (Weaver, Chicago TDD, integration tests)
4. **Where in the codebase does this live?**

### Key Covenants

- **Covenant 1 (Ontology-First)**: All data models must have RDF/Turtle representation
- **Covenant 2 (Invariants Are Law)**: Validation is mandatory, not optional
- **Covenant 3 (Performance)**: Critical operations must complete in ≤8 ticks
- **Covenant 5 (Observability)**: All operations must emit OpenTelemetry spans

## Development Setup

### Prerequisites

- Node.js ≥18.0.0
- npm ≥9.0.0
- Git

### Initial Setup

```bash
git clone https://github.com/ruvnet/knhk.git
cd knhk/apps/nextjs-yawl-editor
npm install
```

### Branch Strategy

```bash
# Create feature branch
git checkout -b feature/your-feature-name

# Create bugfix branch
git checkout -b fix/issue-description
```

## Code Standards

### TypeScript

- **Strict mode**: All code must compile with strict TypeScript settings
- **No `any` types**: Use proper type definitions or `unknown`
- **Explicit return types**: For exported functions
- **No unused variables**: Prefix with `_` if intentionally unused

```typescript
// ✅ GOOD
export function validateWorkflow(workflow: YAWLWorkflow): ValidationResult {
  const errors: ValidationError[] = [];
  // ...
  return { valid: errors.length === 0, errors, warnings: [] };
}

// ❌ BAD
export function validateWorkflow(workflow: any) {
  let errors = [];
  // ...
  return { valid: errors.length === 0, errors };
}
```

### React Components

- Use **functional components** with TypeScript
- Prefer **named exports** over default exports
- Use **const** for component definitions
- Include **prop type definitions**

```typescript
// ✅ GOOD
interface ButtonProps {
  label: string;
  onClick: () => void;
  variant?: 'primary' | 'secondary';
}

export const Button: FC<ButtonProps> = ({ label, onClick, variant = 'primary' }) => {
  return (
    <button onClick={onClick} className={`btn btn-${variant}`}>
      {label}
    </button>
  );
};

// ❌ BAD
export default function Button(props) {
  return <button onClick={props.onClick}>{props.label}</button>;
}
```

### File Organization

- Components in `src/components/`
- Utilities in `src/lib/`
- Types in `src/lib/types/`
- Tests colocated with source files

```
src/
├── components/
│   └── editor/
│       ├── PatternPalette.tsx
│       └── PatternPalette.test.tsx
└── lib/
    └── validation/
        ├── pattern-validator.ts
        └── pattern-validator.test.ts
```

### Naming Conventions

- **Components**: PascalCase (`PatternPalette.tsx`)
- **Files**: kebab-case (`pattern-validator.ts`)
- **Functions**: camelCase (`validateWorkflow`)
- **Constants**: SCREAMING_SNAKE_CASE (`MAX_NODES`)
- **Types/Interfaces**: PascalCase (`ValidationResult`)

## Testing

### Test Requirements

All code changes MUST include tests:

```typescript
import { validateWorkflow } from './pattern-validator';
import { YAWLWorkflow } from '@/types';

describe('validateWorkflow', () => {
  it('should reject workflows without start node', () => {
    const workflow: YAWLWorkflow = {
      id: 'wf-1',
      name: 'Test',
      version: '1.0.0',
      nodes: [],
      edges: [],
    };

    const result = validateWorkflow(workflow);

    expect(result.valid).toBe(false);
    expect(result.errors).toContainEqual(
      expect.objectContaining({
        code: 'NO_START_NODE',
      })
    );
  });
});
```

### Running Tests

```bash
# Run all tests
npm run test

# Run with coverage
npm run test -- --coverage

# Run specific test file
npm run test pattern-validator.test.ts

# Run in watch mode
npm run test:watch
```

### Test Coverage

Minimum coverage requirements:
- **Statements**: 80%
- **Branches**: 75%
- **Functions**: 80%
- **Lines**: 80%

## OpenTelemetry Instrumentation

All new features MUST include telemetry:

```typescript
import { withSpan, recordMetric } from '@/lib/telemetry/setup';

export async function myNewFeature(input: string): Promise<Result> {
  return withSpan('feature.my_new_feature', async () => {
    const startTime = performance.now();

    // Feature implementation
    const result = processInput(input);

    const duration = performance.now() - startTime;
    recordMetric('feature.duration', duration, {
      'input.length': input.length,
    });

    return result;
  }, {
    'feature.name': 'my_new_feature',
  });
}
```

### Telemetry Schema

Update the Weaver schema in `registry/` when adding new telemetry:

```yaml
# registry/yawl-editor-spans.yaml
spans:
  - span_name: feature.my_new_feature
    attributes:
      - name: feature.name
        type: string
      - name: input.length
        type: int
```

## Documentation

### Code Comments

- Use TSDoc format for exported functions
- Include DOCTRINE alignment comments for key files
- Explain WHY, not WHAT

```typescript
/**
 * DOCTRINE ALIGNMENT: Q (Hard Invariants)
 *
 * Validates YAWL workflow against permutation matrix.
 *
 * CRITICAL: Invalid patterns are REJECTED (hard errors).
 * Performance requirement: ≤8 ticks (Chatman Constant)
 *
 * @param workflow - The workflow to validate
 * @returns Validation result with errors and warnings
 *
 * @example
 * ```ts
 * const result = validateWorkflow(myWorkflow);
 * if (!result.valid) {
 *   throw new Error(`Invalid workflow: ${result.errors.join(', ')}`);
 * }
 * ```
 */
export function validateWorkflow(workflow: YAWLWorkflow): ValidationResult {
  // Implementation
}
```

### README Updates

If your change affects:
- Setup/installation
- Configuration
- Public API
- Scripts

Update the appropriate documentation file:
- `README.md` - Overview and quick start
- `GETTING_STARTED.md` - Developer guide
- `ARCHITECTURE.md` - System design

## Pull Request Process

### Before Submitting

1. **Run all checks**:
   ```bash
   npm run type-check
   npm run lint
   npm run test
   npm run format:check
   ```

2. **Update documentation** if needed

3. **Add telemetry** for new features

4. **Write tests** for all changes

### PR Template

```markdown
## Description
Brief description of changes

## DOCTRINE Alignment
- Principle: [O/Σ/Q/Π/MAPE-K/Chatman]
- Covenant: [Number and brief description]

## Changes
- List of specific changes

## Testing
- Description of test coverage

## Checklist
- [ ] TypeScript compiles without errors
- [ ] All tests pass
- [ ] Linting passes
- [ ] Telemetry added (if applicable)
- [ ] Documentation updated
- [ ] DOCTRINE covenant identified
```

### Review Process

1. Submit PR with detailed description
2. Automated checks must pass (CI/CD)
3. Code review by maintainer
4. Address feedback
5. Approval and merge

## Common Issues

### TypeScript Errors

```bash
# Clear cache and rebuild
rm -rf .next
npm run type-check
```

### Lint Errors

```bash
# Auto-fix most issues
npm run lint -- --fix

# Format code
npm run format
```

### Test Failures

```bash
# Run single test file for debugging
npm run test -- --watch pattern-validator.test.ts
```

## Questions?

- Check `DOCTRINE_2027.md` for principles
- Review `DOCTRINE_COVENANT.md` for covenants
- Open an issue on GitHub
- Consult existing code for patterns

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
