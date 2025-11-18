/**
 * Ontology Builder - YAWL RDF Ontology Construction
 * Builds and maintains YAWL semantic ontologies
 * Aligned with DOCTRINE Σ (Ontology/Schema)
 */

import * as N3 from 'n3'

export interface OntologyDefinition {
  namespace: string
  prefix: string
  classes: OntologyClass[]
  properties: OntologyProperty[]
  individuals?: OntologyIndividual[]
}

export interface OntologyClass {
  name: string
  label?: string
  description?: string
  superclass?: string
  restrictions?: PropertyRestriction[]
}

export interface OntologyProperty {
  name: string
  label?: string
  description?: string
  domain?: string
  range?: string
  subPropertyOf?: string
  isInverseFunctional?: boolean
  isFunctional?: boolean
}

export interface PropertyRestriction {
  type: 'cardinality' | 'hasValue' | 'someValuesFrom' | 'allValuesFrom'
  property: string
  value?: unknown
  cardinality?: number
}

export interface OntologyIndividual {
  name: string
  type: string
  properties: Record<string, unknown>
}

/**
 * Builder for constructing YAWL ontologies
 */
export class OntologyBuilder {
  private namespaces: Map<string, string> = new Map()
  private classes: Map<string, OntologyClass> = new Map()
  private properties: Map<string, OntologyProperty> = new Map()
  private individuals: Map<string, OntologyIndividual> = new Map()

  constructor(private baseNamespace = 'http://www.yawlfoundation.org/yawl/') {
    this.initializeDefaultNamespaces()
  }

  private initializeDefaultNamespaces(): void {
    this.namespaces.set('yawl', this.baseNamespace)
    this.namespaces.set('rdf', 'http://www.w3.org/1999/02/22-rdf-syntax-ns#')
    this.namespaces.set('rdfs', 'http://www.w3.org/2000/01/rdf-schema#')
    this.namespaces.set('owl', 'http://www.w3.org/2002/07/owl#')
    this.namespaces.set('xsd', 'http://www.w3.org/2001/XMLSchema#')
  }

  /**
   * Add namespace
   */
  addNamespace(prefix: string, namespace: string): this {
    this.namespaces.set(prefix, namespace)
    return this
  }

  /**
   * Add class definition
   */
  addClass(classDef: OntologyClass): this {
    this.classes.set(classDef.name, classDef)
    return this
  }

  /**
   * Add property definition
   */
  addProperty(propDef: OntologyProperty): this {
    this.properties.set(propDef.name, propDef)
    return this
  }

  /**
   * Add individual instance
   */
  addIndividual(individual: OntologyIndividual): this {
    this.individuals.set(individual.name, individual)
    return this
  }

  /**
   * Build N3 RDF representation
   */
  build(): string {
    const writer = new N3.Writer({
      prefixes: Object.fromEntries(this.namespaces),
    })

    // Add classes
    this.classes.forEach((classDef) => {
      const classIRI = this.expandIRI(`yawl:${classDef.name}`)
      writer.addQuad(
        N3.DataFactory.namedNode(classIRI),
        N3.DataFactory.namedNode(this.expandIRI('rdf:type')),
        N3.DataFactory.namedNode(this.expandIRI('rdfs:Class'))
      )

      if (classDef.label) {
        writer.addQuad(
          N3.DataFactory.namedNode(classIRI),
          N3.DataFactory.namedNode(this.expandIRI('rdfs:label')),
          N3.DataFactory.literal(classDef.label)
        )
      }

      if (classDef.description) {
        writer.addQuad(
          N3.DataFactory.namedNode(classIRI),
          N3.DataFactory.namedNode(
            this.expandIRI('rdfs:comment')
          ),
          N3.DataFactory.literal(classDef.description)
        )
      }

      if (classDef.superclass) {
        writer.addQuad(
          N3.DataFactory.namedNode(classIRI),
          N3.DataFactory.namedNode(this.expandIRI('rdfs:subClassOf')),
          N3.DataFactory.namedNode(this.expandIRI(`yawl:${classDef.superclass}`))
        )
      }
    })

    // Add properties
    this.properties.forEach((propDef) => {
      const propIRI = this.expandIRI(`yawl:${propDef.name}`)
      writer.addQuad(
        N3.DataFactory.namedNode(propIRI),
        N3.DataFactory.namedNode(this.expandIRI('rdf:type')),
        N3.DataFactory.namedNode(this.expandIRI('rdf:Property'))
      )

      if (propDef.label) {
        writer.addQuad(
          N3.DataFactory.namedNode(propIRI),
          N3.DataFactory.namedNode(this.expandIRI('rdfs:label')),
          N3.DataFactory.literal(propDef.label)
        )
      }

      if (propDef.domain) {
        writer.addQuad(
          N3.DataFactory.namedNode(propIRI),
          N3.DataFactory.namedNode(this.expandIRI('rdfs:domain')),
          N3.DataFactory.namedNode(this.expandIRI(`yawl:${propDef.domain}`))
        )
      }

      if (propDef.range) {
        writer.addQuad(
          N3.DataFactory.namedNode(propIRI),
          N3.DataFactory.namedNode(this.expandIRI('rdfs:range')),
          N3.DataFactory.namedNode(this.expandIRI(`yawl:${propDef.range}`))
        )
      }
    })

    return `${this.getNamespacePreamble()}\n\n${this.serializeWriter(writer)}`
  }

