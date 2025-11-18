/**
 * Advanced Components Export
 * Hyper-advanced YAWL UI components
 */

export { WorkflowGraph } from './WorkflowGraph'
export { PatternValidator } from './PatternValidator'
export { MAPEKDashboard } from './MAPEKDashboard'
export { DynamicFormBuilder } from './DynamicFormBuilder'

export default {
  WorkflowGraph: require('./WorkflowGraph').WorkflowGraph,
  PatternValidator: require('./PatternValidator').PatternValidator,
  MAPEKDashboard: require('./MAPEKDashboard').MAPEKDashboard,
  DynamicFormBuilder: require('./DynamicFormBuilder').DynamicFormBuilder,
}
