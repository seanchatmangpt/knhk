/**
 * usePatternValidator - Advanced YAWL pattern validation
 * Validates workflow patterns against combinatorial rules
 * Aligned with DOCTRINE Q (Hard Invariants)
 */

import { useCallback, useState } from 'react'
import type {
  YAWLSpecification,
  ControlFlowPattern,
  ValidationResult,
  ValidationError,
} from '@/types/yawl'

interface PatternRules {
  pattern: ControlFlowPattern
  allowedSuccessors: ControlFlowPattern[]
  requiredPredecessors?: ControlFlowPattern[]
  maxInstances?: number
  constraints: PatternConstraint[]
}

interface PatternConstraint {
  id: string
  rule: string
  enforce: (spec: YAWLSpecification, index: number) => boolean
}

// Define pattern permutation matrix (YAWL control flow compatibility)
const PATTERN_MATRIX: Record<ControlFlowPattern, ControlFlowPattern[]> = {
  sequence: ['sequence', 'parallel', 'choice', 'synchronization'],
  parallel: ['parallel', 'synchronization', 'exclusive-choice'],
  'exclusive-choice': ['sequence', 'parallel', 'synchronization'],
  choice: ['sequence', 'parallel', 'synchronization'],
  'deferred-choice': ['sequence', 'parallel'],
  'implicit-choice': ['sequence'],
  'interleaved-parallel': ['synchronization', 'multiple-merge'],
  'multi-choice': ['synchronization', 'discriminator'],
  synchronization: ['sequence', 'choice', 'exclusive-choice'],
  'multiple-merge': ['sequence', 'exclusive-choice'],
  discriminator: ['sequence', 'exclusive-choice'],
}

/**
 * Hook for validating workflow patterns
 * Enforces pattern compatibility and combinatorial rules
 */
export function usePatternValidator() {
  const [violations, setViolations] = useState<ValidationError[]>([])
  const [isValid, setIsValid] = useState(true)

  // Validate pattern sequence
  const validatePatternSequence = useCallback(
    (spec: YAWLSpecification): boolean => {
      const errors: ValidationError[] = []
      const net = spec.nets[0]

      if (!net || !net.flows) {
        setViolations(errors)
        setIsValid(true)
        return true
      }

      // Check each flow pattern for compatibility
      net.flows.forEach((flow, index) => {
        if (!flow.pattern) return

        const nextFlow = net.flows[index + 1]
        if (nextFlow && nextFlow.pattern) {
          const allowedPatterns = PATTERN_MATRIX[flow.pattern]

          if (!allowedPatterns.includes(nextFlow.pattern)) {
            errors.push({
              code: 'INVALID_PATTERN_SEQUENCE',
              message: `Pattern "${flow.pattern}" cannot be followed by "${nextFlow.pattern}"`,
              location: `flow-${index}`,
              severity: 'error',
            })
          }
        }
      })

      const hasErrors = errors.some((e) => e.severity === 'error')
      setViolations(errors)
      setIsValid(!hasErrors)

      return !hasErrors
    },
    []
  )

  // Validate parallel/synchronization balance
  const validateParallelBalance = useCallback(
    (spec: YAWLSpecification): boolean => {
      const errors: ValidationError[] = []
      const net = spec.nets[0]

      if (!net || !net.flows) return true

      const parallelFlows = net.flows.filter((f) => f.pattern === 'parallel')
      const syncFlows = net.flows.filter((f) => f.pattern === 'synchronization')

      if (parallelFlows.length > syncFlows.length) {
        errors.push({
          code: 'UNBALANCED_PARALLELISM',
          message: `Found ${parallelFlows.length} parallel patterns but only ${syncFlows.length} synchronization patterns`,
          severity: 'warning',
        })
      }

      setViolations((prev) => [...prev, ...errors])

      return errors.length === 0
    },
    []
  )

  // Validate choice/merge balance
  const validateChoiceMergeBalance = useCallback(
    (spec: YAWLSpecification): boolean => {
      const errors: ValidationError[] = []
      const net = spec.nets[0]

      if (!net || !net.flows) return true

      const choiceFlows = net.flows.filter(
        (f) =>
          f.pattern === 'choice' ||
          f.pattern === 'exclusive-choice' ||
          f.pattern === 'implicit-choice'
      )
      const mergeFlows = net.flows.filter(
        (f) =>
          f.pattern === 'multiple-merge' || f.pattern === 'discriminator'
      )

      if (choiceFlows.length !== mergeFlows.length) {
        errors.push({
          code: 'UNBALANCED_CHOICE',
          message: `Found ${choiceFlows.length} choice patterns but ${mergeFlows.length} merge patterns`,
          severity: 'warning',
        })
      }

      setViolations((prev) => [...prev, ...errors])

      return errors.length === 0
    },
    []
  )

  // Validate pattern coverage
  const getPatternCoverage = useCallback((spec: YAWLSpecification) => {
    const net = spec.nets[0]
    if (!net || !net.flows) return {}

    const coverage: Record<ControlFlowPattern, number> = {
      sequence: 0,
      parallel: 0,
      choice: 0,
      'exclusive-choice': 0,
      'implicit-choice': 0,
      'deferred-choice': 0,
      'interleaved-parallel': 0,
      'multi-choice': 0,
      synchronization: 0,
      'multiple-merge': 0,
      discriminator: 0,
    }

    net.flows.forEach((flow) => {
      if (flow.pattern && flow.pattern in coverage) {
        coverage[flow.pattern]++
      }
    })

    return coverage
  }, [])

  // Get pattern recommendations
  const getPatternRecommendations = useCallback(
    (spec: YAWLSpecification): ControlFlowPattern[] => {
      const recommended: ControlFlowPattern[] = []

      if (spec.tasks.length > 3) {
        recommended.push('parallel')
      }
      if (spec.tasks.length > 5) {
        recommended.push('choice', 'exclusive-choice')
      }
      if (
        spec.nets[0]?.flows?.some((f) => f.pattern === 'parallel')
      ) {
        recommended.push('synchronization')
      }

      return recommended
    },
    []
  )

  // Comprehensive validation
  const validateAll = useCallback((spec: YAWLSpecification) => {
    const errors: ValidationError[] = []

    // Run all validations
    validatePatternSequence(spec)
    validateParallelBalance(spec)
    validateChoiceMergeBalance(spec)

    const coverage = getPatternCoverage(spec)
    const recommendations = getPatternRecommendations(spec)

    return {
      isValid,
      errors: violations,
      coverage,
      recommendations,
    }
  }, [
    validatePatternSequence,
    validateParallelBalance,
    validateChoiceMergeBalance,
    getPatternCoverage,
    getPatternRecommendations,
    isValid,
    violations,
  ])

  return {
    violations,
    isValid,
    validatePatternSequence,
    validateParallelBalance,
    validateChoiceMergeBalance,
    getPatternCoverage,
    getPatternRecommendations,
    validateAll,
  }
}

export default usePatternValidator
