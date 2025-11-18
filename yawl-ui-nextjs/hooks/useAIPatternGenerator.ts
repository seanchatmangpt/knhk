/**
 * useAIPatternGenerator - AI-powered pattern generation hook
 * Generates YAWL patterns based on requirements
 */

import { useState, useCallback } from 'react'
import type { ControlFlowPattern, YAWLTask } from '@/types/yawl'

interface PatternGenerationState {
  isGenerating: boolean
  generatedPattern: ControlFlowPattern | null
  explanation: string | null
  implementationSteps: string[]
}

/**
 * AI-powered pattern generator hook
 */
export function useAIPatternGenerator() {
  const [state, setState] = useState<PatternGenerationState>({
    isGenerating: false,
    generatedPattern: null,
    explanation: null,
    implementationSteps: [],
  })

  const suggestPattern = useCallback(async (description: string) => {
    setState((prev) => ({
      ...prev,
      isGenerating: true,
    }))

    try {
      const response = await fetch('/api/pattern-generator', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          messages: [{ role: 'user', content: description }],
        }),
      })

      const text = await response.text()
      try {
        const data = JSON.parse(text)
        setState((prev) => ({
          ...prev,
          generatedPattern: data.pattern as ControlFlowPattern,
          explanation: data.explanation,
          implementationSteps: data.steps || [],
          isGenerating: false,
        }))
      } catch {
        setState((prev) => ({
          ...prev,
          explanation: text,
          isGenerating: false,
        }))
      }
    } catch (err) {
      setState((prev) => ({
        ...prev,
        isGenerating: false,
      }))
    }
  }, [])

  return {
    ...state,
    suggestPattern,
  }
}

export default useAIPatternGenerator
