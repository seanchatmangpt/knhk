/**
 * useAIWorkflowAssistant - AI-powered workflow assistant hook
 * Uses Vercel AI SDK for intelligent workflow suggestions and generation
 * Aligned with DOCTRINE O (Observation) principle
 */

import { useState, useCallback } from 'react'
import { useChat } from 'ai/react'
import type { YAWLSpecification, YAWLTask } from '@/types/yawl'

interface AssistantState {
  isGenerating: boolean
  suggestedWorkflow: YAWLSpecification | null
  suggestions: string[]
  analysisResult: string | null
}

/**
 * AI-powered workflow assistant hook
 * Provides intelligent suggestions for workflow design
 */
export function useAIWorkflowAssistant() {
  const [state, setState] = useState<AssistantState>({
    isGenerating: false,
    suggestedWorkflow: null,
    suggestions: [],
    analysisResult: null,
  })

  const { messages, input, handleInputChange, handleSubmit } = useChat({
    api: '/api/workflow-assistant',
    onFinish: (message) => {
      // Process AI response
      try {
        const data = JSON.parse(message.content)
        setState((prev) => ({
          ...prev,
          suggestedWorkflow: data.workflow,
          suggestions: data.suggestions || [],
          analysisResult: data.analysis,
        }))
      } catch {
        // Message is text, not JSON
        setState((prev) => ({
          ...prev,
          analysisResult: message.content,
        }))
      }
    },
  })

  /**
   * Ask AI to generate workflow from description
   */
  const generateWorkflow = useCallback(
    async (description: string) => {
      setState((prev) => ({ ...prev, isGenerating: true }))

      const prompt = `You are an expert workflow designer. Generate a YAWL workflow based on this description:

Description: ${description}

Provide a JSON response with:
- workflow: { id, name, version, tasks: [], nets: [] }
- suggestions: [] (array of improvement suggestions)

Ensure the workflow includes:
1. Clear task sequence
2. Appropriate control flow patterns
3. Data mappings where relevant`

      try {
        // Submit to chat API
        const formData = new FormData()
        formData.append('messages', JSON.stringify([...messages, { role: 'user', content: prompt }]))

        // This will be handled by the chat hook's onFinish
        handleInputChange({ target: { value: prompt } } as any)
      } finally {
        setState((prev) => ({ ...prev, isGenerating: false }))
      }
    },
    [messages, handleInputChange]
  )

  /**
   * Ask AI to analyze workflow
   */
  const analyzeWorkflow = useCallback(
    async (spec: YAWLSpecification) => {
      setState((prev) => ({ ...prev, isGenerating: true }))

      const prompt = `Analyze this YAWL workflow and provide improvements:

${JSON.stringify(spec, null, 2)}

Provide:
1. Complexity analysis
2. Pattern recommendations
3. Potential bottlenecks
4. Suggested optimizations`

      try {
        handleInputChange({ target: { value: prompt } } as any)
      } finally {
        setState((prev) => ({ ...prev, isGenerating: false }))
      }
    },
    [handleInputChange]
  )

  /**
   * Ask AI for pattern suggestions
   */
  const suggestPatterns = useCallback(
    async (spec: YAWLSpecification) => {
      setState((prev) => ({ ...prev, isGenerating: true }))

      const prompt = `Given this workflow:

${JSON.stringify(spec, null, 2)}

Suggest 3 control flow patterns that would improve this workflow. For each pattern:
1. Name
2. Why it applies
3. Where to apply it
4. Expected benefits`

      try {
        handleInputChange({ target: { value: prompt } } as any)
      } finally {
        setState((prev) => ({ ...prev, isGenerating: false }))
      }
    },
    [handleInputChange]
  )

  /**
   * Ask AI to optimize workflow
   */
  const optimizeWorkflow = useCallback(
    async (spec: YAWLSpecification) => {
      setState((prev) => ({ ...prev, isGenerating: true }))

      const prompt = `Optimize this workflow for efficiency:

${JSON.stringify(spec, null, 2)}

Provide:
1. Identified inefficiencies
2. Proposed optimizations
3. Expected performance improvement
4. Implementation steps`

      try {
        handleInputChange({ target: { value: prompt } } as any)
      } finally {
        setState((prev) => ({ ...prev, isGenerating: false }))
      }
    },
    [handleInputChange]
  )

  return {
    // State
    messages,
    input,
    isGenerating: state.isGenerating,
    suggestedWorkflow: state.suggestedWorkflow,
    suggestions: state.suggestions,
    analysisResult: state.analysisResult,

    // Actions
    handleInputChange,
    handleSubmit,
    generateWorkflow,
    analyzeWorkflow,
    suggestPatterns,
    optimizeWorkflow,
  }
}

export default useAIWorkflowAssistant
