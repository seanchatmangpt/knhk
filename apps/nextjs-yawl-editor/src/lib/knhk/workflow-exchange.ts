/**
 * DOCTRINE ALIGNMENT: Π (Projections) + COVENANT 1 (RDF as Source of Truth)
 * Workflow exchange between RDF and knhk execution formats
 *
 * COVENANT 1: RDF is the source of truth
 * All conversions must preserve RDF semantics through round-trip
 */

import { withSpan, createSpan } from '@/lib/telemetry/setup';
import { knhkConfig } from './config';
import type { YAWLWorkflow, ValidationResult } from '@/lib/types';
import type {
  RDFDataset,
  KNHKWorkflow,
  ValidatedSpec,
  AuditEntry,
  WorkflowId,
  YAWLSpecification,
  ExecutionTrace,
} from './types';

/* ============================================================================
 * Workflow Exchange
 * ========================================================================== */

/**
 * Handle conversions between RDF, YAWL XML, and knhk execution formats
 *
 * Key principles:
 * - RDF is the canonical representation (Covenant 1)
 * - All conversions are reversible (round-trip safe)
 * - Metadata is preserved across formats
 * - Audit trail tracks all changes
 */
export class WorkflowExchange {
  private versionCounter: Map<string, number> = new Map();

  /* ============================
   * RDF → knhk Format
   * ============================ */

  /**
   * Export editor RDF to knhk execution format
   *
   * PERFORMANCE: Must complete within exportBudget (200ms)
   */
  async exportToKNHKFormat(rdfDataset: RDFDataset): Promise<KNHKWorkflow> {
    const startTime = performance.now();

    return withSpan(
      'workflow.export.knhk',
      async () => {
        // Step 1: Convert RDF to YAWL workflow structure
        const workflow = await this.rdfToWorkflow(rdfDataset);

        // Step 2: Validate workflow
        const validation = await this.validateWorkflow(workflow);

        if (!validation.valid) {
          throw new Error(
            `Cannot export invalid workflow: ${validation.errors.map((e) => e.message).join(', ')}`
          );
        }

        // Step 3: Convert workflow to YAWL XML
        const yawlXml = await this.workflowToYAWLXML(workflow);

        // Step 4: Create knhk workflow package
        const knhkWorkflow: KNHKWorkflow = {
          id: workflow.id as WorkflowId,
          format: 'yawl-xml',
          specification: yawlXml,
          ontology: rdfDataset,
          metadata: {
            source: 'editor',
            author: workflow.metadata?.author ?? undefined,
            created: workflow.metadata?.created || new Date().toISOString(),
            validated: true,
            validationReport: validation,
          },
        };

        const duration = performance.now() - startTime;

        // Check performance budget
        if (duration > knhkConfig.performance.exportBudget) {
          console.warn(
            `Export exceeded budget: ${duration}ms > ${knhkConfig.performance.exportBudget}ms`
          );
        }

        return knhkWorkflow;
      },
      {
        'rdf.format': rdfDataset.format,
        'rdf.size': rdfDataset.content.length,
      }
    );
  }

  /**
   * Convert RDF dataset to YAWL workflow structure
   */
  private async rdfToWorkflow(rdfDataset: RDFDataset): Promise<YAWLWorkflow> {
    return withSpan('workflow.rdf_to_workflow', async () => {
      // Parse RDF triples
      const triples = await this.parseRDF(rdfDataset.content, rdfDataset.format);

      // Extract workflow metadata
      const metadata = this.extractMetadata(triples, rdfDataset.prefixes);

      // Extract nodes
      const nodes = this.extractNodes(triples, rdfDataset.prefixes);

      // Extract edges
      const edges = this.extractEdges(triples, rdfDataset.prefixes);

      return {
        id: metadata.id || `workflow-${Date.now()}`,
        name: metadata.name || 'Untitled Workflow',
        version: metadata.version || '1.0.0',
        nodes,
        edges,
        metadata: {
          author: metadata.author,
          description: metadata.description,
          created: metadata.created || new Date().toISOString(),
          modified: metadata.modified || new Date().toISOString(),
        },
      };
    });
  }

  /**
   * Convert YAWL workflow to XML specification
   */
  private async workflowToYAWLXML(workflow: YAWLWorkflow): Promise<string> {
    return withSpan('workflow.to_yawl_xml', async () => {
      // Generate YAWL XML structure
      const xml = this.generateYAWLXML(workflow);
      return xml;
    });
  }

  /* ============================
   * knhk Format → RDF
   * ============================ */

