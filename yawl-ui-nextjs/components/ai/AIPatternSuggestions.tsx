/**
 * AI Pattern Suggestions Component
 * Displays AI-powered pattern recommendations
 */

'use client'

import React, { useEffect } from 'react'
import { useAIPatternGenerator } from '@/hooks/useAIPatternGenerator'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Sparkles, Zap, CheckCircle } from 'lucide-react'
import type { YAWLSpecification } from '@/types/yawl'

interface AIPatternSuggestionsProps {
  specification: YAWLSpecification
  onPatternSelected?: (pattern: string) => void
}

/**
 * Display AI pattern suggestions for a workflow
 */
export function AIPatternSuggestions({
  specification,
  onPatternSelected,
}: AIPatternSuggestionsProps) {
  const {
    isGenerating,
    pattern,
    explanation,
    steps,
    suggestPattern,
  } = useAIPatternGenerator()

  useEffect(() => {
    // Suggest patterns on mount
    const parallelizability = specification.tasks.length > 3 ? 'high' : 'low'
    suggestPattern(
      `Workflow with ${specification.tasks.length} tasks`,
      specification.tasks.length,
      parallelizability
    )
  }, [specification.tasks.length, suggestPattern])

  return (
    <Card className="w-full">
      <CardHeader>
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <Sparkles className="h-5 w-5" />
            <CardTitle className="text-lg">AI Pattern Recommendations</CardTitle>
          </div>
          {pattern && (
            <Button
              size="sm"
              onClick={() => onPatternSelected?.(pattern)}
            >
              <Zap className="h-3 w-3 mr-1" />
              Apply Pattern
            </Button>
          )}
        </div>
      </CardHeader>

      <CardContent className="space-y-4">
        {isGenerating ? (
          <div className="text-center py-6">
            <div className="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-primary" />
            <p className="text-sm text-muted-foreground mt-2">
              Analyzing workflow...
            </p>
          </div>
        ) : pattern ? (
          <div className="space-y-4">
            {/* Pattern Badge */}
            <div className="bg-gradient-to-r from-blue-50 to-purple-50 border border-blue-200 rounded-lg p-4">
              <div className="flex items-center gap-2 mb-2">
                <CheckCircle className="h-5 w-5 text-green-600" />
                <p className="font-semibold">{pattern}</p>
              </div>
              <p className="text-sm text-gray-700">{explanation}</p>
            </div>

            {/* Implementation Steps */}
            {steps.length > 0 && (
              <div className="space-y-2">
                <h4 className="font-semibold text-sm">Implementation Steps</h4>
                <ol className="space-y-1">
                  {steps.map((step, idx) => (
                    <li key={idx} className="text-sm flex gap-2">
                      <span className="font-medium text-blue-600">
                        {idx + 1}.
                      </span>
                      <span>{step}</span>
                    </li>
                  ))}
                </ol>
              </div>
            )}

            {/* Why This Pattern */}
            <div className="bg-green-50 border border-green-200 rounded-lg p-3">
              <p className="text-xs font-semibold text-green-900 mb-1">
                WHY THIS PATTERN
              </p>
              <p className="text-sm text-green-800">
                This pattern optimizes for workflows with{' '}
                <strong>{specification.tasks.length} tasks</strong> and
                provides clear synchronization points.
              </p>
            </div>
          </div>
        ) : (
          <p className="text-center text-muted-foreground py-6">
            No pattern recommendations available
          </p>
        )}
      </CardContent>
    </Card>
  )
}

export default AIPatternSuggestions
