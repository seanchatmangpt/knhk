/**
 * DOCTRINE ALIGNMENT: O (Ontology-First)
 * RDF/Turtle parser using N3 library
 * Provides semantic ontology integration for YAWL workflows
 */

import { Parser, Store, DataFactory, Writer } from 'n3';
import type { OntologyDefinition, RDFTriple } from '@/lib/types';

const { namedNode, literal } = DataFactory;

/**
 * Parse Turtle/RDF string into structured ontology definition
 *
 * PERFORMANCE: This operation should complete in ≤8 ticks (Chatman Constant)
 * for typical YAWL ontology files (<10KB)
 */
export async function parseTurtle(turtleString: string): Promise<OntologyDefinition> {
  const parser = new Parser({ format: 'text/turtle' });
  const store = new Store();

  return new Promise((resolve, reject) => {
    parser.parse(turtleString, (error, quad, prefixes) => {
      if (error) {
        reject(new Error(`Turtle parsing failed: ${error.message}`));
        return;
      }

      if (quad) {
        store.addQuad(quad);
      } else {
        // Parsing complete
        const triples: RDFTriple[] = store.getQuads(null, null, null, null).map((q) => ({
          subject: q.subject.value,
          predicate: q.predicate.value,
          object: q.object.value,
        }));

        const classes = extractClasses(store);
        const properties = extractProperties(store);

        // Convert Prefixes object to Record<string, string>
        const prefixesRecord: Record<string, string> = {};
        if (prefixes) {
          for (const [key, value] of Object.entries(prefixes)) {
            prefixesRecord[key] = value.value;
          }
        }

        resolve({
          uri: extractBaseURI(prefixesRecord),
          prefixes: prefixesRecord,
          classes,
          properties,
          triples,
        });
      }
    });
  });
}

/**
 * Serialize ontology definition back to Turtle format
 */
export function serializeToTurtle(ontology: OntologyDefinition): Promise<string> {
  return new Promise((resolve, reject) => {
    const writer = new Writer({
      prefixes: ontology.prefixes,
      format: 'text/turtle',
    });

    ontology.triples.forEach((triple: RDFTriple) => {
      writer.addQuad(
        namedNode(triple.subject),
        namedNode(triple.predicate),
        triple.object.startsWith('http') ? namedNode(triple.object) : literal(triple.object)
      );
    });

    writer.end((error, result) => {
      if (error) {
        reject(new Error(`Turtle serialization failed: ${error.message}`));
      } else {
        resolve(result);
      }
    });
  });
}

/**
 * Extract RDF classes from store
 */
function extractClasses(store: Store): string[] {
  const classQuads = store.getQuads(
    null,
    namedNode('http://www.w3.org/1999/02/22-rdf-syntax-ns#type'),
    namedNode('http://www.w3.org/2002/07/owl#Class'),
    null
  );

  return classQuads.map((quad) => quad.subject.value);
}

/**
 * Extract RDF properties from store
 */
function extractProperties(store: Store): string[] {
  const propertyQuads = store.getQuads(
    null,
    namedNode('http://www.w3.org/1999/02/22-rdf-syntax-ns#type'),
    namedNode('http://www.w3.org/2002/07/owl#ObjectProperty'),
    null
  );

  return propertyQuads.map((quad) => quad.subject.value);
}

/**
 * Extract base URI from prefixes
 */
function extractBaseURI(prefixes: Record<string, string>): string {
  return prefixes[''] || prefixes['base'] || 'http://example.org/';
}

/**
 * Validate YAWL ontology structure
 *
 * COVENANT 2: Invariants Are Law
 * This validation ensures ontology conforms to YAWL specification
 */
export function validateYAWLOntology(ontology: OntologyDefinition): {
  valid: boolean;
  errors: string[];
} {
  const errors: string[] = [];

  // Check for required YAWL classes
  const requiredClasses = [
    'yawl:Workflow',
    'yawl:Task',
    'yawl:Condition',
    'yawl:Pattern',
  ];

  requiredClasses.forEach((requiredClass) => {
    const found = ontology.classes.some((cls: string) => cls.includes(requiredClass.split(':')[1] || ''));
    if (!found) {
      errors.push(`Missing required class: ${requiredClass}`);
    }
  });

  return {
    valid: errors.length === 0,
    errors,
  };
}