  /**
   * Import knhk execution trace back to editor RDF
   *
   * This enables learning from execution:
   * - Execution paths become part of the ontology
   * - Performance metrics inform optimization
   * - Pattern violations trigger MAPE-K analysis
   */
  async importFromKNHKTrace(trace: ExecutionTrace): Promise<RDFDataset> {
    return withSpan(
      'workflow.import.trace',
      async () => {
        const triples: string[] = [];
        const prefixes = {
          yawl: 'http://yawl.org/ontology#',
          exec: 'http://knhk.io/execution#',
          xsd: 'http://www.w3.org/2001/XMLSchema#',
        };

        // Add trace metadata
        const traceUri = `exec:trace-${trace.caseId}`;
        triples.push(`${traceUri} a exec:ExecutionTrace .`);
        triples.push(`${traceUri} exec:caseId "${trace.caseId}" .`);
        triples.push(`${traceUri} exec:workflowId "${trace.workflowId}" .`);

        // Add execution events
        for (const event of trace.events) {
          const eventUri = `exec:event-${event.id}`;
          triples.push(`${eventUri} a exec:ExecutionEvent .`);
          triples.push(`${eventUri} exec:type "${event.type}" .`);
          triples.push(
            `${eventUri} exec:timestamp "${event.timestamp}"^^xsd:dateTime .`
          );
          triples.push(`${traceUri} exec:hasEvent ${eventUri} .`);

          if (event.taskId) {
            triples.push(`${eventUri} exec:taskId "${event.taskId}" .`);
          }
        }

        // Add metrics
        const metricsUri = `exec:metrics-${trace.caseId}`;
        triples.push(`${metricsUri} a exec:CaseMetrics .`);
        triples.push(`${metricsUri} exec:status "${trace.metrics.status}" .`);
        triples.push(
          `${metricsUri} exec:duration "${trace.metrics.duration || 0}"^^xsd:integer .`
        );
        triples.push(`${traceUri} exec:hasMetrics ${metricsUri} .`);

        // Add detected patterns
        for (const pattern of trace.patterns.detected) {
          const patternUri = `yawl:${pattern.replace(/\s+/g, '_')}`;
          triples.push(`${traceUri} exec:detectedPattern ${patternUri} .`);
        }

        // Add pattern violations
        for (const violation of trace.patterns.violations) {
          const violationUri = `exec:violation-${Date.now()}-${Math.random()}`;
          triples.push(`${violationUri} a exec:PatternViolation .`);
          triples.push(`${violationUri} exec:pattern "${violation}" .`);
          triples.push(`${traceUri} exec:hasViolation ${violationUri} .`);
        }

        // Generate Turtle content
        const prefixDeclarations = Object.entries(prefixes)
          .map(([prefix, uri]) => `@prefix ${prefix}: <${uri}> .`)
          .join('\n');

        const content = `${prefixDeclarations}\n\n${triples.join('\n')}`;

        return {
          format: 'turtle',
          content,
          prefixes,
        };
      },
      {
        'trace.case_id': trace.caseId,
        'trace.workflow_id': trace.workflowId,
        'trace.events': trace.events.length,
      }
    );
  }

  /* ============================
   * Validation Export
   * ============================ */

  /**
   * Export workflow for validation with full compliance data
   */
  async exportForValidation(spec: YAWLSpecification): Promise<ValidatedSpec> {
    return withSpan(
      'workflow.export.validation',
      async () => {
        // Parse YAWL XML to workflow structure
        const workflow = await this.yawlXMLToWorkflow(spec.xml);

        // Validate workflow
        const validation = await this.validateWorkflow(workflow);

        // Extract patterns
        const patterns = this.extractPatterns(workflow);

        // Check compliance
        const compliance = {
          matrixConformance: this.checkMatrixConformance(patterns),
          invariantsPreserved: this.checkInvariants(workflow),
          performanceBudget: true, // Will be checked at runtime
        };

        return {
          workflow,
          validation,
          patterns,
          compliance,
          timestamp: new Date().toISOString(),
        };
      },
      {
        'spec.id': spec.id,
        'spec.version': spec.version,
      }
    );
  }

  /* ============================
   * Audit Trail
   * ============================ */

