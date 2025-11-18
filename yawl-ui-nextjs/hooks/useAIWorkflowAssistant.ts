/**
 * useAIWorkflowAssistant - AI-powered workflow assistant hook
 * Uses Vercel AI SDK for intelligent workflow suggestions and generation
 * Aligned with DOCTRINE O (Observation) principle
 */

import { useState, useCallback } from 'react'
import type { YAWLSpecification, YAWLTask } from '@/types/yawl'

interface AssistantState {
  isGenerating: boolean
  suggestedWorkflow: YAWLSpecification | null
  suggestions: string[]
  analysisResult: string | null
}

/**
 * AI-powered workflow assistant hook
 */
export function useAIWorkflowAssistant() {
  const [state, setState] = useState<AssistantState>({
    isGenerating: false,
    suggestedWorkflow: null,
    suggestions: [],
    analysisResult: null,
  })

  const generateWorkflow = useCallback(async (description: string) => {
    setState((prev) => ({ ...prev, isGenerating: true }))

    try {
      const response = await fetch('/api/workflow-assistant', {
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
          suggestedWorkflow: data.workflow,
          suggestions: data.suggestions || [],
          analysisResult: data.analysis,
          isGenerating: false,
        }))
      } catch {
        setState((prev) => ({
          ...prev,
          analysisResult: text,
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

  const analyzeWorkflow = useCallback(async (workflow: YAWLSpecification) => {
    setState((prev) => ({ ...prev, isGenerating: true }))

    try {
      const response = await fetch('/api/workflow-assistant', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          messages: [
            {
              role: 'user',
              content: `Analyze this workflow: ${JSON.stringify(workflow)}`,
            },
          ],
        }),
      })

      const text = await response.text()
      setState((prev) => ({
        ...prev,
        analysisResult: text,
        isGenerating: false,
      }))
    } catch (err) {
      setState((prev) => ({
        ...prev,
        isGenerating: false,
      }))
    }
  }, [])

  const suggestPatterns = useCallback(async (workflowDescription: string) => {
    setState((prev) => ({ ...prev, isGenerating: true }))

    try {
      const response = await fetch('/api/workflow-assistant', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          messages: [
            {
              role: 'user',
              content: `Suggest control flow patterns for: ${workflowDescription}`,
            },
          ],
        }),
      })

      const text = await response.text()
      setState((prev) => ({
        ...prev,
        suggestions: text.split('\n').filter((s) => s.trim()),
        isGenerating: false,
      }))
    } catch (err) {
      setState((prev) => ({
        ...prev,
        isGenerating: false,
      }))
    }
  }, [])

  return {
    ...state,
    generateWorkflow,
    analyzeWorkflow,
    suggestPatterns,
  }
}

export default useAIWorkflowAssistant
