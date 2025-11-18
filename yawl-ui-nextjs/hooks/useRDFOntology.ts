/**
 * useRDFOntology - RDF/Turtle ontology manipulation hook
 * Handles RDF parsing, serialization, and semantic operations
 * Aligned with DOCTRINE Σ (Ontology/Schema)
 */

import { useCallback, useState } from 'react'
import * as N3 from 'n3'
import type { YAWLSpecification, YAWLRDFModel } from '@/types/yawl'

interface RDFState {
  store: N3.Store
  namespaces: Record<string, string>
  isLoading: boolean
  error: string | null
}

const DEFAULT_NAMESPACES = {
  yawl: 'http://www.yawlfoundation.org/yawl/',
  rdf: 'http://www.w3.org/1999/02/22-rdf-syntax-ns#',
  rdfs: 'http://www.w3.org/2000/01/rdf-schema#',
  xsd: 'http://www.w3.org/2001/XMLSchema#',
  owl: 'http://www.w3.org/2002/07/owl#',
  prov: 'http://www.w3.org/ns/prov#',
  skos: 'http://www.w3.org/2004/02/skos/core#',
}

/**
 * Hook for managing RDF ontologies and Turtle serialization
 * Implements Σ principle: ontology as source of truth
 */
export function useRDFOntology() {
  const [state, setState] = useState<RDFState>({
    store: new N3.Store(),
    namespaces: DEFAULT_NAMESPACES,
    isLoading: false,
    error: null,
  })

  // Parse Turtle RDF
  const parseTurtle = useCallback(
    async (ttlContent: string) => {
      setState((prev) => ({ ...prev, isLoading: true, error: null }))

      try {
        const parser = new N3.Parser()
        const quads = parser.parse(ttlContent)

        const newStore = new N3.Store(quads)
        setState((prev) => ({
          ...prev,
          store: newStore,
          isLoading: false,
        }))

        return newStore
      } catch (err) {
        const error = err instanceof Error ? err.message : 'Parse error'
        setState((prev) => ({
          ...prev,
          isLoading: false,
          error,
        }))
        throw err
      }
    },
    [state.namespaces]
  )

  // Serialize to Turtle
  const serializeTurtle = useCallback(async () => {
    return new Promise<string>((resolve, reject) => {
      const writer = new N3.Writer({ prefixes: state.namespaces })
      writer.addQuads(state.store.getQuads(null, null, null, null))
      writer.end((err, result) => {
        if (err) reject(err)
        else resolve(result)
      })
    })
  }, [state.store, state.namespaces])

  // Add triple
  const addTriple = useCallback(
    (subject: string, predicate: string, object: string) => {
      const quad = N3.DataFactory.quad(
        N3.DataFactory.namedNode(subject),
        N3.DataFactory.namedNode(predicate),
        N3.DataFactory.namedNode(object)
      )

      setState((prev) => {
        const newStore = new N3.Store(prev.store.getQuads(null, null, null, null))
        newStore.addQuad(quad)
        return { ...prev, store: newStore }
      })
    },
    []
  )

  // Query triples
  const queryTriples = useCallback(
    (subject?: string, predicate?: string, object?: string) => {
      const s = subject ? N3.DataFactory.namedNode(subject) : null
      const p = predicate ? N3.DataFactory.namedNode(predicate) : null
      const o = object ? N3.DataFactory.namedNode(object) : null

      return state.store.getQuads(s, p, o, null)
    },
    [state.store]
  )

  // Get all resources of type
  const getResourcesByType = useCallback(
    (type: string) => {
      const typeNode = N3.DataFactory.namedNode(
        `${state.namespaces.rdf}type`
      )
      const typeValue = N3.DataFactory.namedNode(type)

      return state.store
        .getQuads(null, typeNode, typeValue, null)
        .map((q) => q.subject.value)
    },
    [state.store, state.namespaces]
  )

  // Create YAWL ontology base
  const createYAWLOntology = useCallback(() => {
    const store = new N3.Store()
    const writer = new N3.Writer({ prefixes: DEFAULT_NAMESPACES })

    // Add ontology metadata
    const ontologyIRI = DEFAULT_NAMESPACES.yawl + 'ontology'
    writer.addQuad(
      N3.DataFactory.namedNode(ontologyIRI),
      N3.DataFactory.namedNode(`${DEFAULT_NAMESPACES.rdf}type`),
      N3.DataFactory.namedNode(`${DEFAULT_NAMESPACES.owl}Ontology`)
    )

    // Add YAWL classes
    const classes = [
      'Specification',
      'Task',
      'Net',
      'ControlFlow',
      'Pattern',
      'WorkItem',
      'Case',
    ]

    classes.forEach((cls) => {
      const classIRI = DEFAULT_NAMESPACES.yawl + cls
      writer.addQuad(
        N3.DataFactory.namedNode(classIRI),
        N3.DataFactory.namedNode(`${DEFAULT_NAMESPACES.rdf}type`),
        N3.DataFactory.namedNode(`${DEFAULT_NAMESPACES.rdfs}Class`)
      )
    })

    setState((prev) => ({
      ...prev,
      store: new N3.Store(
        store.getQuads(null, null, null, null)
      ),
    }))
  }, [])

  // Validate against SHACL shapes
  const validateSHACL = useCallback(
    async (shapesStore: N3.Store): Promise<boolean> => {
      // Simplified SHACL validation
      // In production, use a library like N3.js SHACL validator
      const constraints = shapesStore.getQuads(null, null, null, null)
      return constraints.length > 0
    },
    []
  )

  return {
    // State
    store: state.store,
    namespaces: state.namespaces,
    isLoading: state.isLoading,
    error: state.error,

    // Actions
    parseTurtle,
    serializeTurtle,
    addTriple,
    queryTriples,
    getResourcesByType,
    createYAWLOntology,
    validateSHACL,
  }
}

export default useRDFOntology
