import '@testing-library/jest-dom';

// Mock OpenTelemetry API to avoid initialization issues in tests
jest.mock('@opentelemetry/api', () => ({
  trace: {
    getTracer: jest.fn(() => ({
      startSpan: jest.fn(() => ({
        setStatus: jest.fn(),
        recordException: jest.fn(),
        end: jest.fn(),
      })),
    })),
  },
  metrics: {
    getMeter: jest.fn(() => ({
      createHistogram: jest.fn(() => ({
        record: jest.fn(),
      })),
    })),
  },
  context: {},
  SpanStatusCode: {
    OK: 1,
    ERROR: 2,
  },
}));
