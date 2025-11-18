/**
 * Zustand store for workflow state management
 * Provides global state for workflow specifications and cases
 */

import { create } from 'zustand'
import { devtools } from 'zustand/middleware'
import type {
  YAWLSpecification,
  WorkflowCase,
  WorkItem,
} from '@/types/yawl'

interface WorkflowStore {
  // Specifications
  specifications: YAWLSpecification[]
  currentSpec: YAWLSpecification | null
  addSpecification: (spec: YAWLSpecification) => void
  setCurrentSpec: (spec: YAWLSpecification | null) => void
  updateSpecification: (id: string, updates: Partial<YAWLSpecification>) => void
  deleteSpecification: (id: string) => void

  // Cases
  cases: WorkflowCase[]
  addCase: (caseItem: WorkflowCase) => void
  updateCase: (id: string, updates: Partial<WorkflowCase>) => void
  deleteCase: (id: string) => void

  // Work Items
  workItems: WorkItem[]
  addWorkItem: (item: WorkItem) => void
  updateWorkItem: (id: string, updates: Partial<WorkItem>) => void

  // UI State
  selectedSpecId: string | null
  selectedCaseId: string | null
  setSelectedSpec: (id: string | null) => void
  setSelectedCase: (id: string | null) => void

  // Stats
  getStats: () => {
    totalSpecs: number
    totalCases: number
    activeCases: number
    pendingItems: number
  }
}

export const useWorkflowStore = create<WorkflowStore>()(
  devtools(
    (set, get) => ({
      specifications: [],
      currentSpec: null,
      cases: [],
      workItems: [],
      selectedSpecId: null,
      selectedCaseId: null,

      addSpecification: (spec) =>
        set((state) => ({
          specifications: [...state.specifications, spec],
        })),

      setCurrentSpec: (spec) =>
        set(() => ({
          currentSpec: spec,
          selectedSpecId: spec?.id || null,
        })),

      updateSpecification: (id, updates) =>
        set((state) => ({
          specifications: state.specifications.map((s) =>
            s.id === id ? { ...s, ...updates } : s
          ),
          currentSpec:
            state.currentSpec?.id === id
              ? { ...state.currentSpec, ...updates }
              : state.currentSpec,
        })),

      deleteSpecification: (id) =>
        set((state) => ({
          specifications: state.specifications.filter((s) => s.id !== id),
          currentSpec:
            state.currentSpec?.id === id ? null : state.currentSpec,
        })),

      addCase: (caseItem) =>
        set((state) => ({
          cases: [...state.cases, caseItem],
        })),

      updateCase: (id, updates) =>
        set((state) => ({
          cases: state.cases.map((c) =>
            c.id === id ? { ...c, ...updates } : c
          ),
        })),

      deleteCase: (id) =>
        set((state) => ({
          cases: state.cases.filter((c) => c.id !== id),
        })),

      addWorkItem: (item) =>
        set((state) => ({
          workItems: [...state.workItems, item],
        })),

      updateWorkItem: (id, updates) =>
        set((state) => ({
          workItems: state.workItems.map((w) =>
            w.id === id ? { ...w, ...updates } : w
          ),
        })),

      setSelectedSpec: (id) =>
        set(() => ({
          selectedSpecId: id,
        })),

      setSelectedCase: (id) =>
        set(() => ({
          selectedCaseId: id,
        })),

      getStats: () => {
        const state = get()
        return {
          totalSpecs: state.specifications.length,
          totalCases: state.cases.length,
          activeCases: state.cases.filter(
            (c) => c.status === 'running' || c.status === 'suspended'
          ).length,
          pendingItems: state.workItems.filter(
            (w) =>
              w.status === 'offered' ||
              w.status === 'allocated'
          ).length,
        }
      },
    }),
    { name: 'workflow-store' }
  )
)

export default useWorkflowStore
