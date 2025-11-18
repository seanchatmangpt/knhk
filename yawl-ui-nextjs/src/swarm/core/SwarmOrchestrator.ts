/**
 * SwarmOrchestrator - Core 2028 AI Agent Swarm Architecture
 * Autonomous Collective Intelligence System
 *
 * DOCTRINE_2027 Alignment:
 * - O (Observation): Continuous telemetry of agent interactions
 * - Σ (Ontology): RDF-based swarm specification
 * - Q (Invariants): Provable swarm safety properties
 * - MAPE-K: Multi-agent learning and evolution
 * - Chatman Constant: ≤8 tick decision latency at swarm level
 */

import { EventEmitter } from 'events'
import { v4 as uuidv4 } from 'uuid'

/**
 * Agent Personality Types
 * Define behavioral archetypes for emergent team formation
 */
export enum AgentPersonality {
  STRATEGIC = 'strategic',      // Long-term planning, risk assessment
  CREATIVE = 'creative',        // Novel ideas, lateral thinking
  ANALYTICAL = 'analytical',    // Data-driven, precision-focused
  EXECUTION = 'execution',      // Fast action, implementation
  DIPLOMATIC = 'diplomatic',    // Consensus-building, mediation
  SKEPTICAL = 'skeptical',      // Devil's advocate, challenge assumptions
}

/**
 * Agent Status Enumeration
 */
export enum AgentStatus {
  IDLE = 'idle',
  THINKING = 'thinking',
  COMMUNICATING = 'communicating',
  EXECUTING = 'executing',
  LEARNING = 'learning',
  FAILED = 'failed',
  RETIRED = 'retired',
}

/**
 * Swarm Decision Types
 */
export enum DecisionType {
  CONSENSUS = 'consensus',              // Must agree
  MAJORITY = 'majority',                // >50% agree
  QUORUM = 'quorum',                    // 66% agree
  ADVERSARIAL = 'adversarial',          // Red vs Blue teams
  WEIGHTED_EXPERT = 'weighted_expert',  // Expert weighting
}

/**
 * Agent Interface - Core swarm unit
 */
export interface IAgent {
  id: string
  name: string
  personality: AgentPersonality
  expertise: Map<string, number>        // domain -> skill level (0-1)
  status: AgentStatus
  reputation: number                    // 0-100, influenced by past decisions
  age: number                           // ticks since creation
  mentorId?: string                     // for succession planning
  memoryCapacity: number                // max experiences retained
  memories: AgentExperience[]           // learned experiences
}

/**
 * Agent Experience - Single learning event
 */
export interface AgentExperience {
  id: string
  type: 'success' | 'failure' | 'observation'
  domain: string
  context: Record<string, unknown>
  outcome: unknown
  confidence: number
  timestamp: number
  propagatedToAgents: string[]          // agents who learned this
}

/**
 * Swarm Communication Message
 */
export interface SwarmMessage {
  id: string
  senderId: string
  type: 'observation' | 'idea' | 'question' | 'decision' | 'learning'
  content: Record<string, unknown>
  timestamp: number
  recipients: string[]                  // '*' = broadcast
  importance: number                    // 0-1
  causality?: string[]                  // message IDs that caused this
}

/**
 * Collective Decision
 */
export interface CollectiveDecision {
  id: string
  topic: string
  decisionType: DecisionType
  proposedBy: string
  agents: Map<string, {
    vote: unknown
    confidence: number
    reasoning: string
    personality: AgentPersonality
  }>
  consensus: unknown
  convergenceTime: number               // ticks to reach decision
  timestamp: number
}

/**
 * Swarm Knowledge (Collective Learning)
 */
export interface SwarmKnowledge {
  domain: string
  pattern: Record<string, unknown>
  successRate: number                   // 0-1
  confidence: number                    // 0-1
  contributors: Set<string>             // agent IDs who contributed
  firstObserved: number
  lastUpdated: number
  applicability: Record<string, number> // scenario -> applicability score
}

/**
 * Core SwarmOrchestrator Class
 * 2028: Autonomous collective intelligence without central control
 */
