# YAWL Editor Test Suite

Comprehensive test suite for the Next.js YAWL workflow editor following London School TDD methodology and DOCTRINE covenants.

## Quick Start

```bash
# Install dependencies
npm install
npm run playwright:install

# Run all tests
npm test                  # Jest tests
npm run test:e2e          # Playwright E2E tests
npm run test:all          # All Jest test categories

# Run specific categories
npm run test:unit         # Unit tests
npm run test:integration  # Integration tests
npm run test:performance  # Chicago TDD performance tests
npm run test:weaver       # Weaver schema validation

# Coverage
npm run test:coverage     # Generate coverage report

# Development
npm run test:watch        # Watch mode
npm run test:e2e:ui       # E2E with Playwright UI
```

## Test Structure

```
tests/
├── fixtures/           # Test data (workflows, mocks, validation cases)
├── helpers/            # Test utilities (custom render, mocks, assertions)
├── components/         # Component unit tests
├── lib/                # Library unit tests
├── integration/        # Integration tests (component interactions)
├── performance/        # Chicago TDD performance tests
├── weaver/             # Weaver schema validation tests
└── e2e/                # Playwright end-to-end tests
```

## DOCTRINE Alignment

- **Covenant 1**: RDF round-trip integrity (`integration/`)
- **Covenant 2**: Pattern validation enforcement (`lib/pattern-validator.test.ts`)
- **Covenant 3**: MAPE-K feedback loops (`integration/`)
- **Covenant 5**: Chatman Constant ≤8ms (`performance/`)
- **Covenant 6**: Observable telemetry (`weaver/`)

## Testing Methodology

**London School TDD**:
- Mock-first approach
- Behavior verification over state inspection
- Focus on object interactions

**Chicago TDD**:
- Performance-focused testing
- State-based assertions
- Latency constraints (≤8ms hot path)

## Key Test Files

### Unit Tests

- `components/workflow-canvas.test.tsx` - Canvas rendering and RDF projection
- `components/task-node.test.tsx` - Node rendering and status indicators
- `lib/pattern-validator.test.ts` - Pattern validation (Covenant 2)
- `lib/knhk-client.test.ts` - Kernel communication

### Integration Tests

- `integration/editor-workflow.test.tsx` - Full workflow scenarios

### Performance Tests

- `performance/validation-latency.test.ts` - Benchmarking (Covenant 5)

### Schema Validation

- `weaver/schema-validation.test.ts` - Weaver validation (Covenant 6)

### E2E Tests

- `e2e/editor-workflow.spec.ts` - Complete user workflows

## Test Fixtures

### Sample Workflows
- Simple sequence
- Parallel split (AND-split/join)
- Exclusive choice (XOR-split/join)
- Invalid workflows (missing end, unmatched splits)
- Performance test workflows (50-200 nodes)

### Mock Kernels
- Workflow submission responses
- Validation errors
- Execution traces
- MAPE-K cycle outputs
- Pattern matrix entries

### Validation Cases
- Valid patterns
- Invalid patterns
- Edge cases
- Performance benchmarks

## Coverage Targets

- **Line coverage**: >80%
- **Branch coverage**: >75%
- **Function coverage**: >85%
- **Statement coverage**: >80%

Coverage is enforced in `jest.config.ts`. CI will fail if targets not met.

## Performance Constraints

### Chatman Constant (Covenant 5)

- **Hot path**: ≤8ms (workflow validation, pattern checks)
- **Warm path**: ≤100ms (complex workflows)
- **Cold path**: ≤500ms (large workflows 200+ nodes)

Performance tests measure p50, p95, p99 percentiles over 100 iterations.

## Weaver Validation

**Critical**: Weaver schema validation is the ONLY source of truth for KNHK.

```bash
# Validate schema structure
weaver registry check -r ../../registry/

# Validate runtime telemetry
weaver registry live-check --registry ../../registry/
```

**Truth Hierarchy**:
1. Weaver validation (source of truth)
2. Compilation (code quality baseline)
3. Traditional tests (supporting evidence)

