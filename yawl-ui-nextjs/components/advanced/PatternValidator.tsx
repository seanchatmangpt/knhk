/**
 * Advanced Pattern Validator Component
 * Real-time pattern validation with compliance checking
 * Aligned with DOCTRINE Q (Hard Invariants)
 */

'use client'

import React, { useEffect } from 'react'
import { usePatternValidator } from '@/hooks/usePatternValidator'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import type { YAWLSpecification } from '@/types/yawl'
import { AlertCircle, CheckCircle, AlertTriangle } from 'lucide-react'

interface PatternValidatorProps {
  specification: YAWLSpecification
  onValidationChange?: (isValid: boolean) => void
}

/**
 * Real-time pattern validation component
 * Validates pattern sequences and compatibility
 */
export function PatternValidator({
  specification,
  onValidationChange,
}: PatternValidatorProps) {
  const {
    violations,
    isValid,
    validateAll,
    getPatternCoverage,
    getPatternRecommendations,
  } = usePatternValidator()

  const result = validateAll(specification)
  const coverage = getPatternCoverage(specification)
  const recommendations = getPatternRecommendations(specification)

  useEffect(() => {
    onValidationChange?.(isValid)
  }, [isValid, onValidationChange])

  const errorCount = violations.filter((v) => v.severity === 'error').length
  const warningCount = violations.filter((v) => v.severity === 'warning').length

  return (
    <Card className="w-full">
      <CardHeader>
        <div className="flex items-center justify-between">
          <CardTitle className="text-lg">Pattern Validation</CardTitle>
          <div className="flex items-center gap-2">
            {isValid ? (
              <>
                <CheckCircle className="h-5 w-5 text-green-600" />
                <span className="text-sm font-medium text-green-600">Valid</span>
              </>
            ) : (
              <>
                <AlertCircle className="h-5 w-5 text-red-600" />
                <span className="text-sm font-medium text-red-600">Invalid</span>
              </>
            )}
          </div>
        </div>
      </CardHeader>
      <CardContent className="space-y-4">
        {/* Violations Summary */}
        <div className="flex gap-4">
          {errorCount > 0 && (
            <div className="flex items-center gap-1">
              <AlertCircle className="h-4 w-4 text-red-600" />
              <span className="text-sm">
                {errorCount} Error{errorCount !== 1 ? 's' : ''}
              </span>
            </div>
          )}
          {warningCount > 0 && (
            <div className="flex items-center gap-1">
              <AlertTriangle className="h-4 w-4 text-yellow-600" />
              <span className="text-sm">
                {warningCount} Warning{warningCount !== 1 ? 's' : ''}
              </span>
            </div>
          )}
          {errorCount === 0 && warningCount === 0 && (
            <span className="text-sm text-green-600">All patterns valid</span>
          )}
        </div>

        {/* Detailed Violations */}
        {violations.length > 0 && (
          <div className="space-y-2 bg-gray-50 p-3 rounded-md">
            {violations.map((violation) => (
              <div
                key={violation.code}
                className={`text-sm p-2 rounded ${
                  violation.severity === 'error'
                    ? 'bg-red-50 text-red-900'
                    : 'bg-yellow-50 text-yellow-900'
                }`}
              >
                <p className="font-medium">{violation.message}</p>
                {violation.location && (
                  <p className="text-xs opacity-75">{violation.location}</p>
                )}
              </div>
            ))}
          </div>
        )}

        {/* Pattern Coverage */}
        {Object.entries(coverage).some(([_, count]) => count > 0) && (
          <div className="space-y-2">
            <h4 className="font-semibold text-sm">Pattern Coverage</h4>
            <div className="flex flex-wrap gap-2">
              {Object.entries(coverage)
                .filter(([_, count]) => count > 0)
                .map(([pattern, count]) => (
                  <Badge key={pattern} variant="secondary">
                    {pattern}: {count}
                  </Badge>
                ))}
            </div>
          </div>
        )}

        {/* Recommendations */}
        {recommendations.length > 0 && (
          <div className="space-y-2">
            <h4 className="font-semibold text-sm">Recommendations</h4>
            <div className="bg-blue-50 p-3 rounded-md space-y-1">
              {recommendations.map((rec) => (
                <p key={rec} className="text-sm text-blue-900">
                  • Consider using <span className="font-medium">{rec}</span>{' '}
                  pattern
                </p>
              ))}
            </div>
          </div>
        )}
      </CardContent>
    </Card>
  )
}

export default PatternValidator
