/**
 * KNHK WebAssembly Workflow Engine
 *
 * High-level TypeScript API for executing KNHK workflows in WASM environments.
 *
 * @example
 * ```typescript
 * import { WorkflowEngine, WorkflowDefinition } from '@knhk/wasm';
 *
 * const engine = await WorkflowEngine.create();
 * const result = await engine.execute(workflowDef, inputData);
 * console.log('Result:', result);
 * ```
 */

import init, {
  WasmWorkflowEngine,
  WasmEngineConfig,
  HostFunctionRegistry,
} from './knhk_wasm';

export * from './knhk_wasm';

/**
 * Workflow definition in JSON format
 */
export interface WorkflowDefinition {
  /** Unique workflow identifier */
  id: string;
  /** Workflow pattern (Sequence, Parallel, Choice, Loop) */
  pattern: 'Sequence' | 'Parallel' | 'Choice' | 'Loop';
  /** List of tasks in the workflow */
  tasks: Task[];
  /** Loop condition (for Loop pattern) */
  loopCondition?: any;
}

/**
 * Task definition
 */
export interface Task {
  /** Unique task identifier */
  id: string;
  /** Task type */
  type: 'transform' | 'validate' | 'compute' | string;
  /** Task configuration */
  config?: any;
  /** Conditional execution condition */
  condition?: any;
}

/**
 * Workflow execution result
 */
export interface WorkflowResult {
  /** Case ID */
  caseId: string;
  /** Execution status */
  status: 'completed' | 'failed' | 'running';
  /** Output data */
  output: any;
  /** Execution time in milliseconds */
  executionTimeMs: number;
}

/**
 * Engine statistics
 */
export interface EngineStats {
  /** Total workflows executed */
  totalExecuted: number;
  /** Currently running workflows */
  runningWorkflows: number;
  /** Failed workflows */
  failedWorkflows: number;
  /** Average execution time in milliseconds */
  avgExecutionTimeMs: number;
  /** Approximate memory usage in bytes */
  memoryUsageBytes: number;
}

/**
 * Engine configuration options
 */
export interface EngineConfig {
  /** Maximum number of concurrent workflows */
  maxWorkflows?: number;
  /** Enable telemetry logging */
  enableTelemetry?: boolean;
  /** Workflow execution timeout in milliseconds */
  timeoutMs?: number;
}

/**
 * Host function callback type
 */
export type HostFunction = (args: any) => Promise<any> | any;

/**
 * High-level workflow engine API
 */
export class WorkflowEngine {
  private wasmEngine: WasmWorkflowEngine | null = null;
  private hostRegistry: HostFunctionRegistry | null = null;
  private initialized = false;

  private constructor() {}

  /**
   * Create a new workflow engine instance
   *
   * @param config - Engine configuration options
   * @returns Initialized workflow engine
   */
  static async create(config?: EngineConfig): Promise<WorkflowEngine> {
    const engine = new WorkflowEngine();
    await engine.initialize(config);
    return engine;
  }

  /**
   * Initialize the WASM module
   */
  private async initialize(config?: EngineConfig): Promise<void> {
    if (this.initialized) {
      return;
    }

    // Initialize WASM module
    await init();

    // Create engine config
    const wasmConfig = new WasmEngineConfig();
    if (config?.maxWorkflows !== undefined) {
      wasmConfig.set_max_workflows(config.maxWorkflows);
    }
    if (config?.enableTelemetry !== undefined) {
      wasmConfig.set_enable_telemetry(config.enableTelemetry);
    }
    if (config?.timeoutMs !== undefined) {
      wasmConfig.set_timeout_ms(config.timeoutMs);
    }

    // Create WASM engine
    this.wasmEngine = WasmWorkflowEngine.with_config(wasmConfig);

    // Create host function registry
    this.hostRegistry = new HostFunctionRegistry();

    this.initialized = true;
  }

  /**
   * Execute a workflow
   *
   * @param workflow - Workflow definition (JSON object or Turtle string)
   * @param input - Input data for the workflow
   * @returns Workflow execution result
   */
  async execute(
    workflow: WorkflowDefinition | string,
    input: any = {}
  ): Promise<any> {
    this.ensureInitialized();

    if (typeof workflow === 'string') {
      // Assume Turtle format
      return await this.wasmEngine!.execute_workflow(workflow, input);
    } else {
      // JSON format
      const workflowJson = JSON.stringify(workflow);
      return await this.wasmEngine!.execute_workflow_json(workflowJson, input);
    }
  }

  /**
   * Validate a workflow definition
   *
   * @param workflow - Workflow definition to validate
   * @returns True if valid, throws error otherwise
   */
  validate(workflow: WorkflowDefinition | string): boolean {
    this.ensureInitialized();

    const workflowStr = typeof workflow === 'string'
      ? workflow
      : JSON.stringify(workflow);

    return this.wasmEngine!.validate_workflow(workflowStr);
  }

  /**
   * Register a host function
   *
   * @param name - Function name
   * @param func - Function implementation
   */
  registerHostFunction(name: string, func: HostFunction): void {
    this.ensureInitialized();

    const jsFunc = async (args: any) => {
      try {
        return await func(args);
      } catch (error) {
        console.error(`Host function '${name}' failed:`, error);
        throw error;
      }
    };

    this.hostRegistry!.register(name, jsFunc as any);
  }

  /**
   * Call a registered host function
   *
   * @param name - Function name
   * @param args - Function arguments
   * @returns Function result
   */
  async callHostFunction(name: string, args: any): Promise<any> {
    this.ensureInitialized();
    return await this.hostRegistry!.call(name, args);
  }

  /**
   * Get engine statistics
   *
   * @returns Current engine statistics
   */
  getStats(): EngineStats {
    this.ensureInitialized();
    return this.wasmEngine!.get_stats();
  }

  /**
   * Reset the engine state
   */
  reset(): void {
    this.ensureInitialized();
    this.wasmEngine!.reset();
  }

  /**
   * Get the WASM module version
   */
  static getVersion(): string {
    return WasmWorkflowEngine.version();
  }

  private ensureInitialized(): void {
    if (!this.initialized || !this.wasmEngine) {
      throw new Error('WorkflowEngine not initialized. Call WorkflowEngine.create() first.');
    }
  }
}

/**
 * Convenience function to create and execute a workflow in one call
 *
 * @param workflow - Workflow definition
 * @param input - Input data
 * @param config - Engine configuration
 * @returns Workflow execution result
 */
export async function executeWorkflow(
  workflow: WorkflowDefinition | string,
  input: any = {},
  config?: EngineConfig
): Promise<any> {
  const engine = await WorkflowEngine.create(config);
  return await engine.execute(workflow, input);
}

/**
 * Convenience function to validate a workflow
 *
 * @param workflow - Workflow definition
 * @returns True if valid, throws error otherwise
 */
export async function validateWorkflow(
  workflow: WorkflowDefinition | string
): Promise<boolean> {
  const engine = await WorkflowEngine.create();
  return engine.validate(workflow);
}