export class SwarmOrchestrator extends EventEmitter {
  private agents: Map<string, IAgent> = new Map()
  private messages: SwarmMessage[] = []
  private decisions: CollectiveDecision[] = []
  private knowledge: Map<string, SwarmKnowledge> = new Map()

  private currentTick: number = 0
  private tickDuration: number = 1000 / 125  // 8ms per tick (125 ticks/sec = Chatman Constant)
  private swarmId: string = uuidv4()
  private swarmStartTime: number = Date.now()

  // Swarm properties
  private consensusThreshold: number = 0.66
  private emergenceDetector: EmergenceDetector
  private evolutionEngine: EvolutionEngine
  private causalGraph: CausalGraph

  constructor(agentCount: number = 10) {
    super()
    this.emergenceDetector = new EmergenceDetector(this)
    this.evolutionEngine = new EvolutionEngine(this)
    this.causalGraph = new CausalGraph()

    // Initialize swarm with diverse personalities
    this.initializeSwarm(agentCount)
  }

  /**
   * Initialize swarm with diverse agent personalities
   * Creates natural diversity for better problem-solving
   */
  private initializeSwarm(agentCount: number): void {
    const personalities = Object.values(AgentPersonality)

    for (let i = 0; i < agentCount; i++) {
      const personality = personalities[i % personalities.length]
      const agent: IAgent = {
        id: uuidv4(),
        name: `Agent-${personality}-${i}`,
        personality,
        expertise: new Map([
          ['problem-solving', 0.5],
          ['communication', 0.6],
          ['learning', 0.7],
          [personality, 0.8],
        ]),
        status: AgentStatus.IDLE,
        reputation: 75,
        age: 0,
        memoryCapacity: 100,
        memories: [],
      }

      this.agents.set(agent.id, agent)
      this.emit('agent:created', { agent, swarmId: this.swarmId })
    }
  }

  /**
   * Core Swarm Cognitive Cycle
   * Single tick of swarm thinking (MAPE-K style)
   */
  public async tick(): Promise<void> {
    const tickStart = Date.now()

    // M: Monitor - gather observations
    await this.monitorPhase()

    // A: Analyze - detect patterns and anomalies
    await this.analyzePhase()

    // P: Plan - propose ideas and solutions
    await this.planPhase()

    // E: Execute - take collective action
    await this.executePhase()

    // K: Knowledge - update collective memory
    await this.knowledgePhase()

    // Detect emergent behaviors
    const emergentBehaviors = this.emergenceDetector.detect()
    if (emergentBehaviors.length > 0) {
      this.emit('emergence:detected', { behaviors: emergentBehaviors, tick: this.currentTick })
    }

    // Enforce Chatman Constant
    const tickDuration = Date.now() - tickStart
    if (tickDuration > 8) {  // 8ms = one tick
      console.warn(`Tick ${this.currentTick} exceeded Chatman Constant: ${tickDuration}ms`)
    }

    this.currentTick++
  }

  /**
   * Monitor Phase - Agents observe and report
   */
  private async monitorPhase(): Promise<void> {
    for (const agent of this.agents.values()) {
      if (agent.status === AgentStatus.IDLE || agent.status === AgentStatus.THINKING) {
        agent.status = AgentStatus.THINKING

        // Each agent observes their domain
        const observations = this.generateObservations(agent)

        for (const obs of observations) {
          const message: SwarmMessage = {
            id: uuidv4(),
            senderId: agent.id,
            type: 'observation',
            content: obs,
            timestamp: this.currentTick,
            recipients: ['*'],  // broadcast to all
            importance: obs.importance || 0.5,
          }

          this.messages.push(message)
          this.causalGraph.addMessage(message)
        }
      }
    }
  }

