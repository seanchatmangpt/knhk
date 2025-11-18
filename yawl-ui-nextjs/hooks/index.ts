/**
 * Advanced Hooks Export
 * Hyper-advanced custom React hooks for YAWL workflows
 */

export { useWorkflow } from './useWorkflow'
export { useRDFOntology } from './useRDFOntology'
export { useMAPEK } from './useMAPEK'
export { usePatternValidator } from './usePatternValidator'

export default {
  useWorkflow: require('./useWorkflow').useWorkflow,
  useRDFOntology: require('./useRDFOntology').useRDFOntology,
  useMAPEK: require('./useMAPEK').useMAPEK,
  usePatternValidator: require('./usePatternValidator').usePatternValidator,
}
