/**
 * YAWL (Yet Another Workflow Language) Type Definitions
 * Comprehensive type system for YAWL workflows, tasks, and patterns
 */

export type ControlFlowPattern =
  | 'sequence'
  | 'parallel'
  | 'choice'
  | 'exclusive-choice'
  | 'implicit-choice'
  | 'deferred-choice'
  | 'interleaved-parallel'
  | 'multi-choice'
  | 'synchronization'
  | 'multiple-merge'
  | 'discriminator'

export type TaskType =
  | 'atomic'
  | 'composite'
  | 'multi-instance'
  | 'multi-instance-dynamic'

export type WorkItemStatus =
  | 'fired'
  | 'enabled'
  | 'offered'
  | 'allocated'
  | 'started'
  | 'completed'
  | 'suspended'
  | 'failed'

export type CaseStatus =
  | 'created'
  | 'launched'
  | 'running'
  | 'suspended'
  | 'completed'
  | 'cancelled'

// YAWL Specification
export interface YAWLSpecification {
  id: string
  uri: string
  name: string
  version: string
  xmlns: string
  isValid: boolean
  tasks: YAWLTask[]
  nets: YAWLNet[]
  dataTypes?: YAWLDataType[]
  metadata?: YAWLMetadata
}

export interface YAWLNet {
  id: string
  name: string
  documentation?: string
  tasks: YAWLTask[]
  flows: ControlFlow[]
  startTask?: YAWLTask
  endTask?: YAWLTask
}

// Task Definition
export interface YAWLTask {
  id: string
  name: string
  documentation?: string
  type: TaskType
  isMultiInstance?: boolean
  inputData?: YAWLDataMapping
  outputData?: YAWLDataMapping
  resourcing?: YAWLResourcing
  decomposition?: YAWLDecomposition
  preconditions?: string
  postconditions?: string
  timers?: YAWLTimer[]
  attributes?: Record<string, unknown>
}

export interface YAWLResourcing {
  allocate?: string
  start?: string
  offer?: string[]
  privileges?: string[]
  resources?: YAWLResource[]
}

export interface YAWLResource {
  id: string
  name: string
  type: 'human' | 'system' | 'shared'
  capabilities?: string[]
}

export interface YAWLDecomposition {
  id: string
  type: 'WebServiceGateway' | 'YAWLService' | 'YAWLTask'
  inputQuery?: string
  outputQuery?: string
}

export interface YAWLTimer {
  id: string
  trigger: string
  duration?: string
  expiry?: string
}

// Data Mapping
export interface YAWLDataMapping {
  variables: YAWLVariable[]
}

export interface YAWLVariable {
  name: string
  type: string
  initialValue?: unknown
  documentation?: string
  isArray?: boolean
}

// Data Types
export interface YAWLDataType {
  name: string
  restriction?: string
  baseType?: string
}

// Control Flow
export interface ControlFlow {
  id: string
  source: string
  target: string
  predicate?: string
  pattern?: ControlFlowPattern
  documentation?: string
}

// Work Item
export interface WorkItem {
  id: string
  taskID: string
  caseID: string
  specificationID: string
  status: WorkItemStatus
  resourceStatus?: string
  data: Record<string, unknown>
  enablementTime?: Date
  expiryTime?: Date
  firingTime?: Date
  startTime?: Date
  completionTime?: Date
  documentation?: string
}

// Case / Workflow Instance
export interface WorkflowCase {
  id: string
  specificationID: string
  status: CaseStatus
  launchTime: Date
  completionTime?: Date
  workItems: WorkItem[]
  data: Record<string, unknown>
  parentCaseID?: string
  childCases?: WorkflowCase[]
  metadata?: CaseMetadata
}

export interface CaseMetadata {
  initiator?: string
  priority?: number
  dueDate?: Date
  tags?: string[]
  customFields?: Record<string, unknown>
}

// Participant / Resource
export interface Participant {
  id: string
  userid: string
  password?: string
  fullName?: string
  email?: string
  roles?: string[]
  capabilities?: string[]
  privileges?: string[]
  calendar?: ResourceCalendar
}

export interface ResourceCalendar {
  id: string
  participantID: string
  workingDays: DayOfWeek[]
  workingHours?: TimeRange
  holidays?: Date[]
}

export type DayOfWeek = 'MON' | 'TUE' | 'WED' | 'THU' | 'FRI' | 'SAT' | 'SUN'

export interface TimeRange {
  startTime: string
  endTime: string
}

// Metadata
export interface YAWLMetadata {
  title?: string
  creator?: string
  description?: string
  version?: string
  dateCreated?: Date
  dateModified?: Date
  keywords?: string[]
  customProperties?: Record<string, unknown>
}

// Pattern-related types
export interface PatternInstance {
  id: string
  patternType: ControlFlowPattern
  participants: string[]
  conditions?: PatternCondition[]
}

export interface PatternCondition {
  type: string
  expression: string
}

// Network Instance
export interface YAWLNetworkInstance {
  id: string
  name: string
  tasks: YAWLTask[]
  flows: ControlFlow[]
  documentation?: string
}

// RDF/Turtle representation
export interface YAWLRDFModel {
  resourceIRI: string
  triples: RDFTriple[]
  ontologyIRI?: string
}

export interface RDFTriple {
  subject: string
  predicate: string
  object: string | RDFObject
}

export interface RDFObject {
  value: string
  type: 'literal' | 'uri' | 'bnode'
  datatype?: string
}

// Workflow Graph Node for visualization
export interface WorkflowNode {
  id: string
  label: string
  type: 'task' | 'start' | 'end' | 'gateway'
  position: { x: number; y: number }
  data: YAWLTask | { label: string }
  style?: NodeStyle
}

export interface WorkflowEdge {
  id: string
  source: string
  target: string
  label?: string
  pattern?: ControlFlowPattern
  style?: EdgeStyle
}

export interface NodeStyle {
  width?: number
  height?: number
  backgroundColor?: string
  borderColor?: string
  fontSize?: number
}

export interface EdgeStyle {
  strokeColor?: string
  strokeWidth?: number
}

// Import/Export types
export interface ImportOptions {
  format: 'xml' | 'turtle' | 'json'
  validate?: boolean
  mergeExisting?: boolean
}

export interface ExportOptions {
  format: 'xml' | 'turtle' | 'json'
  includeMetadata?: boolean
  prettyPrint?: boolean
}

// Validation Result
export interface ValidationResult {
  isValid: boolean
  errors: ValidationError[]
  warnings: ValidationWarning[]
  statistics?: ValidationStatistics
}

export interface ValidationError {
  code: string
  message: string
  location?: string
  severity: 'error' | 'warning'
}

export interface ValidationWarning {
  code: string
  message: string
  suggestion?: string
}

export interface ValidationStatistics {
  totalTasks: number
  totalConnections: number
  totalResources: number
  complexityScore: number
}