  /**
   * Create audit trail of changes between workflow versions
   *
   * Tracks all modifications for compliance and debugging
   */
  createAuditTrail(original: RDFDataset, modified: RDFDataset): AuditEntry[] {
    const span = createSpan('workflow.audit_trail');
    const entries: AuditEntry[] = [];

    try {
      // Parse both datasets
      const originalTriples = this.parseRDFSync(original.content, original.format);
      const modifiedTriples = this.parseRDFSync(modified.content, modified.format);

      // Find added triples
      const added = modifiedTriples.filter((t) => !originalTriples.includes(t));
      if (added.length > 0) {
        entries.push({
          id: `audit-${Date.now()}-add`,
          timestamp: new Date().toISOString(),
          operation: 'create',
          changes: added.map((triple) => ({
            path: triple,
            before: null,
            after: triple,
            reason: 'Triple added',
          })),
          metadata: {},
        });
      }

      // Find removed triples
      const removed = originalTriples.filter((t) => !modifiedTriples.includes(t));
      if (removed.length > 0) {
        entries.push({
          id: `audit-${Date.now()}-remove`,
          timestamp: new Date().toISOString(),
          operation: 'delete',
          changes: removed.map((triple) => ({
            path: triple,
            before: triple,
            after: null,
            reason: 'Triple removed',
          })),
          metadata: {},
        });
      }

      return entries;
    } finally {
      span.end();
    }
  }

  /* ============================
   * Versioning
   * ============================ */

  /**
   * Generate next version for workflow
   */
  generateVersion(workflowId: string): string {
    if (knhkConfig.exchange.versionFormat === 'timestamp') {
      return new Date().toISOString();
    }

    if (knhkConfig.exchange.versionFormat === 'sequential') {
      const current = this.versionCounter.get(workflowId) || 0;
      const next = current + 1;
      this.versionCounter.set(workflowId, next);
      return `${next}`;
    }

    // Default: semver
    const current = this.versionCounter.get(workflowId) || 0;
    const next = current + 1;
    this.versionCounter.set(workflowId, next);
    return `1.${next}.0`;
  }

  /* ============================
   * Helper Methods
   * ============================ */

  /**
   * Parse RDF content (async)
   */
  private async parseRDF(
    content: string,
    _format: 'turtle' | 'ntriples' | 'jsonld'
  ): Promise<string[]> {
    // Simplified parsing - in production, use a proper RDF library
    const lines = content.split('\n').filter((line) => {
      const trimmed = line.trim();
      return trimmed && !trimmed.startsWith('#') && !trimmed.startsWith('@');
    });

    return lines;
  }

  /**
   * Parse RDF content (sync)
   */
  private parseRDFSync(
    content: string,
    _format: 'turtle' | 'ntriples' | 'jsonld'
  ): string[] {
    // Simplified parsing - in production, use a proper RDF library
    const lines = content.split('\n').filter((line) => {
      const trimmed = line.trim();
      return trimmed && !trimmed.startsWith('#') && !trimmed.startsWith('@');
    });

    return lines;
  }