  /**
   * Analyze Phase - Detect patterns, anomalies, causality
   */
  private async analyzePhase(): Promise<void> {
    // Analyze messages for patterns
    const recentMessages = this.messages.slice(-50)

    for (const agent of this.agents.values()) {
      if (agent.personality === AgentPersonality.ANALYTICAL) {
        const patterns = this.detectPatterns(recentMessages)
        const anomalies = this.detectAnomalies(recentMessages)

        if (patterns.length > 0 || anomalies.length > 0) {
          const message: SwarmMessage = {
            id: uuidv4(),
            senderId: agent.id,
            type: 'observation',
            content: { patterns, anomalies },
            timestamp: this.currentTick,
            recipients: ['*'],
            importance: 0.8,
          }

          this.messages.push(message)
        }
      }
    }

    // Causal inference
    this.causalGraph.inferCausality(recentMessages)
  }

  /**
   * Plan Phase - Generate and discuss ideas
   */
  private async planPhase(): Promise<void> {
    for (const agent of this.agents.values()) {
      if (agent.personality === AgentPersonality.CREATIVE) {
        const ideas = this.generateIdeas(agent)

        for (const idea of ideas) {
          const message: SwarmMessage = {
            id: uuidv4(),
            senderId: agent.id,
            type: 'idea',
            content: idea,
            timestamp: this.currentTick,
            recipients: ['*'],
            importance: Math.random(),
          }

          this.messages.push(message)
        }
      }
    }
  }

  /**
   * Execute Phase - Take collective action
   */
  private async executePhase(): Promise<void> {
    // Find actionable decisions
    const decisions = this.messages.filter(m => m.type === 'decision')

    for (const decision of decisions) {
      const action = await this.getConsensusDecision(decision)

      if (action) {
        this.emit('action:executed', {
          action,
          tick: this.currentTick,
          agents: this.agents.size,
        })
      }
    }
  }

  /**
   * Knowledge Phase - Update collective memory
   */
  private async knowledgePhase(): Promise<void> {
    for (const agent of this.agents.values()) {
      if (agent.status === AgentStatus.THINKING) {
        // Agent learns from recent messages
        const newExperiences = this.extractExperiences(agent, this.messages.slice(-20))

        for (const exp of newExperiences) {
          agent.memories.push(exp)

          // Keep only recent memories (bounded)
          if (agent.memories.length > agent.memoryCapacity) {
            agent.memories.shift()
          }

          // Share with other agents (collective learning)
          this.propagateExperience(agent, exp)
        }

        // Update reputation based on contribution quality
        this.updateReputation(agent)
        agent.status = AgentStatus.IDLE
      }
    }
  }

  /**
   * Get Consensus Decision from swarm
   * Implements democratic collective intelligence
   */
  private async getConsensusDecision(proposal: SwarmMessage): Promise<unknown> {
    const decision: CollectiveDecision = {
      id: uuidv4(),
      topic: proposal.content.topic as string,
      decisionType: DecisionType.CONSENSUS,
      proposedBy: proposal.senderId,
      agents: new Map(),
      consensus: undefined,
      convergenceTime: 0,
      timestamp: this.currentTick,
    }

    const startTick = this.currentTick

    // Each agent votes
    for (const agent of this.agents.values()) {
      const vote = this.agentVote(agent, proposal)

      decision.agents.set(agent.id, {
        vote,
        confidence: Math.random() * 0.5 + 0.5,
        reasoning: `${agent.personality}-perspective`,
        personality: agent.personality,
      })
    }

    // Calculate consensus
    const votes = Array.from(decision.agents.values())
    const agreeCount = votes.filter(v => v.vote === true).length
    const agreePercentage = agreeCount / votes.length

    decision.convergenceTime = this.currentTick - startTick
    decision.consensus = agreePercentage >= this.consensusThreshold

    this.decisions.push(decision)
    this.emit('decision:made', { decision, swarmId: this.swarmId })

    return decision.consensus ? proposal.content.action : null
  }

  /**
   * Agent voting mechanism
   * Combines personality with domain expertise
   */
  private agentVote(agent: IAgent, proposal: SwarmMessage): boolean {
    // Personality influences voting
    const personalityBias = this.getPersonalityBias(agent.personality)

    // Domain expertise influences voting
    const topic = proposal.content.topic as string
    const expertise = agent.expertise.get(topic) || 0.5

    // Reputation influences confidence
    const confidenceModifier = (agent.reputation / 100) * 0.3

    const finalScore = (expertise * 0.5) + (personalityBias * 0.3) + (confidenceModifier * 0.2)

    return finalScore > 0.5
  }