If Weaver fails, the feature does NOT work, regardless of test results.

## CI/CD Integration

```yaml
# GitHub Actions example
- name: Run tests
  run: npm run test:ci

- name: Weaver validation
  run: weaver registry check -r registry/

- name: Upload coverage
  uses: codecov/codecov-action@v3
```

## Writing Tests

### Unit Test Example (London School)

```typescript
import { renderWithProviders } from '../helpers/test-utils';
import { MyComponent } from '@/components/my-component';
import { useMyHook } from '@/hooks/use-my-hook';

jest.mock('@/hooks/use-my-hook');

describe('MyComponent', () => {
  const mockHook = {
    data: { value: 'test' },
    update: jest.fn(),
  };

  beforeEach(() => {
    (useMyHook as jest.Mock).mockReturnValue(mockHook);
  });

  it('should interact with hook correctly', () => {
    renderWithProviders(<MyComponent />);

    // Verify behavior: component talks to hook
    expect(useMyHook).toHaveBeenCalled();
  });
});
```

### Performance Test Example (Chicago TDD)

```typescript
it('should validate in ≤8ms (Chatman Constant)', () => {
  const start = performance.now();

  validateWorkflow(workflow);

  const elapsed = performance.now() - start;

  expect(elapsed).toBeLessThan(8);
  console.log(`✓ Validation: ${elapsed.toFixed(2)}ms`);
});
```

### Integration Test Example

```typescript
it('should update RDF when edge is added', async () => {
  renderWithProviders(<WorkflowCanvas />);

  // Simulate edge creation
  fireEvent.click(screen.getByTestId('connect-button'));

  await waitFor(() => {
    expect(mockWorkflow.addEdge).toHaveBeenCalled();
  });
});
```

### E2E Test Example (Playwright)

```typescript
test('should create workflow and export Turtle', async ({ page }) => {
  await page.goto('/editor');
  await page.click('[data-testid="add-task-button"]');
  await page.click('[data-testid="export-button"]');

  const download = await page.waitForEvent('download');
  expect(download.suggestedFilename()).toMatch(/\.ttl$/);
});
```

## Test Utilities

### Custom Render
```typescript
import { renderWithProviders } from './helpers/test-utils';

renderWithProviders(<Component />);
```

### Performance Mocking
```typescript
const perfTimer = mockPerformanceNow();
perfTimer.advance(50);  // Advance 50ms
```

### Mock OTEL
```typescript
const mockSpan = createMockSpan();
const mockTracer = createMockTracer();
```

### Test Data
```typescript
import { simpleSequenceWorkflow, mockKernelResponses } from './fixtures';
```

## Troubleshooting

### Tests failing after component changes
- Check mock implementations in `jest.setup.ts`
- Verify hook contracts haven't changed
- Update test fixtures if RDF structure changed

### Performance tests failing
- Check if running on slow CI machines
- Verify no debug code slowing execution
- Review algorithm complexity

### E2E tests timing out
- Increase timeout in `playwright.config.ts`
- Check if dev server is running
- Verify network requests aren't blocked

### Coverage below threshold
- Add tests for uncovered branches
- Remove dead code
- Check `jest.config.ts` coverage paths

## References

- [London School TDD](https://github.com/testdouble/contributing-tests/wiki/London-school-TDD)
- [Chicago School TDD](https://github.com/testdouble/contributing-tests/wiki/Detroit-school-TDD)
- [OpenTelemetry Weaver](https://github.com/open-telemetry/weaver)
- [DOCTRINE_2027.md](/home/user/knhk/DOCTRINE_2027.md)
- [DOCTRINE_COVENANT.md](/home/user/knhk/DOCTRINE_COVENANT.md)

## Support

For questions or issues:
1. Check `TEST_SUMMARY.md` for detailed test documentation
2. Review `DOCTRINE_COVENANT.md` for covenant alignment
3. Check existing test files for examples
4. Consult team documentation

---

**Test Suite Version**: 1.0.0
**Last Updated**: 2025-11-18
**Status**: ✅ COMPLETE
