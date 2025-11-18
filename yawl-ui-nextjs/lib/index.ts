/**
 * Library Utilities Export
 * Core services and utilities for YAWL workflows
 */

export { default as WorkflowService } from './workflow-service'
export { default as RDFService } from './rdf-service'
export { performanceGuard, PerformanceGuard } from './performance-guard'
export { OntologyBuilder, createDefaultYAWLOntology } from './ontology-builder'
export { cn } from './utils'

export default {
  WorkflowService: require('./workflow-service').default,
  RDFService: require('./rdf-service').default,
  performanceGuard: require('./performance-guard').performanceGuard,
  OntologyBuilder: require('./ontology-builder').OntologyBuilder,
  createDefaultYAWLOntology: require('./ontology-builder').createDefaultYAWLOntology,
}
