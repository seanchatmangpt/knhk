/**
 * DOCTRINE ALIGNMENT: Q (Hard Invariants)
 * ValidationFeedback - Display real-time validation results
 *
 * COVENANT 2: Invariants Are Law
 * Shows blocking errors, never just warnings
 */

'use client';

import { useCallback } from 'react';
import { AlertCircle, CheckCircle2, XCircle, Info } from 'lucide-react';
import { useValidation } from '@/hooks/use-validation';
import { useTelemetry } from '@/hooks/use-telemetry';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Separator } from '@/components/ui/separator';
import { cn } from '@/lib/utils';
import type { ValidationError, ValidationWarning } from '@/lib/types';

export interface ValidationFeedbackProps {
  className?: string;
}

/**
 * ValidationFeedback - Display workflow validation results
 *
 * Features:
 * - Error list with details
 * - Warning indicators
 * - Severity levels
 * - Clear button
 * - Real-time updates
 *
 * @example
 * ```tsx
 * <ValidationFeedback className="w-80" />
 * ```
 */
export function ValidationFeedback({ className }: ValidationFeedbackProps) {
  const { validation, isValidating, validate, clearValidation } = useValidation();
  const { trackEvent } = useTelemetry('ValidationFeedback');

  const handleValidate = useCallback(async () => {
    trackEvent('validation.trigger');
    await validate();
  }, [validate, trackEvent]);

  const handleClear = useCallback(() => {
    trackEvent('validation.clear');
    clearValidation();
  }, [clearValidation, trackEvent]);

  const handleErrorClick = useCallback((error: ValidationError) => {
    trackEvent('validation.error.click', {
      code: error.code,
      node: error.node || 'unknown',
    });
    // Could emit event to select the node
  }, [trackEvent]);

  const handleWarningClick = useCallback((warning: ValidationWarning) => {
    trackEvent('validation.warning.click', {
      code: warning.code,
      node: warning.node || 'unknown',
    });
  }, [trackEvent]);

  const errorCount = validation?.errors.length || 0;
  const warningCount = validation?.warnings.length || 0;

  return (
    <Card className={cn('border rounded-lg shadow-lg', className)}>
      <CardHeader className="pb-4">
        <div className="flex items-center justify-between">
          <div>
            <CardTitle className="text-lg">Validation</CardTitle>
            <CardDescription className="text-xs mt-1">
              {isValidating
                ? 'Validating workflow...'
                : validation
                  ? validation.valid
                    ? 'No issues found'
                    : `${errorCount} error${errorCount > 1 ? 's' : ''}, ${warningCount} warning${warningCount > 1 ? 's' : ''}`
                  : 'Not validated'}
            </CardDescription>
          </div>

          {/* Status icon */}
          {validation && !isValidating && (
            validation.valid ? (
              <CheckCircle2 className="h-6 w-6 text-green-600" />
            ) : (
              <XCircle className="h-6 w-6 text-red-600" />
            )
          )}
          {isValidating && (
            <div className="animate-spin h-6 w-6 border-2 border-blue-600 border-t-transparent rounded-full" />
          )}
        </div>

        {/* Action buttons */}
        <div className="flex gap-2 pt-3">
          <Button
            variant="outline"
            size="sm"
            onClick={handleValidate}
            disabled={isValidating}
            className="flex-1"
          >
            Validate
          </Button>
          {validation && (
            <Button
              variant="ghost"
              size="sm"
              onClick={handleClear}
            >
              Clear
            </Button>
          )}
        </div>
      </CardHeader>

      {validation && (
        <>
          <Separator />

          <CardContent className="p-0">
            <ScrollArea className="h-[300px]">
              <div className="p-4 space-y-3">
                {/* Errors */}
                {validation.errors.length > 0 && (
                  <div className="space-y-2">
                    <div className="flex items-center gap-2">
                      <AlertCircle className="h-4 w-4 text-red-600" />
                      <div className="text-sm font-semibold text-red-600">
                        Errors ({validation.errors.length})
                      </div>
                    </div>

                    <div className="space-y-2">
                      {validation.errors.map((error, idx) => (
                        <div
                          key={idx}
                          onClick={() => handleErrorClick(error)}
                          className={cn(
                            'p-3 rounded-lg border-l-4 border-red-500',
                            'bg-red-50 cursor-pointer hover:bg-red-100',
                            'transition-colors'
                          )}
                        >
                          <div className="flex items-start justify-between gap-2">
                            <div className="flex-1 min-w-0">
                              <div className="text-sm font-medium text-red-900">
                                {error.code}
                              </div>
                              <div className="text-xs text-red-700 mt-1">
                                {error.message}
                              </div>
                              {error.node && (
                                <Badge variant="outline" className="mt-2 text-xs">
                                  Node: {error.node}
                                </Badge>
                              )}
                            </div>
                            <Badge variant="destructive" className="flex-shrink-0">
                              Error
                            </Badge>
                          </div>
                        </div>
                      ))}
                    </div>
                  </div>
                )}

                {/* Warnings */}
                {validation.warnings.length > 0 && (
                  <div className="space-y-2">
                    <div className="flex items-center gap-2">
                      <Info className="h-4 w-4 text-yellow-600" />
                      <div className="text-sm font-semibold text-yellow-600">
                        Warnings ({validation.warnings.length})
                      </div>
                    </div>

                    <div className="space-y-2">
                      {validation.warnings.map((warning, idx) => (
                        <div
                          key={idx}
                          onClick={() => handleWarningClick(warning)}
                          className={cn(
                            'p-3 rounded-lg border-l-4 border-yellow-500',
                            'bg-yellow-50 cursor-pointer hover:bg-yellow-100',
                            'transition-colors'
                          )}
                        >
                          <div className="flex items-start justify-between gap-2">
                            <div className="flex-1 min-w-0">
                              <div className="text-sm font-medium text-yellow-900">
                                {warning.code}
                              </div>
                              <div className="text-xs text-yellow-700 mt-1">
                                {warning.message}
                              </div>
                              {warning.node && (
                                <Badge variant="outline" className="mt-2 text-xs">
                                  Node: {warning.node}
                                </Badge>
                              )}
                            </div>
                            <Badge variant="secondary" className="flex-shrink-0">
                              Warning
                            </Badge>
                          </div>
                        </div>
                      ))}
                    </div>
                  </div>
                )}

                {/* Success state */}
                {validation.valid && (
                  <div className="text-center py-8">
                    <CheckCircle2 className="h-12 w-12 text-green-600 mx-auto mb-3" />
                    <div className="text-sm font-medium text-green-900">
                      Workflow is valid
                    </div>
                    <div className="text-xs text-green-700 mt-1">
                      All pattern constraints satisfied
                    </div>
                  </div>
                )}
              </div>
            </ScrollArea>
          </CardContent>
        </>
      )}

      {!validation && !isValidating && (
        <CardContent className="p-8 text-center text-gray-500 text-sm">
          Click Validate to check workflow
        </CardContent>
      )}
    </Card>
  );
}