  /**
   * Get personality bias in decision-making
   */
  private getPersonalityBias(personality: AgentPersonality): number {
    const biases: Record<AgentPersonality, number> = {
      [AgentPersonality.STRATEGIC]: 0.7,
      [AgentPersonality.CREATIVE]: 0.8,
      [AgentPersonality.ANALYTICAL]: 0.6,
      [AgentPersonality.EXECUTION]: 0.75,
      [AgentPersonality.DIPLOMATIC]: 0.65,
      [AgentPersonality.SKEPTICAL]: 0.4,
    }
    return biases[personality]
  }

  /**
   * Generate observations for agent
   */
  private generateObservations(agent: IAgent): Record<string, unknown>[] {
    return [
      {
        type: 'agent_status',
        agentId: agent.id,
        status: agent.status,
        importance: 0.3,
      },
      {
        type: 'expertise_assessment',
        agent: agent.id,
        topSkills: Array.from(agent.expertise.entries())
          .sort((a, b) => b[1] - a[1])
          .slice(0, 3)
          .map(([domain, level]) => ({ domain, level })),
        importance: 0.5,
      },
    ]
  }

  /**
   * Detect patterns in messages
   */
  private detectPatterns(messages: SwarmMessage[]): Record<string, unknown>[] {
    const patterns: Record<string, unknown>[] = []

    // Simple pattern: message frequency
    const senderFrequency = new Map<string, number>()
    for (const msg of messages) {
      senderFrequency.set(msg.senderId, (senderFrequency.get(msg.senderId) || 0) + 1)
    }

    for (const [senderId, count] of senderFrequency) {
      if (count > 10) {
        patterns.push({
          type: 'high_activity',
          agent: senderId,
          messageCount: count,
        })
      }
    }

    return patterns
  }

  /**
   * Detect anomalies in swarm behavior
   */
  private detectAnomalies(messages: SwarmMessage[]): Record<string, unknown>[] {
    const anomalies: Record<string, unknown>[] = []

    // Detect unusual message importance
    const importanceAvg = messages.reduce((sum, m) => sum + m.importance, 0) / messages.length
    const importanceStdDev = Math.sqrt(
      messages.reduce((sum, m) => sum + Math.pow(m.importance - importanceAvg, 2), 0) / messages.length
    )

    for (const msg of messages) {
      if (Math.abs(msg.importance - importanceAvg) > 2 * importanceStdDev) {
        anomalies.push({
          type: 'unusual_importance',
          messageId: msg.id,
          importance: msg.importance,
          expectedRange: [importanceAvg - 2 * importanceStdDev, importanceAvg + 2 * importanceStdDev],
        })
      }
    }

    return anomalies
  }

  /**
   * Generate creative ideas
   */
  private generateIdeas(agent: IAgent): Record<string, unknown>[] {
    return [
      {
        id: uuidv4(),
        type: 'optimization_suggestion',
        content: `Idea from ${agent.name}`,
        domain: 'workflow_optimization',
        novelty: Math.random(),
      },
    ]
  }

  /**
   * Extract experiences from messages
   */
  private extractExperiences(agent: IAgent, messages: SwarmMessage[]): AgentExperience[] {
    return messages
      .filter(m => m.type === 'observation' || m.type === 'decision')
      .map(m => ({
        id: uuidv4(),
        type: m.type === 'decision' ? 'success' : 'observation',
        domain: (m.content.domain as string) || 'general',
        context: m.content,
        outcome: m.content.result || null,
        confidence: m.importance,
        timestamp: m.timestamp,
        propagatedToAgents: [],
      }))
  }

