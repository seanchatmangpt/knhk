/**
 * Workflow Knowledge Base - RAG System
 * Retrieval-Augmented Generation for YAWL workflow knowledge
 * Aligned with DOCTRINE Σ (Ontology/Schema) principle
 */

import * as N3 from 'n3'

export interface WorkflowKnowledge {
  id: string
  title: string
  description: string
  patterns: string[]
  bestPractices: string[]
  rdfTriples: N3.Quad[]
  embedding?: number[]
}

/**
 * Knowledge base for workflow patterns and best practices
 */
export class WorkflowKnowledgeBase {
  private knowledge: Map<string, WorkflowKnowledge> = new Map()
  private index: Map<string, Set<string>> = new Map() // Term -> Document IDs

  constructor() {
    this.initializeDefaultKnowledge()
  }

  private initializeDefaultKnowledge(): void {
    // Order Processing Workflow
    this.addKnowledge({
      id: 'order-process',
      title: 'Order Processing Workflow',
      description: 'Standard e-commerce order processing',
      patterns: ['sequence', 'choice', 'parallel'],
      bestPractices: [
        'Validate order before processing',
        'Use parallel for payment and inventory check',
        'Synchronize payment and shipping',
        'Include error handling for failed payments',
      ],
      rdfTriples: [],
    })

    // Approval Workflow
    this.addKnowledge({
      id: 'approval-flow',
      title: 'Approval Workflow',
      description: 'Multi-level approval process',
      patterns: ['sequence', 'choice'],
      bestPractices: [
        'Route based on approval amount',
        'Set escalation timeouts',
        'Include audit trail',
        'Support delegation',
      ],
      rdfTriples: [],
    })

    // Parallel Tasks Workflow
    this.addKnowledge({
      id: 'parallel-tasks',
      title: 'Parallel Processing',
      description: 'Execute multiple tasks simultaneously',
      patterns: ['parallel', 'synchronization', 'multiple-merge'],
      bestPractices: [
        'Use parallel split to initiate',
        'Synchronize at completion point',
        'Monitor all parallel branches',
        'Handle partial failures gracefully',
      ],
      rdfTriples: [],
    })

    // Complex Decision Flow
    this.addKnowledge({
      id: 'complex-decisions',
      title: 'Complex Decision Trees',
      description: 'Multi-condition decision logic',
      patterns: ['exclusive-choice', 'implicit-choice', 'deferred-choice'],
      bestPractices: [
        'Use exclusive-choice for mutually exclusive paths',
        'Consider deferred-choice for uncertain conditions',
        'Include default path for unmapped conditions',
        'Document decision logic clearly',
      ],
      rdfTriples: [],
    })
  }

  /**
   * Add knowledge to the base
   */
  addKnowledge(knowledge: WorkflowKnowledge): void {
    this.knowledge.set(knowledge.id, knowledge)

    // Index by terms
    const terms = [
      ...knowledge.title.toLowerCase().split(/\s+/),
      ...knowledge.patterns,
      ...knowledge.description.toLowerCase().split(/\s+/),
    ]

    terms.forEach((term) => {
      if (!this.index.has(term)) {
        this.index.set(term, new Set())
      }
      this.index.get(term)!.add(knowledge.id)
    })
  }

  /**
   * Search knowledge base by terms (BM25-like ranking)
   */
  search(query: string, limit = 3): WorkflowKnowledge[] {
    const terms = query.toLowerCase().split(/\s+/)
    const scores: Map<string, number> = new Map()

    // Score documents by term frequency
    terms.forEach((term) => {
      const docs = this.index.get(term) || new Set()
      docs.forEach((docId) => {
        scores.set(docId, (scores.get(docId) || 0) + 1)
      })
    })

    // Return top results
    return Array.from(scores.entries())
      .sort((a, b) => b[1] - a[1])
      .slice(0, limit)
      .map(([id]) => this.knowledge.get(id)!)
      .filter((k): k is WorkflowKnowledge => k !== undefined)
  }

  /**
   * Get patterns used in similar workflows
   */
  getPatternsForScenario(scenario: string): string[] {
    const results = this.search(scenario, 5)
    const patterns = new Set<string>()
    results.forEach((r) => r.patterns.forEach((p) => patterns.add(p)))
    return Array.from(patterns)
  }

  /**
   * Get best practices for scenario
   */
  getBestPractices(scenario: string): string[] {
    const results = this.search(scenario, 3)
    const practices = new Set<string>()
    results.forEach((r) =>
      r.bestPractices.forEach((p) => practices.add(p))
    )
    return Array.from(practices)
  }

  /**
   * Get workflow recommendations
   */
  getRecommendations(
    taskCount: number,
    complexity: number
  ): WorkflowKnowledge[] {
    return Array.from(this.knowledge.values()).filter((k) => {
      const patternCount = k.patterns.length
      return (
        patternCount >= Math.floor(taskCount / 3) &&
        patternCount <= Math.ceil(taskCount)
      )
    })
  }

  /**
   * Export as RDF/Turtle
   */
  exportAsTurtle(): string {
    const writer = new N3.Writer({
      prefixes: {
        yawl: 'http://www.yawlfoundation.org/yawl/',
        kb: 'http://example.com/knowledge/',
        rdfs: 'http://www.w3.org/2000/01/rdf-schema#',
        rdf: 'http://www.w3.org/1999/02/22-rdf-syntax-ns#',
      },
    })

    this.knowledge.forEach((knowledge) => {
      const kbIRI = `http://example.com/knowledge/${knowledge.id}`

      // Add knowledge description
      writer.addQuad(
        N3.DataFactory.namedNode(kbIRI),
        N3.DataFactory.namedNode('http://www.w3.org/2000/01/rdf-schema#label'),
        N3.DataFactory.literal(knowledge.title)
      )

      writer.addQuad(
        N3.DataFactory.namedNode(kbIRI),
        N3.DataFactory.namedNode('http://www.w3.org/2000/01/rdf-schema#comment'),
        N3.DataFactory.literal(knowledge.description)
      )

      // Add patterns
      knowledge.patterns.forEach((pattern) => {
        writer.addQuad(
          N3.DataFactory.namedNode(kbIRI),
          N3.DataFactory.namedNode('http://www.yawlfoundation.org/yawl/hasPattern'),
          N3.DataFactory.literal(pattern)
        )
      })
    })

    return writer.quads.map((q) => q.toString()).join('\n')
  }

  /**
   * Get all knowledge
   */
  getAll(): WorkflowKnowledge[] {
    return Array.from(this.knowledge.values())
  }
}

// Export singleton
export const workflowKnowledgeBase = new WorkflowKnowledgeBase()

export default WorkflowKnowledgeBase
