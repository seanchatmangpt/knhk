# unrdf Knowledge Hook Engine Integration

## Overview

This document describes the integration of the unrdf knowledge hook engine into the KNHK project. The unrdf knowledge hook engine provides autonomic triggers for policy-driven automation, which complements KNHK's reflex system architecture.

## Repository Location

The unrdf repository has been cloned into `./vendors/unrdf/`:
- **Source**: https://github.com/seanchatmangpt/unrdf
- **Version**: 3.0.4
- **Type**: JavaScript/TypeScript (ESM modules)

## Key Components

### Core Knowledge Engine (`src/knowledge-engine/`)

The knowledge hook engine is located in `vendors/unrdf/src/knowledge-engine/` with the following key modules:

#### 1. Hook Definition (`define-hook.mjs`)
- **Purpose**: Defines the contract for knowledge hooks
- **Key Features**:
  - Content-addressed file references (external SPARQL/SHACL files)
  - Reflex arc lifecycle: `before`, `run`, `after` functions
  - Declarative configuration (determinism, receipting strategies)
  - Comprehensive Zod validation

#### 2. Hook Manager (`knowledge-hook-manager.mjs`)
- **Purpose**: Manages hook registration, evaluation, and execution
- **Key Features**:
  - Extends `TransactionManager` for transaction integration
  - Hook registration and lifecycle management
  - Policy pack integration
  - Security validation and sandboxing

#### 3. Hook Executor (`hook-executor.mjs`)
- **Purpose**: Executes registered hooks based on conditions
- **Key Features**:
  - Batching support (`hook-executor-batching.mjs`)
  - Condition evaluation (`condition-evaluator.mjs`)
  - Performance optimization

#### 4. Dark Matter Core (`dark-matter-core.mjs` / `knowledge-substrate-core.mjs`)
- **Purpose**: Implements 80/20 framework for knowledge processing
- **Key Features**:
  - Focuses on critical 20% that delivers 80% of value
  - Performance targets (p50 ≤200µs, p99 ≤2ms)
  - Transaction management
  - Observability integration

#### 5. Policy Pack Manager (`policy-pack.mjs`)
- **Purpose**: Manages versioned policy packs for governance
- **Key Features**:
  - Versioned governance units
  - Policy pack loading and validation
  - Integration with hook execution

#### 6. Lockchain Writer (`lockchain-writer.mjs`)
- **Purpose**: Cryptographic provenance and audit trails
- **Key Features**:
  - SHA3-256 Merkle verification
  - Cryptographic audit trails
  - Receipt generation

#### 7. Query System (`query.mjs`, `query-optimizer.mjs`)
- **Purpose**: SPARQL query execution via Comunica
- **Key Features**:
  - Full SPARQL 1.1 support
  - Query caching and optimization
  - Delta-aware optimization

## API Surface

### Main Entry Point (`src/index.mjs`)

```javascript
// Core knowledge engine exports
export * from "./knowledge-engine/index.mjs";

// Key exports:
export { defineHook, registerHook, evaluateHook } from "./knowledge-engine/";
export { createDarkMatterCore, KnowledgeSubstrateCore } from "./knowledge-engine/";
export { PolicyPackManager } from "./knowledge-engine/";
export { LockchainWriter } from "./knowledge-engine/";
```

### Hook Definition API

```javascript
const hook = defineHook({
  meta: {
    name: 'hook-name',
    description: 'Hook description'
  },
  when: {
    kind: 'sparql-ask',
    ref: {
      uri: 'file://path/to/query.rq',
      sha256: '...',
      mediaType: 'application/sparql-query'
    }
  },
  run: async (event) => {
    // Hook execution logic
  }
});
```

## Relationship to KNHK Architecture

### KNHK Reflex System vs unrdf Knowledge Hooks

| Aspect | KNHK Reflex | unrdf Knowledge Hook |
|--------|-------------|---------------------|
| **Execution Layer** | Hot path (C, ≤8 ticks) | Cold path (JavaScript/Node.js) |
| **Purpose** | Sub-2ns deterministic reflexes | Policy-driven autonomic triggers |
| **Trigger Type** | ASK, COUNT, COMPARE, CONSTRUCT8 | SPARQL-ASK, SHACL, Delta, Threshold |
| **Lifecycle** | Single-shot evaluation | Before/Run/After lifecycle |
| **Integration** | FFI from Rust | Node.js runtime |

### Integration Points

#### 1. Cold Path Integration (Erlang/Node.js Bridge)

KNHK's cold path architecture can integrate unrdf hooks for complex policy evaluation:

```
┌─────────────────┐
│  KNHK Warm Path │  (Rust)
│  (≤500ms)       │
└────────┬────────┘
         │ Complex queries / Policy evaluation
         ▼
┌─────────────────┐
│  KNHK Cold Path │  (Erlang)
│  (>500ms)       │
└────────┬────────┘
         │ Bridge to Node.js
         ▼
┌─────────────────┐
│  unrdf Hook     │  (Node.js)
│  Engine         │
└─────────────────┘
```

**Integration Strategy**:
- Use Erlang/Node.js port or NIF bridge
- Execute unrdf hooks for complex SPARQL queries
- Return results to KNHK cold path for further processing