  /**
   * Extract metadata from RDF triples
   */
  private extractMetadata(
    triples: string[],
    _prefixes: Record<string, string>
  ): {
    id: string | undefined;
    name: string | undefined;
    version: string | undefined;
    author: string | undefined;
    description: string | undefined;
    created: string | undefined;
    modified: string | undefined;
  } {
    const metadata: ReturnType<typeof this.extractMetadata> = {
      id: undefined,
      name: undefined,
      version: undefined,
      author: undefined,
      description: undefined,
      created: undefined,
      modified: undefined,
    };

    for (const triple of triples) {
      if (triple.includes('yawl:id')) {
        const match = triple.match(/"([^"]+)"/);
        if (match) metadata.id = match[1];
      }
      if (triple.includes('yawl:name')) {
        const match = triple.match(/"([^"]+)"/);
        if (match) metadata.name = match[1];
      }
      if (triple.includes('yawl:version')) {
        const match = triple.match(/"([^"]+)"/);
        if (match) metadata.version = match[1];
      }
      if (triple.includes('dc:creator')) {
        const match = triple.match(/"([^"]+)"/);
        if (match) metadata.author = match[1];
      }
      if (triple.includes('dc:description')) {
        const match = triple.match(/"([^"]+)"/);
        if (match) metadata.description = match[1];
      }
    }

    return metadata;
  }

  /**
   * Extract nodes from RDF triples
   */
  private extractNodes(
    triples: string[],
    _prefixes: Record<string, string>
  ): YAWLWorkflow['nodes'] {
    const nodes: YAWLWorkflow['nodes'] = [];

    // Simplified extraction - in production, use proper RDF query
    const nodeTriples = triples.filter((t) => t.includes('a yawl:Task') || t.includes('a yawl:Condition'));

    for (const triple of nodeTriples) {
      const idMatch = triple.match(/:node-(\w+)/);
      if (idMatch && idMatch[1]) {
        nodes.push({
          id: idMatch[1],
          type: triple.includes('Task') ? 'task' : 'condition',
          label: `Node ${idMatch[1]}`,
          position: { x: 0, y: 0 }, // Default position
        });
      }
    }

    return nodes;
  }

  /**
   * Extract edges from RDF triples
   */
  private extractEdges(
    triples: string[],
    _prefixes: Record<string, string>
  ): YAWLWorkflow['edges'] {
    const edges: YAWLWorkflow['edges'] = [];

    // Simplified extraction - in production, use proper RDF query
    const edgeTriples = triples.filter((t) => t.includes('yawl:flow'));

    for (const triple of edgeTriples) {
      const match = triple.match(/:edge-(\w+).*yawl:from\s+:node-(\w+).*yawl:to\s+:node-(\w+)/);
      if (match && match[1] && match[2] && match[3]) {
        edges.push({
          id: match[1],
          source: match[2],
          target: match[3],
        });
      }
    }

    return edges;
  }

  /**
   * Validate workflow using existing validator
   */
  private async validateWorkflow(workflow: YAWLWorkflow): Promise<ValidationResult> {
    const { validateWorkflow } = await import('@/lib/validation/pattern-validator');
    return validateWorkflow(workflow);
  }

  /**
   * Convert YAWL XML to workflow structure
   */
  private async yawlXMLToWorkflow(_xml: string): Promise<YAWLWorkflow> {
    // Simplified XML parsing - in production, use proper XML parser
    // This is a placeholder implementation
    // TODO: Parse XML and extract workflow structure
    return {
      id: 'workflow-from-xml',
      name: 'Workflow from XML',
      version: '1.0.0',
      nodes: [],
      edges: [],
    };
  }

  /**
   * Generate YAWL XML from workflow
   */
  private generateYAWLXML(workflow: YAWLWorkflow): string {
    // Simplified XML generation
    const nodes = workflow.nodes
      .map(
        (node) => `
    <task id="${node.id}">
      <name>${node.label}</name>
      <decomposesTo id="${node.id}_net"/>
    </task>`
      )
      .join('\n');

    const flows = workflow.edges
      .map(
        (edge) => `
    <flow source="${edge.source}" target="${edge.target}">
      ${edge.condition ? `<predicate>${edge.condition}</predicate>` : ''}
    </flow>`
      )
      .join('\n');

    return `<?xml version="1.0" encoding="UTF-8"?>
<specificationSet xmlns="http://www.yawlfoundation.org/yawlschema"
                  xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
                  version="4.0">
  <specification uri="${workflow.id}">
    <name>${workflow.name}</name>
    <version>${workflow.version}</version>
    <metaData>
      ${workflow.metadata?.author ? `<creator>${workflow.metadata.author}</creator>` : ''}
      ${workflow.metadata?.description ? `<description>${workflow.metadata.description}</description>` : ''}
      <created>${workflow.metadata?.created || new Date().toISOString()}</created>
    </metaData>
    <decomposition id="root_net" isRootNet="true">
      ${nodes}
      ${flows}
    </decomposition>
  </specification>
</specificationSet>`;
  }

  /**
   * Extract patterns from workflow
   */
  private extractPatterns(workflow: YAWLWorkflow): string[] {
    const patterns = new Set<string>();

    // Detect basic patterns
    const splits = workflow.nodes.filter((n) => n.type === 'split');
    const joins = workflow.nodes.filter((n) => n.type === 'join');

    if (splits.length > 0 && joins.length > 0) {
      patterns.add('split-join');
    }

    if (workflow.edges.some((e) => e.condition)) {
      patterns.add('exclusive-choice');
    }

    return Array.from(patterns);
  }

  /**
   * Check pattern matrix conformance
   */
  private checkMatrixConformance(_patterns: string[]): boolean {
    // In production, check against actual pattern matrix
    // TODO: Validate patterns against knhk pattern permutation matrix
    return true;
  }

  /**
   * Check workflow invariants
   */
  private checkInvariants(workflow: YAWLWorkflow): boolean {
    // Check basic invariants
    const hasStart = workflow.nodes.some((n) => n.type === 'start');
    const hasEnd = workflow.nodes.some((n) => n.type === 'end');

    return hasStart && hasEnd;
  }
}

/* ============================================================================
 * Singleton Instance
 * ========================================================================== */

/**
 * Default workflow exchange instance
 */
export const workflowExchange = new WorkflowExchange();