  /**
   * Propagate experience to other agents
   * Collective learning - one agent's breakthrough becomes swarm knowledge
   */
  private propagateExperience(sourceAgent: IAgent, experience: AgentExperience): void {
    // Share with agents of similar personality first
    for (const agent of this.agents.values()) {
      if (agent.id !== sourceAgent.id && agent.status === AgentStatus.IDLE) {
        // Probability of learning based on personality match and relevance
        const learnProbability = this.calculateLearnProbability(
          agent.personality,
          sourceAgent.personality,
          experience.domain,
          agent.expertise
        )

        if (Math.random() < learnProbability) {
          agent.memories.push({
            ...experience,
            propagatedToAgents: [],
          })
          experience.propagatedToAgents.push(agent.id)
        }
      }
    }
  }

  /**
   * Calculate probability agent will learn from another's experience
   */
  private calculateLearnProbability(
    learnerPersonality: AgentPersonality,
    teacherPersonality: AgentPersonality,
    domain: string,
    learnerExpertise: Map<string, number>
  ): number {
    // Base probability from personality compatibility
    const personalityCompatibility = this.getPersonalityCompatibility(learnerPersonality, teacherPersonality)

    // Domain relevance
    const domainExpertise = learnerExpertise.get(domain) || 0.3
    const domainRelevance = 1 - domainExpertise  // More likely to learn in weak domains

    return personalityCompatibility * 0.6 + domainRelevance * 0.4
  }

  /**
   * Get personality compatibility matrix
   */
  private getPersonalityCompatibility(from: AgentPersonality, to: AgentPersonality): number {
    const compatibilityMatrix: Record<AgentPersonality, Record<AgentPersonality, number>> = {
      [AgentPersonality.STRATEGIC]: { [AgentPersonality.STRATEGIC]: 0.9, [AgentPersonality.CREATIVE]: 0.7, [AgentPersonality.ANALYTICAL]: 0.8, [AgentPersonality.EXECUTION]: 0.8, [AgentPersonality.DIPLOMATIC]: 0.6, [AgentPersonality.SKEPTICAL]: 0.5 },
      [AgentPersonality.CREATIVE]: { [AgentPersonality.STRATEGIC]: 0.7, [AgentPersonality.CREATIVE]: 0.8, [AgentPersonality.ANALYTICAL]: 0.6, [AgentPersonality.EXECUTION]: 0.7, [AgentPersonality.DIPLOMATIC]: 0.8, [AgentPersonality.SKEPTICAL]: 0.6 },
      [AgentPersonality.ANALYTICAL]: { [AgentPersonality.STRATEGIC]: 0.8, [AgentPersonality.CREATIVE]: 0.6, [AgentPersonality.ANALYTICAL]: 0.95, [AgentPersonality.EXECUTION]: 0.7, [AgentPersonality.DIPLOMATIC]: 0.5, [AgentPersonality.SKEPTICAL]: 0.8 },
      [AgentPersonality.EXECUTION]: { [AgentPersonality.STRATEGIC]: 0.8, [AgentPersonality.CREATIVE]: 0.7, [AgentPersonality.ANALYTICAL]: 0.7, [AgentPersonality.EXECUTION]: 0.9, [AgentPersonality.DIPLOMATIC]: 0.7, [AgentPersonality.SKEPTICAL]: 0.4 },
      [AgentPersonality.DIPLOMATIC]: { [AgentPersonality.STRATEGIC]: 0.6, [AgentPersonality.CREATIVE]: 0.8, [AgentPersonality.ANALYTICAL]: 0.5, [AgentPersonality.EXECUTION]: 0.7, [AgentPersonality.DIPLOMATIC]: 0.95, [AgentPersonality.SKEPTICAL]: 0.5 },
      [AgentPersonality.SKEPTICAL]: { [AgentPersonality.STRATEGIC]: 0.5, [AgentPersonality.CREATIVE]: 0.6, [AgentPersonality.ANALYTICAL]: 0.8, [AgentPersonality.EXECUTION]: 0.4, [AgentPersonality.DIPLOMATIC]: 0.5, [AgentPersonality.SKEPTICAL]: 0.85 },
    }

    return compatibilityMatrix[from]?.[to] || 0.5
  }