#### 2. Policy Pack Integration

unrdf's policy pack system aligns with KNHK's governance model:

- **KNHK**: Guards and invariants defined as RDF triples
- **unrdf**: Policy packs as versioned governance units

**Integration Strategy**:
- Map KNHK guards to unrdf policy packs
- Use unrdf's policy pack manager for versioned governance
- Synchronize policy definitions between systems

#### 3. Lockchain Integration

Both systems use cryptographic provenance:

- **KNHK**: Receipts with `hash(A) = hash(μ(O))`
- **unrdf**: Lockchain with SHA3-256 Merkle verification

**Integration Strategy**:
- Align hash algorithms (SHA-256 vs SHA3-256)
- Cross-reference receipts and lockchain entries
- Unified audit trail

#### 4. Query Optimization

Both systems optimize for 80/20 critical paths:

- **KNHK**: Hot path ≤8 ticks, warm path ≤500ms
- **unrdf**: Dark Matter 80/20 framework, p50 ≤200µs, p99 ≤2ms

**Integration Strategy**:
- Route simple queries to KNHK hot path
- Route complex queries to unrdf cold path
- Cache results for repeated queries

## Integration Architecture

### Option 1: Node.js Sidecar (Recommended)

Run unrdf as a sidecar service:

```
┌──────────────┐
│  KNHK Core   │
│  (Rust/C)    │
└──────┬───────┘
       │ gRPC/HTTP
       ▼
┌──────────────┐
│ unrdf Sidecar│
│  (Node.js)   │
└──────────────┘
```

**Benefits**:
- Isolated runtime (no FFI complexity)
- Independent scaling
- Clear service boundaries

### Option 2: Erlang Port Bridge

Bridge from Erlang cold path to Node.js:

```erlang
% Erlang cold path
unrdf_hook:execute(HookName, Graph, Delta) ->
    Port = open_port({spawn, "node unrdf-bridge.mjs"}, [...]),
    port_command(Port, encode_request(HookName, Graph, Delta)),
    receive
        {Port, {data, Result}} ->
            decode_response(Result)
    end.
```

**Benefits**:
- Direct integration with Erlang cold path
- No network overhead
- Process isolation

### Option 3: Extract Core Logic

Extract hook evaluation logic for Rust/Erlang implementation:

**Benefits**:
- No Node.js dependency
- Better performance
- Full control over execution

**Challenges**:
- Requires porting JavaScript to Rust/Erlang
- Loss of existing ecosystem (N3.js, Comunica)

## Dependencies

### Runtime Dependencies

unrdf requires Node.js ≥18.0.0 with the following key dependencies:

- **@comunica/query-sparql**: SPARQL query engine
- **n3**: RDF store and parsing
- **rdf-validate-shacl**: SHACL validation
- **@opentelemetry/api**: Observability
- **zod**: Runtime validation
- **vm2**: Sandboxing for hook execution

### Build Dependencies

- **vitest**: Testing framework
- **eslint**: Linting
- **prettier**: Code formatting

## Usage Examples

### Example 1: Basic Hook Definition

```javascript
import { defineHook, registerHook } from 'unrdf';

const hook = defineHook({
  meta: {
    name: 'data-quality-gate',
    description: 'Ensures all persons have names'
  },
  when: {
    kind: 'sparql-ask',
    ref: {
      uri: 'file://hooks/data-quality.rq',
      sha256: 'abc123...',
      mediaType: 'application/sparql-query'
    }
  },
  run: async (event) => {
    if (event.result === true) {
      throw new Error('All persons must have names');
    }
  }
});

await registerHook(hook);
```

### Example 2: Dark Matter Core Usage

```javascript
import { createDarkMatterCore } from 'unrdf';

const system = await createDarkMatterCore({
  enableKnowledgeHookManager: true,
  enableLockchainWriter: true,
  enableObservability: true
});

await system.executeTransaction({
  additions: [...triples...],
  removals: [],
  actor: 'system'
});
```

### Example 3: Policy Pack Integration

```javascript
import { PolicyPackManager } from 'unrdf';

const policyManager = new PolicyPackManager();
await policyManager.loadPolicyPack('compliance-v1');

const hooks = policyManager.getHooks();
for (const hook of hooks) {
  await registerHook(hook);
}
```

## Next Steps

1. **Evaluate Integration Strategy**: Choose between sidecar, port bridge, or extraction
2. **Prototype Bridge**: Implement Erlang/Node.js or Rust/Node.js bridge
3. **Map APIs**: Align KNHK reflex API with unrdf hook API
4. **Test Integration**: Validate hook execution from KNHK cold path
5. **Performance Testing**: Measure latency impact of hook execution
6. **Documentation**: Update KNHK documentation with unrdf integration

## References

- **unrdf Repository**: `./vendors/unrdf/`
- **unrdf Documentation**: `./vendors/unrdf/README.md`
- **Knowledge Engine API**: `./vendors/unrdf/src/knowledge-engine/index.mjs`
- **KNHK Reflex System**: `rust/knhk-cli/src/commands/reflex.rs`
- **KNHK Cold Path**: `erlang/knhk_rc/src/knhk_rc.erl`

