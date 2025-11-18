/**
 * DOCTRINE ALIGNMENT: Σ (System Integrity)
 * Configuration for knhk ecosystem integration
 *
 * All configuration is externalized for environment-specific deployments
 */

/* ============================================================================
 * Environment Variables
 * ========================================================================== */

const getEnv = (key: string, defaultValue: string): string => {
  if (typeof process !== 'undefined' && process.env) {
    return process.env[key] || defaultValue;
  }
  return defaultValue;
};

const getEnvBool = (key: string, defaultValue: boolean): boolean => {
  if (typeof process !== 'undefined' && process.env) {
    const value = process.env[key];
    if (value === undefined) return defaultValue;
    return value === 'true' || value === '1';
  }
  return defaultValue;
};

const getEnvNumber = (key: string, defaultValue: number): number => {
  if (typeof process !== 'undefined' && process.env) {
    const value = process.env[key];
    if (value === undefined) return defaultValue;
    const parsed = parseInt(value, 10);
    return isNaN(parsed) ? defaultValue : parsed;
  }
  return defaultValue;
};

/* ============================================================================
 * Configuration Object
 * ========================================================================== */

export const knhkConfig = {
  /* ============================
   * knhk-kernel Connection
   * ============================ */
  kernel: {
    url: getEnv('KNHK_KERNEL_URL', 'http://localhost:8080'),
    apiVersion: 'v1',
    timeout: getEnvNumber('KNHK_KERNEL_TIMEOUT', 30000), // ms
    retries: getEnvNumber('KNHK_KERNEL_RETRIES', 3),
    retryDelay: getEnvNumber('KNHK_KERNEL_RETRY_DELAY', 1000), // ms
    retryBackoff: 2, // exponential backoff multiplier
  },

  /* ============================
   * MAPE-K Settings
   * ============================ */
  mapek: {
    enabled: getEnvBool('MAPE_K_ENABLED', true),
    interval: getEnvNumber('MAPE_K_INTERVAL', 5000), // ms
    analysisTimeout: getEnvNumber('MAPE_K_ANALYSIS_TIMEOUT', 10000), // ms
    autoApply: getEnvBool('MAPE_K_AUTO_APPLY', false), // require user confirmation by default
    learningEnabled: getEnvBool('MAPE_K_LEARNING_ENABLED', true),
  },

  /* ============================
   * Pattern Matrix
   * ============================ */
  patternMatrix: {
    url: getEnv(
      'PATTERN_MATRIX_URL',
      '/ontology/yawl-pattern-permutations.ttl'
    ),
    cacheTTL: getEnvNumber('PATTERN_MATRIX_CACHE_TTL', 3600), // seconds
    validateOnLoad: getEnvBool('PATTERN_MATRIX_VALIDATE', true),
    pollInterval: getEnvNumber('PATTERN_MATRIX_POLL_INTERVAL', 60000), // ms (development)
    watchEnabled: getEnvBool('PATTERN_MATRIX_WATCH', false), // file watching in development
  },

  /* ============================
   * Telemetry
   * ============================ */
  telemetry: {
    // OTLP endpoint for metrics/traces/logs
    otlpEndpoint: getEnv(
      'OTEL_EXPORTER_OTLP_ENDPOINT',
      'http://localhost:4318'
    ),

    // Weaver registry for schema validation
    weaverRegistryUrl: getEnv(
      'WEAVER_REGISTRY_URL',
      'http://localhost:8090'
    ),

    // Service identification
    serviceName: getEnv('OTEL_SERVICE_NAME', 'yawl-editor'),
    serviceVersion: getEnv('OTEL_SERVICE_VERSION', '0.1.0'),

    // Export settings
    exportInterval: getEnvNumber('OTEL_EXPORT_INTERVAL', 5000), // ms
    exportTimeout: getEnvNumber('OTEL_EXPORT_TIMEOUT', 3000), // ms

    // Sampling
    samplingRate: parseFloat(getEnv('OTEL_SAMPLING_RATE', '1.0')), // 0.0 to 1.0

    // Schema validation
    schemaValidation: getEnvBool('OTEL_SCHEMA_VALIDATION', true),
    failOnSchemaError: getEnvBool('OTEL_FAIL_ON_SCHEMA_ERROR', false),
  },

  /* ============================
   * Performance Budgets (Chatman Constant)
   * ============================ */
  performance: {
    // All times in milliseconds

    // Hot path budget: ≤8 ticks (Chatman Constant)
    hotPathBudget: 8, // ms - from DOCTRINE

    // Validation budget (Q4 constraint)
    validationBudget: getEnvNumber('PERF_VALIDATION_BUDGET', 100), // ms

    // Export to knhk format
    exportBudget: getEnvNumber('PERF_EXPORT_BUDGET', 200), // ms

    // Pattern matrix sync
    syncBudget: getEnvNumber('PERF_SYNC_BUDGET', 50), // ms

    // RDF parsing
    parseBudget: getEnvNumber('PERF_PARSE_BUDGET', 150), // ms

    // MAPE-K analysis (async, so more lenient)
    analysisBudget: getEnvNumber('PERF_ANALYSIS_BUDGET', 500), // ms

    // Pattern matrix load (one-time, cached)
    matrixLoadBudget: getEnvNumber('PERF_MATRIX_LOAD_BUDGET', 500), // ms
  },

  /* ============================
   * Workflow Exchange
   * ============================ */
  exchange: {
    // Default RDF format
    defaultRdfFormat: getEnv('RDF_DEFAULT_FORMAT', 'turtle') as 'turtle' | 'ntriples' | 'jsonld',

    // Preserve metadata through conversions
    preserveMetadata: getEnvBool('RDF_PRESERVE_METADATA', true),

    // Create audit trail
    auditEnabled: getEnvBool('AUDIT_ENABLED', true),

    // Versioning
    versioningEnabled: getEnvBool('VERSIONING_ENABLED', true),
    versionFormat: getEnv('VERSION_FORMAT', 'semver'), // semver | timestamp | sequential
  },

  /* ============================
   * Development & Debugging
   * ============================ */
  dev: {
    // Enable debug logging
    debug: getEnvBool('DEBUG', false),

    // Log all API requests
    logRequests: getEnvBool('LOG_REQUESTS', false),

    // Log all telemetry spans
    logTelemetry: getEnvBool('LOG_TELEMETRY', false),

    // Mock kernel responses (for offline development)
    mockKernel: getEnvBool('MOCK_KERNEL', false),

    // Disable schema validation (not recommended)
    disableSchemaValidation: getEnvBool('DISABLE_SCHEMA_VALIDATION', false),
  },

  /* ============================
   * Feature Flags
   * ============================ */
  features: {
    // Enable MAPE-K autonomous optimization
    autonomicOptimization: getEnvBool('FEATURE_AUTONOMIC_OPTIMIZATION', true),

    // Enable real-time kernel sync
    realtimeSync: getEnvBool('FEATURE_REALTIME_SYNC', false),

    // Enable pattern learning from execution
    patternLearning: getEnvBool('FEATURE_PATTERN_LEARNING', true),

    // Enable execution replay
    executionReplay: getEnvBool('FEATURE_EXECUTION_REPLAY', true),

    // Enable collaborative editing (future)
    collaborative: getEnvBool('FEATURE_COLLABORATIVE', false),
  },
} as const;