  /**
   * Update agent reputation based on contribution quality
   */
  private updateReputation(agent: IAgent): void {
    const reputationDecay = 0.95  // Decay old reputation

    // Calculate contribution quality from recent messages
    const agentMessages = this.messages.filter(m => m.senderId === agent.id).slice(-10)

    if (agentMessages.length > 0) {
      const avgImportance = agentMessages.reduce((sum, m) => sum + m.importance, 0) / agentMessages.length
      const impact = avgImportance * 100

      agent.reputation = agent.reputation * reputationDecay + impact * (1 - reputationDecay)
    }
  }

  /**
   * Get swarm statistics
   */
  public getSwarmStats() {
    return {
      swarmId: this.swarmId,
      currentTick: this.currentTick,
      agentCount: this.agents.size,
      messageCount: this.messages.length,
      decisionCount: this.decisions.length,
      knowledgeBaseSize: this.knowledge.size,
      averageReputation: Array.from(this.agents.values()).reduce((sum, a) => sum + a.reputation, 0) / this.agents.size,
      uptime: (Date.now() - this.swarmStartTime) / 1000,
    }
  }

  /**
   * Get agent by ID
   */
  public getAgent(id: string): IAgent | undefined {
    return this.agents.get(id)
  }

  /**
   * Get all agents
   */
  public getAgents(): IAgent[] {
    return Array.from(this.agents.values())
  }

  /**
   * Get swarm decisions
   */
  public getDecisions(): CollectiveDecision[] {
    return this.decisions
  }

  /**
   * Get swarm knowledge
   */
  public getKnowledge(): Map<string, SwarmKnowledge> {
    return this.knowledge
  }
}

/**
 * Emergence Detector
 * Detects emergent behaviors that exceed individual agent capabilities
 */
class EmergenceDetector {
  constructor(private orchestrator: SwarmOrchestrator) {}

  detect(): string[] {
    const emergentBehaviors: string[] = []
    const agents = this.orchestrator.getAgents()

    // Detect collective intelligence (swarm > individuals)
    const avgReputation = agents.reduce((sum, a) => sum + a.reputation, 0) / agents.length
    if (avgReputation > 80) {
      emergentBehaviors.push('collective_intelligence_emerging')
    }

    // Detect self-organization
    const personalityDistribution = new Map<AgentPersonality, number>()
    for (const agent of agents) {
      personalityDistribution.set(
        agent.personality,
        (personalityDistribution.get(agent.personality) || 0) + 1
      )
    }

    if (personalityDistribution.size === Object.values(AgentPersonality).length) {
      emergentBehaviors.push('diverse_team_formation')
    }

    return emergentBehaviors
  }
}

/**
 * Evolution Engine
 * Drives swarm evolution through learning and adaptation
 */
class EvolutionEngine {
  constructor(private orchestrator: SwarmOrchestrator) {}

  evolveSwarm(): void {
    const agents = this.orchestrator.getAgents()

    // Agent specialization: let agents drift toward specialization
    for (const agent of agents) {
      // Update expertise based on past successes
      for (const memory of agent.memories) {
        if (memory.type === 'success') {
          const expertise = agent.expertise.get(memory.domain) || 0
          agent.expertise.set(memory.domain, Math.min(expertise + 0.05, 1.0))
        }
      }
    }
  }
}

/**
 * Causal Graph
 * Tracks causal relationships in swarm decisions
 */
class CausalGraph {
  private edges: Map<string, Set<string>> = new Map()
  private messageHistory: SwarmMessage[] = []

  addMessage(message: SwarmMessage): void {
    this.messageHistory.push(message)
  }

  inferCausality(messages: SwarmMessage[]): void {
    // Simple causality: messages that reference previous message IDs
    for (const msg of messages) {
      if (msg.causality) {
        for (const cause of msg.causality) {
          const effects = this.edges.get(cause) || new Set()
          effects.add(msg.id)
          this.edges.set(cause, effects)
        }
      }
    }
  }

  getCausalChain(messageId: string): string[] {
    const chain: string[] = [messageId]
    const effects = this.edges.get(messageId) || new Set()

    for (const effect of effects) {
      chain.push(...this.getCausalChain(effect))
    }

    return chain
  }
}

export { EmergenceDetector, EvolutionEngine, CausalGraph }
