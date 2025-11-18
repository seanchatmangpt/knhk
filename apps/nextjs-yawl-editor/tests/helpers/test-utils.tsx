/**
 * Test Utilities and Helpers
 *
 * DOCTRINE ALIGNMENT:
 * - London School TDD: Mock-first approach
 * - Behavior verification over state inspection
 */

import React, { ReactElement } from 'react';
import { render, RenderOptions } from '@testing-library/react';
import type { RenderResult } from '@testing-library/react';

/**
 * Custom render function with providers
 */
export function renderWithProviders(
  ui: ReactElement,
  options?: Omit<RenderOptions, 'wrapper'>
): RenderResult {
  const Wrapper = ({ children }: { children: React.ReactNode }) => {
    return <>{children}</>;
  };

  return render(ui, { wrapper: Wrapper, ...options });
}

/**
 * Mock performance.now() for deterministic timing tests
 */
export function mockPerformanceNow() {
  let time = 0;
  const originalNow = performance.now;

  beforeEach(() => {
    time = 0;
    performance.now = jest.fn(() => time);
  });

  afterEach(() => {
    performance.now = originalNow;
  });

  return {
    advance: (ms: number) => {
      time += ms;
    },
    set: (ms: number) => {
      time = ms;
    },
    get: () => time,
  };
}

/**
 * Wait for async operations
 */
export const waitFor = async (ms: number) =>
  new Promise(resolve => setTimeout(resolve, ms));

/**
 * Create mock OTEL span
 */
export function createMockSpan() {
  return {
    setStatus: jest.fn(),
    recordException: jest.fn(),
    end: jest.fn(),
    setAttribute: jest.fn(),
    addEvent: jest.fn(),
  };
}

/**
 * Create mock OTEL tracer
 */
export function createMockTracer() {
  return {
    startSpan: jest.fn(() => createMockSpan()),
    startActiveSpan: jest.fn((name: string, fn: Function) => {
      const span = createMockSpan();
      return fn(span);
    }),
  };
}

/**
 * Assert performance constraint (Chicago TDD)
 */
export function assertPerformance(
  operation: () => void | Promise<void>,
  maxMs: number,
  label: string = 'operation'
) {
  return async () => {
    const start = performance.now();
    await operation();
    const elapsed = performance.now() - start;

    expect(elapsed).toBeLessThanOrEqual(maxMs);
    console.log(`✓ ${label}: ${elapsed.toFixed(2)}ms (limit: ${maxMs}ms)`);
  };
}

/**
 * Create deterministic test data
 */
export function createTestNode(id: string, type: 'task' | 'condition', overrides = {}) {
  return {
    id,
    type,
    position: { x: 100, y: 100 },
    data: {
      label: `${type} ${id}`,
      ...(type === 'task' ? { decomposition: 'atomic', resources: [], properties: {} } : {}),
      ...(type === 'condition' ? { conditionType: 'intermediate', splitType: 'none', joinType: 'none' } : {}),
      ...overrides,
    },
  };
}

/**
 * Create test edge
 */
export function createTestEdge(source: string, target: string, data = {}) {
  return {
    id: `${source}-${target}`,
    source,
    target,
    data,
  };
}

// Re-export testing library utilities
export * from '@testing-library/react';
export { default as userEvent } from '@testing-library/user-event';