/* ============================================================================
 * Computed Values
 * ========================================================================== */

/**
 * Full API base URL for kernel
 */
export const getKernelApiUrl = (endpoint: string): string => {
  const { url, apiVersion } = knhkConfig.kernel;
  const base = url.endsWith('/') ? url.slice(0, -1) : url;
  const path = endpoint.startsWith('/') ? endpoint : `/${endpoint}`;
  return `${base}/${apiVersion}${path}`;
};

/**
 * Full OTLP endpoint URL
 */
export const getOTLPEndpoint = (signal: 'traces' | 'metrics' | 'logs'): string => {
  const base = knhkConfig.telemetry.otlpEndpoint;
  return `${base}/v1/${signal}`;
};

/**
 * Check if we're in development mode
 */
export const isDevelopment = (): boolean => {
  return getEnv('NODE_ENV', 'development') === 'development';
};

/**
 * Check if we're in production mode
 */
export const isProduction = (): boolean => {
  return getEnv('NODE_ENV', 'development') === 'production';
};

/* ============================================================================
 * Configuration Validation
 * ========================================================================== */

/**
 * Validate configuration at startup
 * Throws error if critical configuration is missing or invalid
 */
export function validateConfig(): void {
  const errors: string[] = [];

  // Validate kernel URL
  if (!knhkConfig.kernel.url) {
    errors.push('KNHK_KERNEL_URL is required');
  }

  // Validate performance budgets
  if (knhkConfig.performance.hotPathBudget > 10) {
    errors.push('Hot path budget exceeds Chatman Constant (8 ticks)');
  }

  // Validate sampling rate
  if (
    knhkConfig.telemetry.samplingRate < 0 ||
    knhkConfig.telemetry.samplingRate > 1
  ) {
    errors.push('OTEL_SAMPLING_RATE must be between 0.0 and 1.0');
  }

  if (errors.length > 0) {
    throw new Error(
      `Configuration validation failed:\n${errors.map((e) => `  - ${e}`).join('\n')}`
    );
  }
}

/* ============================================================================
 * Export Types
 * ========================================================================== */

export type KNHKConfig = typeof knhkConfig;
