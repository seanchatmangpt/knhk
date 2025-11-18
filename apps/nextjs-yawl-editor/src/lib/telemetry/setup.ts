/**
 * DOCTRINE ALIGNMENT: Σ (Sum of All Fears - System Integrity)
 * OpenTelemetry instrumentation setup
 *
 * CRITICAL: All telemetry MUST conform to Weaver schema
 * This is the ONLY source of truth for validation
 */

import { trace, metrics, SpanStatusCode } from '@opentelemetry/api';
import type { Span, Tracer } from '@opentelemetry/api';

/* ============================================================================
 * Tracer and Meter Initialization
 * ========================================================================== */

const TRACER_NAME = 'yawl-editor';
const METER_NAME = 'yawl-editor';

let tracer: Tracer | null = null;

/**
 * Get or create tracer instance
 */
export function getTracer(): Tracer {
  if (!tracer) {
    tracer = trace.getTracer(TRACER_NAME, '0.1.0');
  }
  return tracer;
}

/* ============================================================================
 * Span Management
 * ========================================================================== */

/**
 * Create and start a new span
 *
 * @example
 * ```ts
 * const span = createSpan('workflow.validate', { workflowId: 'wf-123' });
 * try {
 *   // ... validation logic
 *   span.setStatus({ code: SpanStatusCode.OK });
 * } catch (error) {
 *   recordError(span, error);
 * } finally {
 *   span.end();
 * }
 * ```
 */
export function createSpan(
  name: string,
  attributes: Record<string, string | number | boolean> = {}
): Span {
  const tracer = getTracer();
  return tracer.startSpan(name, {
    attributes: {
      'service.name': 'yawl-editor',
      'service.version': '0.1.0',
      ...attributes,
    },
  });
}

/**
 * Execute function within a span context
 *
 * @example
 * ```ts
 * const result = await withSpan('workflow.parse', async () => {
 *   return parseTurtle(turtleString);
 * }, { format: 'turtle' });
 * ```
 */
export async function withSpan<T>(
  name: string,
  fn: () => Promise<T>,
  attributes: Record<string, string | number | boolean> = {}
): Promise<T> {
  const span = createSpan(name, attributes);

  try {
    const result = await fn();
    span.setStatus({ code: SpanStatusCode.OK });
    return result;
  } catch (error) {
    recordError(span, error as Error);
    throw error;
  } finally {
    span.end();
  }
}

/**
 * Record error in span with proper formatting
 */
export function recordError(span: Span, error: Error): void {
  span.recordException(error);
  span.setStatus({
    code: SpanStatusCode.ERROR,
    message: error.message,
  });
}

/* ============================================================================
 * Metrics
 * ========================================================================== */

/**
 * Record performance metric
 *
 * @example
 * ```ts
 * recordMetric('workflow.validation.duration', 125, { valid: true });
 * ```
 */
export function recordMetric(
  name: string,
  value: number,
  attributes: Record<string, string | number | boolean> = {}
): void {
  const meter = metrics.getMeter(METER_NAME, '0.1.0');
  const histogram = meter.createHistogram(name, {
    description: `${name} metric`,
    unit: 'ms',
  });

  histogram.record(value, attributes);
}

/* ============================================================================
 * Convenience Functions for YAWL-Specific Operations
 * ========================================================================== */

/**
 * Track workflow validation with telemetry
 */
export async function trackValidation<T>(
  workflowId: string,
  fn: () => Promise<T>
): Promise<T> {
  const startTime = performance.now();

  const result = await withSpan(
    'workflow.validate',
    fn,
    { 'workflow.id': workflowId }
  );

  const duration = performance.now() - startTime;
  recordMetric('workflow.validation.duration', duration, {
    'workflow.id': workflowId,
  });

  return result;
}

/**
 * Track RDF parsing with telemetry
 */
export async function trackRDFParsing<T>(
  format: string,
  fn: () => Promise<T>
): Promise<T> {
  const startTime = performance.now();

  const result = await withSpan(
    'rdf.parse',
    fn,
    { 'rdf.format': format }
  );

  const duration = performance.now() - startTime;
  recordMetric('rdf.parse.duration', duration, {
    'rdf.format': format,
  });

  return result;
}

/**
 * Track editor operations
 */
export function trackEditorOperation(
  operation: string,
  attributes: Record<string, string | number | boolean> = {}
): void {
  const span = createSpan('editor.operation', {
    'editor.operation': operation,
    ...attributes,
  });
  span.end();

  recordMetric('editor.operation.count', 1, {
    'editor.operation': operation,
  });
}
