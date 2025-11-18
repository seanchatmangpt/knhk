/**
 * useAIPatternGenerator - AI-powered pattern generation hook
 * Generates YAWL patterns based on requirements
 */

import { useState, useCallback } from 'react'
import { useChat } from 'ai/react'
import type { ControlFlowPattern, YAWLTask } from '@/types/yawl'

interface PatternGenerationState {
  isGenerating: boolean
  generatedPattern: ControlFlowPattern | null
  explanation: string | null
  implementationSteps: string[]
}

/**
 * AI-powered pattern generator hook
 * Intelligently suggests and explains control flow patterns
 */
export function useAIPatternGenerator() {
  const [state, setState] = useState<PatternGenerationState>({
    isGenerating: false,
    generatedPattern: null,
    explanation: null,
    implementationSteps: [],
  })

  const { messages, input, handleInputChange, handleSubmit } = useChat({
    api: '/api/pattern-generator',
    onFinish: (message) => {
      try {
        const data = JSON.parse(message.content)
        setState((prev) => ({
          ...prev,
          generatedPattern: data.pattern,
          explanation: data.explanation,
          implementationSteps: data.steps || [],
        }))
      } catch {
        setState((prev) => ({
          ...prev,
          explanation: message.content,
        }))
      }
    },
  })

  /**
   * Suggest pattern for scenario
   */
  const suggestPattern = useCallback(
    async (scenario: string, taskCount: number, parallelizability: string) => {
      setState((prev) => ({ ...prev, isGenerating: true }))

      const prompt = `As a workflow design expert, suggest the best YAWL control flow pattern for:

Scenario: ${scenario}
Task Count: ${taskCount}
Parallelizability: ${parallelizability}

Provide JSON response:
{
  "pattern": "pattern-name",
  "explanation": "why this pattern is best",
  "steps": ["step1", "step2", ...]
}`

      try {
        handleInputChange({ target: { value: prompt } } as any)
      } finally {
        setState((prev) => ({ ...prev, isGenerating: false }))
      }
    },
    [handleInputChange]
  )

  /**
   * Suggest pattern composition
   */
  const suggestComposition = useCallback(
    async (patterns: ControlFlowPattern[]) => {
      setState((prev) => ({ ...prev, isGenerating: true }))

      const prompt = `How should these patterns be combined?
Patterns: ${patterns.join(', ')}

Provide:
1. Optimal composition
2. Why this combination works
3. Interaction points
4. Potential issues`

      try {
        handleInputChange({ target: { value: prompt } } as any)
      } finally {
        setState((prev) => ({ ...prev, isGenerating: false }))
      }
    },
    [handleInputChange]
  )

  /**
   * Validate pattern implementation
   */
  const validatePattern = useCallback(
    async (pattern: ControlFlowPattern, tasks: YAWLTask[]) => {
      setState((prev) => ({ ...prev, isGenerating: true }))

      const prompt = `Validate this pattern implementation:
Pattern: ${pattern}
Tasks: ${JSON.stringify(tasks)}

Check for:
1. Correctness
2. Efficiency
3. Potential deadlocks
4. Improvements`

      try {
        handleInputChange({ target: { value: prompt } } as any)
      } finally {
        setState((prev) => ({ ...prev, isGenerating: false }))
      }
    },
    [handleInputChange]
  )

  return {
    messages,
    input,
    isGenerating: state.isGenerating,
    pattern: state.generatedPattern,
    explanation: state.explanation,
    steps: state.implementationSteps,
    handleInputChange,
    handleSubmit,
    suggestPattern,
    suggestComposition,
    validatePattern,
  }
}

export default useAIPatternGenerator
