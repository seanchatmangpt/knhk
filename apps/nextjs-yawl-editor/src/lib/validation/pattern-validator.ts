/**
 * DOCTRINE ALIGNMENT: Q (Hard Invariants)
 * YAWL pattern validation against permutation matrix
 *
 * COVENANT 2: Invariants Are Law
 * All patterns MUST conform to the YAWL permutation matrix
 * Invalid patterns are REJECTED, not warned
 */

import type { YAWLWorkflow, ValidationResult, PatternType } from '@/lib/types';

/**
 * Validate workflow against YAWL pattern permutation matrix
 *
 * CRITICAL: This is a hard gate. Invalid patterns MUST be rejected.
 * Performance requirement: ≤8 ticks (Chatman Constant)
 */
export function validateWorkflow(workflow: YAWLWorkflow): ValidationResult {
  const errors: ValidationResult['errors'] = [];
  const warnings: ValidationResult['warnings'] = [];

  // Validation 1: Every workflow must have exactly one start node
  const startNodes = workflow.nodes.filter((node: YAWLWorkflow['nodes'][0]) => node.type === 'start');
  if (startNodes.length === 0) {
    errors.push({
      code: 'NO_START_NODE',
      message: 'Workflow must have exactly one start node',
      severity: 'error',
    });
  } else if (startNodes.length > 1) {
    errors.push({
      code: 'MULTIPLE_START_NODES',
      message: 'Workflow cannot have multiple start nodes',
      severity: 'error',
    });
  }

  // Validation 2: Every workflow must have at least one end node
  const endNodes = workflow.nodes.filter((node: YAWLWorkflow['nodes'][0]) => node.type === 'end');
  if (endNodes.length === 0) {
    errors.push({
      code: 'NO_END_NODE',
      message: 'Workflow must have at least one end node',
      severity: 'error',
    });
  }

  // Validation 3: All nodes must be reachable from start
  const reachableNodes = findReachableNodes(workflow, startNodes[0]?.id);
  const unreachableNodes = workflow.nodes.filter(
    (node: YAWLWorkflow['nodes'][0]) => !reachableNodes.has(node.id) && node.type !== 'start'
  );

  if (unreachableNodes.length > 0) {
    errors.push({
      code: 'UNREACHABLE_NODES',
      message: `Found ${unreachableNodes.length} unreachable nodes`,
      severity: 'error',
    });
  }

  // Validation 4: No dangling edges
  const nodeIds = new Set(workflow.nodes.map((n: YAWLWorkflow['nodes'][0]) => n.id));
  const danglingEdges = workflow.edges.filter(
    (edge: YAWLWorkflow['edges'][0]) => !nodeIds.has(edge.source) || !nodeIds.has(edge.target)
  );

  if (danglingEdges.length > 0) {
    errors.push({
      code: 'DANGLING_EDGES',
      message: `Found ${danglingEdges.length} edges referencing non-existent nodes`,
      severity: 'error',
    });
  }

  // Validation 5: Pattern-specific validation
  workflow.nodes.forEach((node) => {
    const patternErrors = validateNodePattern(node, workflow);
    errors.push(...patternErrors);
  });

  return {
    valid: errors.length === 0,
    errors,
    warnings,
  };
}

/**
 * Find all nodes reachable from a given start node
 */
function findReachableNodes(workflow: YAWLWorkflow, startNodeId: string | undefined): Set<string> {
  if (!startNodeId) return new Set();

  const reachable = new Set<string>([startNodeId]);
  const queue = [startNodeId];

  while (queue.length > 0) {
    const current = queue.shift()!;
    const outgoingEdges = workflow.edges.filter((edge) => edge.source === current);

    outgoingEdges.forEach((edge) => {
      if (!reachable.has(edge.target)) {
        reachable.add(edge.target);
        queue.push(edge.target);
      }
    });
  }

  return reachable;
}

/**
 * Validate pattern-specific constraints
 */
function validateNodePattern(
  node: YAWLWorkflow['nodes'][0],
  workflow: YAWLWorkflow
): ValidationResult['errors'] {
  const errors: ValidationResult['errors'] = [];

  const incomingEdges = workflow.edges.filter((edge: YAWLWorkflow['edges'][0]) => edge.target === node.id);
  const outgoingEdges = workflow.edges.filter((edge: YAWLWorkflow['edges'][0]) => edge.source === node.id);

  switch (node.type) {
    case 'start':
      if (incomingEdges.length > 0) {
        errors.push({
          code: 'INVALID_START_NODE',
          message: 'Start node cannot have incoming edges',
          node: node.id,
          severity: 'error',
        });
      }
      if (outgoingEdges.length !== 1) {
        errors.push({
          code: 'INVALID_START_NODE',
          message: 'Start node must have exactly one outgoing edge',
          node: node.id,
          severity: 'error',
        });
      }
      break;

    case 'end':
      if (outgoingEdges.length > 0) {
        errors.push({
          code: 'INVALID_END_NODE',
          message: 'End node cannot have outgoing edges',
          node: node.id,
          severity: 'error',
        });
      }
      break;

    case 'split':
      if (outgoingEdges.length < 2) {
        errors.push({
          code: 'INVALID_SPLIT',
          message: 'Split node must have at least 2 outgoing edges',
          node: node.id,
          severity: 'error',
        });
      }
      break;

    case 'join':
      if (incomingEdges.length < 2) {
        errors.push({
          code: 'INVALID_JOIN',
          message: 'Join node must have at least 2 incoming edges',
          node: node.id,
          severity: 'error',
        });
      }
      break;
  }

  return errors;
}

/**
 * Check if a pattern combination is valid according to permutation matrix
 *
 * NOTE: This is a placeholder. The actual implementation should reference
 * the YAWL pattern permutation matrix from ontology/yawl-pattern-permutations.ttl
 */
export function isValidPatternCombination(_patterns: PatternType[]): boolean {
  // TODO: Load and validate against actual permutation matrix
  // For now, accept all combinations
  return true;
}
