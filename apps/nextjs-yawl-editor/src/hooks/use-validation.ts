/**
 * DOCTRINE ALIGNMENT: Q (Hard Invariants)
 * Custom hook for real-time pattern validation
 *
 * COVENANT 2: Invariants Are Law
 * All validation errors are blocking - no warnings allowed for invalid patterns
 */

'use client';

import { useCallback, useEffect, useState } from 'react';
import { useWorkflow } from './use-workflow';
import { validateWorkflow } from '@/lib/validation/pattern-validator';
import type { ValidationResult } from '@/lib/types';
import { trackValidation } from '@/lib/telemetry/setup';

export interface UseValidationReturn {
  validation: ValidationResult | null;
  isValidating: boolean;
  validate: () => Promise<ValidationResult>;
  clearValidation: () => void;
}

/**
 * Hook for real-time workflow validation
 * Automatically validates on workflow changes with debouncing
 *
 * @param autoValidate - Whether to automatically validate on changes (default: true)
 * @param debounceMs - Debounce delay in milliseconds (default: 500)
 *
 * @example
 * ```tsx
 * const { validation, isValidating, validate } = useValidation();
 *
 * if (validation && !validation.valid) {
 *   console.error('Validation errors:', validation.errors);
 * }
 * ```
 */
export function useValidation(
  autoValidate = true,
  debounceMs = 500
): UseValidationReturn {
  const { workflow } = useWorkflow();
  const [validation, setValidation] = useState<ValidationResult | null>(null);
  const [isValidating, setIsValidating] = useState(false);
  const [debounceTimer, setDebounceTimer] = useState<NodeJS.Timeout | null>(null);

  const validate = useCallback(async (): Promise<ValidationResult> => {
    if (!workflow) {
      const emptyResult: ValidationResult = {
        valid: false,
        errors: [{
          code: 'NO_WORKFLOW',
          message: 'No workflow loaded',
          severity: 'error',
        }],
        warnings: [],
      };
      setValidation(emptyResult);
      return emptyResult;
    }

    setIsValidating(true);

    try {
      // Track validation with telemetry
      const result = await trackValidation(workflow.id, async () => {
        // Perform validation (must complete in ≤8 ticks per Chatman Constant)
        const startTime = performance.now();
        const validationResult = validateWorkflow(workflow);
        const duration = performance.now() - startTime;

        // Verify performance constraint (8 ticks ~= 100ms at 60fps)
        if (duration > 100) {
          console.warn(
            `Validation exceeded Chatman Constant: ${duration.toFixed(2)}ms > 100ms`
          );
        }

        return validationResult;
      });

      setValidation(result);
      return result;
    } finally {
      setIsValidating(false);
    }
  }, [workflow]);

  const clearValidation = useCallback(() => {
    setValidation(null);
  }, []);

  // Auto-validate on workflow changes with debouncing
  useEffect(() => {
    if (!autoValidate || !workflow) return;

    // Clear existing timer
    if (debounceTimer) {
      clearTimeout(debounceTimer);
    }

    // Set new debounced validation
    const timer = setTimeout(() => {
      validate();
    }, debounceMs);

    setDebounceTimer(timer);

    // Cleanup
    return () => {
      if (timer) {
        clearTimeout(timer);
      }
    };
  }, [workflow, autoValidate, debounceMs]); // Intentionally not including validate to avoid infinite loop

  return {
    validation,
    isValidating,
    validate,
    clearValidation,
  };
}
