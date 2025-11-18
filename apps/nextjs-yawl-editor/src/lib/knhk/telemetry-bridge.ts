/**
 * DOCTRINE ALIGNMENT: O (Observation) + Σ (System Integrity)
 * Bridge editor telemetry to knhk OpenTelemetry registry
 *
 * COVENANT 6: Observations drive everything
 * All operations MUST emit proper telemetry validated by Weaver
 */

// Telemetry imports (used for future enhancements)
// import { trace, metrics, context } from '@opentelemetry/api';
import { createSpan, withSpan } from '@/lib/telemetry/setup';
import { knhkConfig, getOTLPEndpoint } from './config';
import type {
  ReadableSpan,
  SchemaValidation,
  OTelSchema,
  SchemaStatus,
  Metric,
  Operation,
  Result,
  ApiResponse,
} from './types';

/* ============================================================================
 * Telemetry Bridge
 * ========================================================================== */

/**
 * Bridge between editor telemetry and knhk ecosystem
 *
 * - Validates all telemetry against Weaver schema
 * - Exports metrics to OTLP endpoint
 * - Logs receipts for audit trail
 * - Ensures compliance with DOCTRINE observability requirements
 */
export class TelemetryBridge {
  private schemaCache: Map<string, SchemaStatus> = new Map();
  private pendingMetrics: Metric[] = [];
  private exportInterval: NodeJS.Timeout | null = null;

  constructor() {
    // Start metric export interval
    if (knhkConfig.telemetry.exportInterval > 0) {
      this.startExporting();
    }
  }

  /* ============================
   * Schema Validation (Weaver)
   * ============================ */