  /**
   * Get namespace preamble
   */
  private getNamespacePreamble(): string {
    const lines: string[] = []
    this.namespaces.forEach((ns, prefix) => {
      lines.push(`@prefix ${prefix}: <${ns}> .`)
    })
    return lines.join('\n')
  }

  /**
   * Serialize writer (wrapper for promise-based API)
   */
  private serializeWriter(writer: N3.Writer): string {
    return new Promise<string>((resolve) => {
      writer.end((err, result) => {
        if (err) throw err
        resolve(result)
      })
    }) as any // Hack for sync context
  }

  /**
   * Expand abbreviated IRI
   */
  private expandIRI(abbreviated: string): string {
    const [prefix, local] = abbreviated.split(':')
    const namespace = this.namespaces.get(prefix)
    if (!namespace) throw new Error(`Unknown prefix: ${prefix}`)
    return `${namespace}${local}`
  }

  /**
   * Get built ontology as object
   */
  getOntologyDefinition(): OntologyDefinition {
    return {
      namespace: this.baseNamespace,
      prefix: 'yawl',
      classes: Array.from(this.classes.values()),
      properties: Array.from(this.properties.values()),
      individuals: Array.from(this.individuals.values()),
    }
  }
}

/**
 * Create default YAWL ontology
 */
export function createDefaultYAWLOntology(): OntologyBuilder {
  const builder = new OntologyBuilder()

  // Add core classes
  builder
    .addClass({
      name: 'Specification',
      label: 'YAWL Specification',
      description: 'A workflow specification in YAWL',
    })
    .addClass({
      name: 'Task',
      label: 'YAWL Task',
      description: 'An atomic unit of work in YAWL',
      superclass: 'Element',
    })
    .addClass({
      name: 'Net',
      label: 'YAWL Net',
      description: 'A workflow network/process',
    })
    .addClass({
      name: 'ControlFlow',
      label: 'Control Flow',
      description: 'Flow between tasks',
    })
    .addClass({
      name: 'Pattern',
      label: 'Control Flow Pattern',
      description: 'YAWL control flow pattern',
    })

  // Add properties
  builder
    .addProperty({
      name: 'hasTask',
      label: 'has task',
      domain: 'Specification',
      range: 'Task',
    })
    .addProperty({
      name: 'hasNet',
      label: 'has net',
      domain: 'Specification',
      range: 'Net',
    })
    .addProperty({
      name: 'hasControlFlow',
      label: 'has control flow',
      domain: 'Net',
      range: 'ControlFlow',
    })
    .addProperty({
      name: 'pattern',
      label: 'pattern type',
      domain: 'ControlFlow',
      range: 'Pattern',
    })

  return builder
}

export default OntologyBuilder
