/**
 * Workflow Service
 * Handles workflow creation, manipulation, and validation
 */

import type {
  YAWLSpecification,
  YAWLTask,
  ControlFlow,
  WorkflowNode,
  WorkflowEdge,
  ControlFlowPattern,
  ValidationResult,
  ValidationError,
} from '@/types/yawl'

export class WorkflowService {
  /**
   * Create a new workflow specification
   */
  static createSpecification(
    id: string,
    name: string,
    version = '1.0'
  ): YAWLSpecification {
    return {
      id,
      uri: `http://example.com/workflow/${id}`,
      name,
      version,
      xmlns: 'http://www.yawlfoundation.org/yawl/',
      isValid: false,
      tasks: [],
      nets: [],
      metadata: {
        title: name,
        dateCreated: new Date(),
        dateModified: new Date(),
      },
    }
  }

  /**
   * Add a task to workflow
   */
  static addTask(spec: YAWLSpecification, task: YAWLTask): YAWLSpecification {
    return {
      ...spec,
      tasks: [...spec.tasks, task],
      metadata: {
        ...spec.metadata,
        dateModified: new Date(),
      },
    }
  }

  /**
   * Remove a task from workflow
   */
  static removeTask(spec: YAWLSpecification, taskId: string): YAWLSpecification {
    return {
      ...spec,
      tasks: spec.tasks.filter((t) => t.id !== taskId),
      metadata: {
        ...spec.metadata,
        dateModified: new Date(),
      },
    }
  }

  /**
   * Add control flow between tasks
   */
  static addControlFlow(
    spec: YAWLSpecification,
    sourceId: string,
    targetId: string,
    pattern?: ControlFlowPattern
  ): YAWLSpecification {
    const flow: ControlFlow = {
      id: `flow-${Date.now()}`,
      source: sourceId,
      target: targetId,
      pattern,
    }

    const net = spec.nets[0]
    if (!net) return spec

    return {
      ...spec,
      nets: [
        {
          ...net,
          flows: [...(net.flows || []), flow],
        },
        ...spec.nets.slice(1),
      ],
    }
  }

  /**
   * Convert workflow to graph representation for visualization
   */
  static toGraphRepresentation(
    spec: YAWLSpecification
  ): { nodes: WorkflowNode[]; edges: WorkflowEdge[] } {
    const nodes: WorkflowNode[] = spec.tasks.map((task, idx) => ({
      id: task.id,
      label: task.name,
      type: 'task',
      position: { x: 100 + idx * 200, y: 100 },
      data: task,
    }))

    const edges: WorkflowEdge[] = spec.nets
      .flatMap((net) => net.flows || [])
      .map((flow) => ({
        id: flow.id,
        source: flow.source,
        target: flow.target,
        label: flow.predicate,
        pattern: flow.pattern,
      }))

    return { nodes, edges }
  }

  /**
   * Validate workflow specification
   */
  static validate(spec: YAWLSpecification): ValidationResult {
    const errors: ValidationError[] = []

    // Check for empty specification
    if (spec.tasks.length === 0) {
      errors.push({
        code: 'EMPTY_WORKFLOW',
        message: 'Workflow contains no tasks',
        severity: 'error',
      })
    }

    // Check for orphaned tasks
    const net = spec.nets[0]
    if (net) {
      const connectedTasks = new Set<string>()
      net.flows?.forEach((flow) => {
        connectedTasks.add(flow.source)
        connectedTasks.add(flow.target)
      })

      spec.tasks.forEach((task) => {
        if (!connectedTasks.has(task.id)) {
          errors.push({
            code: 'ORPHANED_TASK',
            message: `Task "${task.name}" is not connected to workflow`,
            location: task.id,
            severity: 'warning',
          })
        }
      })
    }

    // Check for task naming
    const duplicateNames = spec.tasks
      .map((t) => t.name)
      .filter((name, index, arr) => arr.indexOf(name) !== index)

    duplicateNames.forEach((name) => {
      errors.push({
        code: 'DUPLICATE_NAME',
        message: `Multiple tasks with name "${name}"`,
        severity: 'warning',
      })
    })

    return {
      isValid: errors.filter((e) => e.severity === 'error').length === 0,
      errors: errors.filter((e) => e.severity === 'error'),
      warnings: errors.filter((e) => e.severity === 'warning'),
      statistics: {
        totalTasks: spec.tasks.length,
        totalConnections: net?.flows?.length || 0,
        totalResources: 0,
        complexityScore: this.calculateComplexity(spec),
      },
    }
  }

  /**
   * Calculate workflow complexity score
   */
  private static calculateComplexity(spec: YAWLSpecification): number {
    const taskScore = spec.tasks.length
    const connectionScore = spec.nets.reduce((sum, net) => sum + (net.flows?.length || 0), 0)
    const patternScore = spec.nets.reduce((sum, net) => {
      return sum + (net.flows?.filter((f) => f.pattern).length || 0)
    }, 0)

    return taskScore * 1 + connectionScore * 0.5 + patternScore * 0.3
  }

  /**
   * Find pattern violations
   */
  static findPatternViolations(spec: YAWLSpecification): ValidationError[] {
    const errors: ValidationError[] = []

    // Check for invalid pattern combinations
    const net = spec.nets[0]
    if (!net) return errors

    // Simple validation: parallel tasks need synchronization
    const parallelFlows = net.flows?.filter((f) => f.pattern === 'parallel') || []
    if (parallelFlows.length > 0) {
      const syncFlows = net.flows?.filter((f) => f.pattern === 'synchronization') || []
      if (syncFlows.length === 0) {
        errors.push({
          code: 'MISSING_SYNC',
          message: 'Parallel tasks require synchronization pattern',
          severity: 'error',
        })
      }
    }

    return errors
  }

  /**
   * Generate recommended patterns
   */
  static getRecommendedPatterns(taskCount: number): ControlFlowPattern[] {
    const patterns: ControlFlowPattern[] = ['sequence']

    if (taskCount > 2) {
      patterns.push('parallel', 'choice')
    }

    if (taskCount > 5) {
      patterns.push('multi-instance', 'synchronization')
    }

    return patterns
  }
}

export default WorkflowService
