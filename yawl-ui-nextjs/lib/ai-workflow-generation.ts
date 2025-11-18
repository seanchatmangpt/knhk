/**
 * AI Workflow Generation Service
 * Uses AI to generate complete YAWL workflows from descriptions
 * Implements RAG (Retrieval-Augmented Generation)
 */

import { workflowKnowledgeBase } from './workflow-knowledge-base'
import WorkflowService from './workflow-service'
import type { YAWLSpecification, YAWLTask, ControlFlowPattern } from '@/types/yawl'

export interface GenerationContext {
  description: string
  taskCount?: number
  patterns?: ControlFlowPattern[]
  complexityLevel?: 'simple' | 'medium' | 'complex'
}

/**
 * Service for AI-powered workflow generation
 * Integrates RAG for context-aware suggestions
 */
export class AIWorkflowGenerationService {
  /**
   * Generate workflow from natural language description
   * Uses knowledge base for context-aware generation
   */
  static generateFromDescription(context: GenerationContext): YAWLSpecification {
    // Retrieve relevant knowledge from KB
    const relevantPatterns = workflowKnowledgeBase.getPatternsForScenario(
      context.description
    )
    const bestPractices = workflowKnowledgeBase.getBestPractices(
      context.description
    )

    // Create base specification
    const spec = WorkflowService.createSpecification(
      `wf-${Date.now()}`,
      this.extractWorkflowName(context.description),
      '1.0'
    )

    // Parse description to extract tasks
    const tasks = this.extractTasks(context.description, context.taskCount || 3)

    // Add tasks to specification
    let updatedSpec = spec
    tasks.forEach((task) => {
      updatedSpec = WorkflowService.addTask(updatedSpec, task)
    })

    // Determine patterns to use
    const patternsToUse =
      context.patterns ||
      this.selectPatterns(
        tasks.length,
        context.complexityLevel || 'medium',
        relevantPatterns
      )

    // Add control flows with patterns
    this.addControlFlows(updatedSpec, tasks, patternsToUse)

    return updatedSpec
  }

  /**
   * Extract workflow name from description
   */
  private static extractWorkflowName(description: string): string {
    // Simple heuristic: first 3 words or up to first punctuation
    const words = description.split(/\s+/).slice(0, 3)
    return words.join(' ') || 'Workflow'
  }

  /**
   * Extract tasks from description using simple NLP
   */
  private static extractTasks(description: string, count: number): YAWLTask[] {
    const tasks: YAWLTask[] = []

    // Split by common delimiters and action verbs
    const actionVerbs = [
      'receive',
      'process',
      'approve',
      'send',
      'validate',
      'check',
      'generate',
      'review',
      'complete',
      'archive',
    ]

    const sentences = description.split(/[,;.!?]/)

    let taskId = 1
    for (const sentence of sentences) {
      if (tasks.length >= count) break

      const lowerSentence = sentence.toLowerCase().trim()

      for (const verb of actionVerbs) {
        if (lowerSentence.includes(verb)) {
          const taskName = sentence
            .trim()
            .replace(/^(and|or|then)/i, '')
            .trim()
            .substring(0, 50)

          tasks.push({
            id: `task-${taskId++}`,
            name: taskName || `Task ${taskId}`,
            type: 'atomic',
            documentation: `Generated task: ${taskName}`,
          })
          break
        }
      }
    }

    // If no tasks found, create default ones
    while (tasks.length < count) {
      tasks.push({
        id: `task-${taskId++}`,
        name: `Task ${taskId}`,
        type: 'atomic',
        documentation: `Generated task ${taskId}`,
      })
    }

    return tasks
  }

  /**
   * Select appropriate patterns based on workflow characteristics
   */
  private static selectPatterns(
    taskCount: number,
    complexity: string,
    suggestedPatterns: string[]
  ): ControlFlowPattern[] {
    const patterns: ControlFlowPattern[] = ['sequence']

    if (taskCount > 2 || suggestedPatterns.includes('parallel')) {
      patterns.push('parallel', 'synchronization')
    }

    if (taskCount > 4 || suggestedPatterns.includes('choice')) {
      patterns.push('exclusive-choice', 'multiple-merge')
    }

    if (complexity === 'complex' || suggestedPatterns.includes('deferred-choice')) {
      patterns.push('deferred-choice')
    }

    return patterns as ControlFlowPattern[]
  }

  /**
   * Add control flows between tasks
   */
  private static addControlFlows(
    spec: YAWLSpecification,
    tasks: YAWLTask[],
    patterns: ControlFlowPattern[]
  ): void {
    if (tasks.length < 2) return

    // Simple sequential flow with occasional parallel splits
    for (let i = 0; i < tasks.length - 1; i++) {
      const pattern =
        i % 3 === 0 && patterns.includes('parallel') ? 'parallel' : 'sequence'

      WorkflowService.addControlFlow(spec, tasks[i].id, tasks[i + 1].id, pattern as any)
    }
  }

  /**
   * Enhance workflow with AI insights
   */
  static enhanceWithInsights(spec: YAWLSpecification): {
    suggestions: string[]
    improvements: string[]
    recommendations: string[]
  } {
    const recommendations = workflowKnowledgeBase.getRecommendations(
      spec.tasks.length,
      1 // complexity
    )

    return {
      suggestions: recommendations
        .slice(0, 2)
        .map((r) => r.title),
      improvements: recommendations
        .slice(0, 1)
        .flatMap((r) => r.bestPractices.slice(0, 2)),
      recommendations: recommendations
        .slice(0, 3)
        .map((r) => r.description),
    }
  }

  /**
   * Generate variations of a workflow
   */
  static generateVariations(
    spec: YAWLSpecification,
    count = 3
  ): YAWLSpecification[] {
    const variations: YAWLSpecification[] = []

    for (let i = 0; i < count; i++) {
      const variation = {
        ...spec,
        id: `${spec.id}-v${i + 1}`,
        name: `${spec.name} (Variation ${i + 1})`,
        tasks: [...spec.tasks],
        nets: [...spec.nets],
      }

      // Vary the patterns slightly
      if (spec.nets[0]) {
        const patterns: ControlFlowPattern[] = [
          'parallel',
          'choice',
          'synchronization',
        ]
        variation.nets[0] = {
          ...spec.nets[0],
          flows: (spec.nets[0].flows || []).map((flow, idx) => ({
            ...flow,
            pattern: patterns[idx % patterns.length],
          })),
        }
      }

      variations.push(variation)
    }

    return variations
  }
}

export default AIWorkflowGenerationService
