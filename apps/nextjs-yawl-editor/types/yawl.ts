/**
 * YAWL TypeScript Type Definitions
 * Generated from YAWL 4.0 RDF Ontology
 *
 * DOCTRINE Alignment: Covenant 1 (Turtle is source of truth)
 * These types are derived from the RDF ontology and represent
 * the canonical YAWL structures.
 */

import type * as RDF from '@rdfjs/types';

// ============================================================================
// Enumerations (from RDF ontology)
// ============================================================================

export type SplitType = 'XOR' | 'OR' | 'AND';
export type JoinType = 'XOR' | 'OR' | 'AND' | 'Discriminator';
export type ControlType = SplitType | JoinType;

export type CreationMode = 'Static' | 'Dynamic';

export type TimerInterval =
  | 'YEAR' | 'MONTH' | 'WEEK' | 'DAY'
  | 'HOUR' | 'MIN' | 'SEC' | 'MSEC';

export type TimerTrigger = 'OnEnabled' | 'OnExecuting';

export type ResourcingInteraction = 'Manual' | 'Automated';

export type CancellationType = 'CancelTask' | 'CancelCase' | 'CancelRegion';

export type IterationType = 'StructuredLoop' | 'Recursion';

// ============================================================================
// Core YAWL Entities
// ============================================================================

/**
 * Top-level YAWL Specification
 * Maps to yawl:Specification in RDF
 */
export interface Specification {
  uri: string;
  name: string;
  documentation?: string;
  version?: string;
  metaData?: MetaData;
  decompositions: Decomposition[];
}

/**
 * Base type for decompositions (Net or WebServiceDecomposition)
 */
export type Decomposition = Net | WebServiceDecomposition;

/**
 * YAWL Net (workflow decomposition)
 * Maps to yawl:Net in RDF
 */
export interface Net {
  type: 'Net';
  id: string;
  name: string;
  isRootNet: boolean;
  inputCondition: InputCondition;
  outputCondition: OutputCondition;
  tasks: Task[];
  conditions: Condition[];
  localVariables?: Variable[];
}

/**
 * YAWL Task
 * Maps to yawl:Task in RDF
 */
export interface Task {
  id: string;
  name: string;
  documentation?: string;

  // Control flow
  splitType: SplitType;
  joinType: JoinType;

  // Flows
  flowsInto: FlowsInto[];

  // Decomposition
  decomposition?: Decomposition;

  // Multi-instance
  isMultiInstance?: boolean;
  multiInstanceConfig?: MultiInstanceConfig;

  // Timers
  timer?: Timer;

  // Resourcing
  resourcing?: Resourcing;

  // Data
  inputMappings?: DataMapping[];
  outputMappings?: DataMapping[];

  // Cancellation
  cancellationSet?: string[];  // IDs of tasks/regions to cancel
}

/**
 * Flow between elements
 * Maps to yawl:FlowsInto in RDF
 */
export interface FlowsInto {
  id: string;
  source: string;  // Task or Condition ID
  target: string;  // Task or Condition ID

  // Conditional flow (for XOR/OR splits)
  predicate?: string;  // XPath expression
  ordering?: number;   // Flow ordering for OR splits

  // Default flow
  isDefaultFlow?: boolean;
}

/**
 * Condition (intermediate state)
 * Maps to yawl:Condition in RDF
 */
export interface Condition {
  id: string;
  name: string;
}

/**
 * Input Condition (start)
 * Maps to yawl:InputCondition in RDF
 */
export interface InputCondition extends Condition {
  type: 'InputCondition';
  flowsInto: FlowsInto[];
}

/**
 * Output Condition (end)
 * Maps to yawl:OutputCondition in RDF
 */
export interface OutputCondition extends Condition {
  type: 'OutputCondition';
}

/**
 * Multi-instance configuration
 */
export interface MultiInstanceConfig {
  minInstances: number;
  maxInstances: number;
  threshold: number;  // For threshold join
  creationMode: CreationMode;

  // Dynamic instance creation
  miInputQuery?: string;  // XQuery for dynamic instance data
  miOutputQuery?: string;

  // Join behavior
  miJoinThreshold?: number;
}

/**
 * Timer configuration
 */
export interface Timer {
  trigger: TimerTrigger;
  duration: number;
  interval: TimerInterval;

  // Expiry action
  expiryAction?: 'CancelTask' | 'Notify' | 'Escalate';
}

/**
 * Resourcing specification
 */
export interface Resourcing {
  interaction: ResourcingInteraction;

  // Allocation (who can do this task)
  offer?: ResourceFilter;
  allocate?: ResourceFilter;

  // Runtime behavior
  startInteraction?: ResourcingInteraction;
  privileges?: string[];
}

/**
 * Resource filter (role, capability, etc.)
 */
export interface ResourceFilter {
  type: 'Role' | 'Capability' | 'Position' | 'OrgGroup';
  value: string;
}

/**
 * Data mapping (input/output)
 */
export interface DataMapping {
  source: string;  // XPath/XQuery expression
  target: string;  // Variable name
  expression?: string;
}

/**
 * Local variable definition
 */
export interface Variable {
  name: string;
  type: string;  // XML Schema type
  initialValue?: string;
  namespace?: string;
}

/**
 * Web service decomposition
 */
export interface WebServiceDecomposition {
  type: 'WebServiceDecomposition';
  id: string;
  name: string;
  wsdlUrl?: string;
  operation?: string;
}

