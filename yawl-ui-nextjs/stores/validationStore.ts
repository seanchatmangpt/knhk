/**
 * Zustand store for validation state and rules
 * Manages validation rules, patterns, and compliance checks
 */

import { create } from 'zustand'
import { devtools } from 'zustand/middleware'
import type { ValidationRule, ValidationError } from '@/types/yawl'

interface ValidationStore {
  // Rules
  rules: ValidationRule[]
  addRule: (rule: ValidationRule) => void
  removeRule: (id: string) => void
  updateRule: (id: string, updates: Partial<ValidationRule>) => void

  // Errors
  errors: ValidationError[]
  addError: (error: ValidationError) => void
  clearErrors: () => void

  // Compliance
  complianceLevel: number // 0-100
  updateComplianceLevel: (level: number) => void

  // Mode
  strictMode: boolean
  toggleStrictMode: () => void

  // Stats
  getValidationStats: () => {
    totalRules: number
    totalErrors: number
    compliancePercentage: number
  }
}

interface ValidationRule {
  id: string
  name: string
  description?: string
  evaluate: (data: unknown) => boolean
  severity: 'error' | 'warning'
}

export const useValidationStore = create<ValidationStore>()(
  devtools(
    (set, get) => ({
      rules: [],
      errors: [],
      complianceLevel: 100,
      strictMode: true,

      addRule: (rule) =>
        set((state) => ({
          rules: [...state.rules, rule],
        })),

      removeRule: (id) =>
        set((state) => ({
          rules: state.rules.filter((r) => r.id !== id),
        })),

      updateRule: (id, updates) =>
        set((state) => ({
          rules: state.rules.map((r) =>
            r.id === id ? { ...r, ...updates } : r
          ),
        })),

      addError: (error) =>
        set((state) => ({
          errors: [...state.errors, error],
          complianceLevel: Math.max(0, state.complianceLevel - 5),
        })),

      clearErrors: () =>
        set(() => ({
          errors: [],
          complianceLevel: 100,
        })),

      updateComplianceLevel: (level) =>
        set(() => ({
          complianceLevel: Math.max(0, Math.min(100, level)),
        })),

      toggleStrictMode: () =>
        set((state) => ({
          strictMode: !state.strictMode,
        })),

      getValidationStats: () => {
        const state = get()
        return {
          totalRules: state.rules.length,
          totalErrors: state.errors.length,
          compliancePercentage: state.complianceLevel,
        }
      },
    }),
    { name: 'validation-store' }
  )
)

export default useValidationStore