  /**
   * Validate span against Weaver schema
   *
   * CRITICAL: This is the ONLY source of truth for validation
   * Tests can have false positives; Weaver schema validation cannot
   */
  async validateSchema(span: ReadableSpan): Promise<SchemaValidation> {
    if (!knhkConfig.telemetry.schemaValidation) {
      return {
        valid: true,
        errors: [],
        warnings: [],
        schemaVersion: 'validation-disabled',
      };
    }

    return withSpan(
      'telemetry.validate_schema',
      async () => {
        const response = await fetch(
          `${knhkConfig.telemetry.weaverRegistryUrl}/validate/span`,
          {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
              span: {
                name: span.name,
                kind: span.kind,
                attributes: span.attributes,
                events: span.events,
                status: span.status,
              },
            }),
            signal: AbortSignal.timeout(5000),
          }
        );

        if (!response.ok) {
          if (knhkConfig.telemetry.failOnSchemaError) {
            throw new Error(`Schema validation failed: ${response.statusText}`);
          }
          console.warn('Schema validation request failed:', response.statusText);
          return {
            valid: false,
            errors: [
              {
                code: 'VALIDATION_FAILED',
                message: `HTTP ${response.status}: ${response.statusText}`,
              },
            ],
            warnings: [],
            schemaVersion: 'unknown',
          };
        }

        const result = (await response.json()) as SchemaValidation;

        // Log validation failures
        if (!result.valid && knhkConfig.dev.debug) {
          console.error('Schema validation failed for span:', span.name, result.errors);
        }

        return result;
      },
      {
        'span.name': span.name,
        'span.kind': span.kind,
      }
    );
  }

  /**
   * Validate metric against schema
   */
  async validateMetric(metric: Metric): Promise<SchemaValidation> {
    if (!knhkConfig.telemetry.schemaValidation) {
      return {
        valid: true,
        errors: [],
        warnings: [],
        schemaVersion: 'validation-disabled',
      };
    }

    return withSpan(
      'telemetry.validate_metric',
      async () => {
        const response = await fetch(
          `${knhkConfig.telemetry.weaverRegistryUrl}/validate/metric`,
          {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ metric }),
            signal: AbortSignal.timeout(5000),
          }
        );

        if (!response.ok) {
          if (knhkConfig.telemetry.failOnSchemaError) {
            throw new Error(`Metric validation failed: ${response.statusText}`);
          }
          return {
            valid: false,
            errors: [
              {
                code: 'VALIDATION_FAILED',
                message: `HTTP ${response.status}`,
              },
            ],
            warnings: [],
            schemaVersion: 'unknown',
          };
        }

        return (await response.json()) as SchemaValidation;
      },
      {
        'metric.name': metric.name,
        'metric.type': metric.type,
      }
    );
  }

  /* ============================
   * Schema Registration (Weaver)
   * ============================ */

  /**
   * Register an OpenTelemetry schema with Weaver
   *
   * This defines the contract for what telemetry the editor emits
   */
  async registerSchema(schema: OTelSchema): Promise<void> {
    return withSpan(
      'telemetry.register_schema',
      async () => {
        const response = await fetch(`${knhkConfig.telemetry.weaverRegistryUrl}/schemas`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(schema),
        });

        if (!response.ok) {
          throw new Error(`Schema registration failed: ${response.statusText}`);
        }

        const result = (await response.json()) as ApiResponse<{ schemaId: string }>;

        if (!result.success || !result.data) {
          throw new Error(result.error?.message || 'Schema registration returned no ID');
        }

        // Cache schema status
        this.schemaCache.set(result.data.schemaId, {
          id: result.data.schemaId,
          version: schema.version,
          status: 'valid',
          registered: new Date().toISOString(),
          lastChecked: new Date().toISOString(),
          errors: [],
        });
      },
      {
        'schema.url': schema.schemaUrl,
        'schema.version': schema.version,
        'schema.spans': schema.spans.length,
        'schema.metrics': schema.metrics.length,
      }
    );
  }

  /**
   * Check schema status in Weaver registry
   */
  async checkSchema(schemaId: string): Promise<SchemaStatus> {
    // Check cache first
    const cached = this.schemaCache.get(schemaId);
    if (cached) {
      const age = Date.now() - new Date(cached.lastChecked).getTime();
      if (age < 60000) {
        // 1 minute cache
        return cached;
      }
    }

    return withSpan(
      'telemetry.check_schema',
      async () => {
        const response = await fetch(
          `${knhkConfig.telemetry.weaverRegistryUrl}/schemas/${schemaId}`,
          { method: 'GET' }
        );

        if (!response.ok) {
          throw new Error(`Schema check failed: ${response.statusText}`);
        }

        const status = (await response.json()) as SchemaStatus;

        // Update cache
        this.schemaCache.set(schemaId, {
          ...status,
          lastChecked: new Date().toISOString(),
        });

        return status;
      },
      { 'schema.id': schemaId }
    );
  }

  /**
   * Run live check of runtime telemetry against schema
   *
   * This is equivalent to: weaver registry live-check --registry registry/
   */
  async liveCheck(durationSeconds = 10): Promise<{
    passed: boolean;
    spanCount: number;
    metricCount: number;
    violations: Array<{
      type: 'span' | 'metric';
      name: string;
      error: string;
    }>;
  }> {
    return withSpan(
      'telemetry.live_check',
      async () => {
        const response = await fetch(
          `${knhkConfig.telemetry.weaverRegistryUrl}/live-check`,
          {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ duration: durationSeconds }),
          }
        );

        if (!response.ok) {
          throw new Error(`Live check failed: ${response.statusText}`);
        }

        return (await response.json()) as {
          passed: boolean;
          spanCount: number;
          metricCount: number;
          violations: Array<{
            type: 'span' | 'metric';
            name: string;
            error: string;
          }>;
        };
      },
      { 'live_check.duration': durationSeconds }
    );
  }

  /* ============================
   * Metrics Export
   * ============================ */

  /**
   * Export metrics to OTLP endpoint
   *
   * Batches metrics for efficient export
   */
  async exportMetrics(metrics: Metric[]): Promise<void> {
    if (metrics.length === 0) {
      return;
    }

    return withSpan(
      'telemetry.export_metrics',
      async () => {
        // Validate metrics if schema validation is enabled
        if (knhkConfig.telemetry.schemaValidation) {
          for (const metric of metrics) {
            const validation = await this.validateMetric(metric);
            if (!validation.valid && knhkConfig.telemetry.failOnSchemaError) {
              throw new Error(
                `Metric validation failed: ${validation.errors.map((e) => e.message).join(', ')}`
              );
            }
          }
        }

        // Convert to OTLP format
        const otlpMetrics = this.convertToOTLPMetrics(metrics);

        // Export to OTLP endpoint
        const response = await fetch(getOTLPEndpoint('metrics'), {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: JSON.stringify(otlpMetrics),
          signal: AbortSignal.timeout(knhkConfig.telemetry.exportTimeout),
        });

        if (!response.ok) {
          throw new Error(`Metric export failed: ${response.statusText}`);
        }
      },
      {
        'export.count': metrics.length,
        'export.endpoint': getOTLPEndpoint('metrics'),
      }
    );
  }

  /**
   * Queue metrics for batched export
   */
  queueMetric(metric: Metric): void {
    this.pendingMetrics.push(metric);

    // Export immediately if batch size reached
    if (this.pendingMetrics.length >= 100) {
      this.flushMetrics().catch((error) => {
        console.error('Failed to flush metrics:', error);
      });
    }
  }

  /**
   * Flush pending metrics
   */
  async flushMetrics(): Promise<void> {
    if (this.pendingMetrics.length === 0) {
      return;
    }

    const metricsToExport = this.pendingMetrics.splice(0);
    await this.exportMetrics(metricsToExport);
  }

  /* ============================
   * Receipt Logging
   * ============================ */

  /**
   * Log operation receipt for audit trail
   *
   * Creates immutable record of operation and result
   */
  async logReceipt(operation: Operation, result: Result): Promise<void> {
    return withSpan(
      'telemetry.log_receipt',
      async () => {
        const receipt = {
          operation: {
            id: operation.id,
            type: operation.type,
            timestamp: operation.timestamp,
            actor: operation.actor,
            target: operation.target,
            params: operation.params,
          },
          result: {
            success: result.success,
            duration: result.duration,
            error: result.error,
            timestamp: Date.now(),
          },
          metadata: {
            service: knhkConfig.telemetry.serviceName,
            version: knhkConfig.telemetry.serviceVersion,
          },
        };

        // Send to kernel for permanent storage
        await fetch(`${knhkConfig.kernel.url}/receipts`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(receipt),
        });

        // Also emit as telemetry event
        const span = createSpan('operation.receipt', {
          'operation.id': operation.id,
          'operation.type': operation.type,
          'result.success': result.success,
          'result.duration': result.duration,
        });

        if (result.metrics) {
          result.metrics.forEach((metric) => this.queueMetric(metric));
        }

        span.end();
      },
      {
        'operation.id': operation.id,
        'operation.type': operation.type,
        'result.success': result.success,
      }
    );
  }

  /* ============================
   * Utilities
   * ============================ */

  /**
   * Convert metrics to OTLP format
   */
  private convertToOTLPMetrics(metrics: Metric[]): unknown {
    // Simplified OTLP conversion
    // In production, use official OTLP SDK
    return {
      resourceMetrics: [
        {
          resource: {
            attributes: [
              { key: 'service.name', value: { stringValue: knhkConfig.telemetry.serviceName } },
              {
                key: 'service.version',
                value: { stringValue: knhkConfig.telemetry.serviceVersion },
              },
            ],
          },
          scopeMetrics: [
            {
              scope: {
                name: 'yawl-editor',
                version: '0.1.0',
              },
              metrics: metrics.map((m) => ({
                name: m.name,
                [m.type]: {
                  dataPoints: [
                    {
                      timeUnixNano: m.timestamp * 1000000,
                      asDouble: m.value,
                      attributes: Object.entries(m.attributes).map(([key, value]) => ({
                        key,
                        value: { stringValue: String(value) },
                      })),
                    },
                  ],
                },
              })),
            },
          ],
        },
      ],
    };
  }

  /**
   * Start automatic metric export
   */
  private startExporting(): void {
    if (this.exportInterval) {
      return;
    }

    this.exportInterval = setInterval(() => {
      this.flushMetrics().catch((error) => {
        console.error('Failed to export metrics:', error);
      });
    }, knhkConfig.telemetry.exportInterval);
  }

  /**
   * Stop automatic metric export
   */
  stopExporting(): void {
    if (this.exportInterval) {
      clearInterval(this.exportInterval);
      this.exportInterval = null;
    }
  }

  /**
   * Cleanup resources
   */
  async destroy(): Promise<void> {
    this.stopExporting();
    await this.flushMetrics();
    this.schemaCache.clear();
  }
}

