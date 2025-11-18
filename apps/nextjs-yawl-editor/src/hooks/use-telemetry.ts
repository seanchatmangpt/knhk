/**
 * DOCTRINE ALIGNMENT: Σ (Sum of All Fears - System Integrity)
 * Custom hook for OpenTelemetry instrumentation in React components
 *
 * CRITICAL: All telemetry MUST conform to Weaver schema
 */

'use client';

import { useCallback, useEffect, useRef } from 'react';
import { createSpan, recordMetric } from '@/lib/telemetry/setup';
import type { Span } from '@opentelemetry/api';
import { SpanStatusCode } from '@opentelemetry/api';

export interface UseTelemetryReturn {
  trackEvent: (name: string, attributes?: Record<string, string | number | boolean>) => void;
  trackMetric: (name: string, value: number, attributes?: Record<string, string | number | boolean>) => void;
  withSpan: <T>(name: string, fn: () => T | Promise<T>, attributes?: Record<string, string | number | boolean>) => Promise<T>;
  startSpan: (name: string, attributes?: Record<string, string | number | boolean>) => Span;
}

/**
 * Hook for component-level OpenTelemetry instrumentation
 *
 * @param componentName - Name of the component for telemetry context
 *
 * @example
 * ```tsx
 * function MyComponent() {
 *   const { trackEvent, trackMetric, withSpan } = useTelemetry('MyComponent');
 *
 *   const handleClick = () => {
 *     trackEvent('button.click', { buttonId: 'save' });
 *   };
 *
 *   const loadData = async () => {
 *     return withSpan('data.load', async () => {
 *       const data = await fetchData();
 *       trackMetric('data.size', data.length);
 *       return data;
 *     });
 *   };
 *
 *   return <button onClick={handleClick}>Save</button>;
 * }
 * ```
 */
export function useTelemetry(componentName: string): UseTelemetryReturn {
  const mountSpanRef = useRef<Span | null>(null);

  // Track component lifecycle
  useEffect(() => {
    const span = createSpan(`${componentName}.mount`, {
      'component.name': componentName,
    });
    mountSpanRef.current = span;

    return () => {
      if (mountSpanRef.current) {
        mountSpanRef.current.setStatus({ code: SpanStatusCode.OK });
        mountSpanRef.current.end();
      }
    };
  }, [componentName]);

  const trackEvent = useCallback((
    name: string,
    attributes: Record<string, string | number | boolean> = {}
  ) => {
    const span = createSpan(`${componentName}.${name}`, {
      'component.name': componentName,
      ...attributes,
    });
    span.setStatus({ code: SpanStatusCode.OK });
    span.end();
  }, [componentName]);

  const trackMetric = useCallback((
    name: string,
    value: number,
    attributes: Record<string, string | number | boolean> = {}
  ) => {
    recordMetric(`${componentName}.${name}`, value, {
      'component.name': componentName,
      ...attributes,
    });
  }, [componentName]);

  const withSpan = useCallback(async <T,>(
    name: string,
    fn: () => T | Promise<T>,
    attributes: Record<string, string | number | boolean> = {}
  ): Promise<T> => {
    const span = createSpan(`${componentName}.${name}`, {
      'component.name': componentName,
      ...attributes,
    });

    try {
      const result = await Promise.resolve(fn());
      span.setStatus({ code: SpanStatusCode.OK });
      return result;
    } catch (error) {
      span.recordException(error as Error);
      span.setStatus({ code: SpanStatusCode.ERROR });
      throw error;
    } finally {
      span.end();
    }
  }, [componentName]);

  const startSpan = useCallback((
    name: string,
    attributes: Record<string, string | number | boolean> = {}
  ): Span => {
    return createSpan(`${componentName}.${name}`, {
      'component.name': componentName,
      ...attributes,
    });
  }, [componentName]);

  return {
    trackEvent,
    trackMetric,
    withSpan,
    startSpan,
  };
}