/**
 * Specification metadata
 */
export interface MetaData {
  title?: string;
  creator?: string;
  description?: string;
  version?: string;
  created?: string;  // ISO 8601 date
  lastModified?: string;
}

// ============================================================================
// Pattern Validation Types (Covenant 4)
// ============================================================================

/**
 * Pattern combination from permutation matrix
 * Maps to yawl:SplitJoinCombination in RDF
 */
export interface PatternCombination {
  splitType: SplitType;
  joinType: JoinType;
  modifiers?: PatternModifiers;
  isValid: boolean;
  supportedPatterns: string[];
}

/**
 * Pattern modifiers (beyond basic split/join)
 */
export interface PatternModifiers {
  requiresFlowPredicate?: boolean;
  requiresQuorum?: boolean;
  requiresBackwardFlow?: boolean;
  requiresDeferredChoice?: boolean;
  requiresInterleaving?: boolean;
  requiresCriticalSection?: boolean;
  requiresMilestone?: boolean;
  requiresCancellation?: boolean;
  requiresIteration?: boolean;
}

// ============================================================================
// Validation Types (Covenant 2)
// ============================================================================

/**
 * Validation error
 */
export interface ValidationError {
  code: string;
  message: string;
  covenant?: string;  // DOCTRINE covenant reference
  severity: 'error' | 'warning';
  location?: {
    taskId?: string;
    flowId?: string;
    property?: string;
  };
}

/**
 * Validation result
 */
export interface ValidationResult {
  isValid: boolean;
  errors: ValidationError[];
  warnings: ValidationError[];
  duration: number;  // Validation latency (Q4 constraint)
}

// ============================================================================
// RDF Conversion Types
// ============================================================================

/**
 * RDF namespace URIs
 */
export const YAWL_NAMESPACES = {
  yawl: 'http://www.yawlfoundation.org/yawlschema#',
  rdf: 'http://www.w3.org/1999/02/22-rdf-syntax-ns#',
  rdfs: 'http://www.w3.org/2000/01/rdf-schema#',
  xsd: 'http://www.w3.org/2001/XMLSchema#',
  bitflow: 'http://bitflow.ai/ontology/yawl/v2#',
} as const;

/**
 * Type for RDF quad with YAWL context
 */
export type YAWLQuad = RDF.Quad;

/**
 * Type for RDF dataset
 */
export type YAWLDataset = RDF.DatasetCore;

// ============================================================================
// Editor UI Types
// ============================================================================

/**
 * Node for React Flow canvas
 */
export interface WorkflowNode {
  id: string;
  type: 'task' | 'inputCondition' | 'outputCondition' | 'condition';
  position: { x: number; y: number };
  data: Task | Condition;
}

/**
 * Edge for React Flow canvas
 */
export interface WorkflowEdge {
  id: string;
  source: string;
  target: string;
  type?: 'default' | 'conditional';
  data?: FlowsInto;
}

/**
 * Editor state
 */
export interface EditorState {
  // RDF state (source of truth - Covenant 1)
  rdf: {
    dataset: YAWLDataset;
    history: YAWLDataset[];
    historyIndex: number;
  };

  // Derived UI state
  ui: {
    selectedNodeId: string | null;
    selectedEdgeId: string | null;
    viewport: { x: number; y: number; zoom: number };
    panelOpen: boolean;
    mode: 'edit' | 'view' | 'validate';
  };

  // Validation state (Q invariants - Covenant 2)
  validation: {
    results: Map<string, ValidationResult>;
    isValidating: boolean;
    lastValidation: number;  // timestamp
  };

  // Telemetry state (observations - Covenant 6)
  telemetry: {
    sessionId: string;
    operations: Operation[];
  };
}

/**
 * User operation (for telemetry)
 */
export interface Operation {
  id: string;
  type: 'create' | 'update' | 'delete' | 'validate' | 'export';
  timestamp: number;
  target: {
    type: 'task' | 'condition' | 'flow' | 'workflow';
    id: string;
  };
  duration?: number;
  result?: 'success' | 'error';
}

// ============================================================================
// Telemetry Types (Covenant 6)
// ============================================================================

/**
 * OpenTelemetry span attributes for YAWL editor
 */
export interface YAWLTelemetryAttributes {
  // Task operations
  'yawl.task.id'?: string;
  'yawl.task.name'?: string;
  'yawl.task.split_type'?: SplitType;
  'yawl.task.join_type'?: JoinType;

  // Validation
  'validation.is_valid'?: boolean;
  'validation.error_count'?: number;
  'validation.duration_ms'?: number;
  'validation.covenant_violations'?: string[];

  // Performance (Q4)
  'operation'?: string;
  'latency_budget_ms'?: number;
  'exceeds_budget'?: boolean;

  // Editor context
  'editor.action'?: string;
  'editor.session.id'?: string;
  'editor.workflow.id'?: string;
}

// ============================================================================
// SPARQL Query Types
// ============================================================================

/**
 * SPARQL query result binding
 */
export interface SPARQLBinding {
  [variable: string]: RDF.Term;
}

/**
 * SPARQL query result
 */
export interface SPARQLResult<T = SPARQLBinding> {
  results: {
    bindings: T[];
  };
}

/**
 * Pre-defined SPARQL queries
 */
export type SPARQLQueryType =
  | 'getTasks'
  | 'getFlows'
  | 'getTask'
  | 'validatePattern'
  | 'getPatternCombinations';