/* ============================================================================
 * Singleton Instance
 * ========================================================================== */

/**
 * Default telemetry bridge instance
 */
export const telemetryBridge = new TelemetryBridge();

/* ============================================================================
 * Editor Schema Definition
 * ========================================================================== */

/**
 * OpenTelemetry schema for YAWL editor
 *
 * This defines what telemetry the editor emits
 * All emitted spans/metrics MUST conform to this schema
 */
export const editorSchema: OTelSchema = {
  schemaUrl: 'https://knhk.io/schemas/yawl-editor/0.1.0',
  version: '0.1.0',
  spans: [
    {
      name: 'workflow.validate',
      attributes: {
        'workflow.id': { type: 'string', required: true },
        'workflow.name': { type: 'string', required: false },
        'workflow.nodes': { type: 'number', required: true },
        'workflow.edges': { type: 'number', required: true },
      },
      events: ['validation.started', 'validation.completed', 'validation.failed'],
    },
    {
      name: 'rdf.parse',
      attributes: {
        'rdf.format': { type: 'string', required: true },
        'rdf.size': { type: 'number', required: false },
      },
    },
    {
      name: 'pattern.validate',
      attributes: {
        'pattern.split': { type: 'string', required: true },
        'pattern.join': { type: 'string', required: true },
        'pattern.modifiers': { type: 'string', required: false },
        'pattern.valid': { type: 'boolean', required: true },
      },
    },
    {
      name: 'editor.operation',
      attributes: {
        'editor.operation': { type: 'string', required: true },
        'editor.user': { type: 'string', required: false },
      },
    },
  ],
  metrics: [
    {
      name: 'workflow.validation.duration',
      type: 'histogram',
      unit: 'ms',
      description: 'Time taken to validate workflow',
    },
    {
      name: 'rdf.parse.duration',
      type: 'histogram',
      unit: 'ms',
      description: 'Time taken to parse RDF',
    },
    {
      name: 'pattern.validation.duration',
      type: 'histogram',
      unit: 'ms',
      description: 'Time taken to validate pattern',
    },
    {
      name: 'editor.operation.count',
      type: 'counter',
      unit: '1',
      description: 'Count of editor operations',
    },
  ],
};

// Auto-register schema on module load
if (knhkConfig.telemetry.schemaValidation) {
  telemetryBridge.registerSchema(editorSchema).catch((error) => {
    console.error('Failed to register editor schema:', error);
  });
}
