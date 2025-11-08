# Fortune 500 CLI Implementation Plan

**Version**: 1.0.0  
**Status**: Planning  
**Target**: clap-noun-verb 3.4.0  
**Storage**: Oxigraph (RDF store)

## Overview

This plan implements a fully functional Fortune 500-level CLI that addresses all identified gaps:
- State management for O, Σ, Q using Oxigraph
- Connector registry that instantiates connectors
- Hook registry integrated with system
- Receipt store with Oxigraph
- Lockchain service for provenance
- Schema validation using Oxigraph SPARQL
- Invariant enforcement using Oxigraph SPARQL
- Dependency checking
- Error handling with rollback
- Command integration

## Current State

- CLI uses clap-noun-verb 3.3.0 (needs upgrade to 3.4.0)
- Commands are isolated, don't share state
- No state management for O, Σ, Q
- Connectors stored but not instantiated
- Hooks stored but not registered
- Receipts generated but not stored
- No schema/invariant loading
- No dependency checking
- No error recovery

## Implementation Phases

### Phase 1: State Management Foundation

**Goal**: Create persistent state management for O, Σ, Q using Oxigraph

**Tasks**:
1. Add oxigraph dependency to Cargo.toml
2. Create `StateManager` component that loads/saves O, Σ, Q
3. Implement `OntologyLoader`, `OntologySaver`, `OntologyMerger` using Oxigraph
4. Implement `SchemaLoader`, `InvariantLoader` using Oxigraph
5. Add state cache for performance
6. Use Oxigraph for persistent RDF storage (O, Σ, Q as RDF graphs)

**Files to Create/Modify**:
- `knhk-cli/Cargo.toml` - Add oxigraph dependency
- `knhk-cli/src/state/mod.rs` - State manager module
- `knhk-cli/src/state/ontology.rs` - Ontology management with Oxigraph
- `knhk-cli/src/state/schema.rs` - Schema management with Oxigraph
- `knhk-cli/src/state/invariant.rs` - Invariant management with Oxigraph
- `knhk-cli/src/state/store.rs` - Oxigraph store wrapper

**Success Criteria**:
- O, Σ, Q can be loaded from Oxigraph
- O can be saved to Oxigraph
- Δ can be merged into O in Oxigraph
- State persists between commands

### Phase 2: Connector Registry

**Goal**: Actually instantiate connectors, not just store names

**Tasks**:
1. Create `ConnectorRegistry` that manages connector instances
2. Implement `ConnectorFactory` to create connector instances from storage
3. Implement `ConnectorPool` for lifecycle management
4. Convert connector storage to actual connector objects
5. Integrate with ETL pipeline

**Files to Create/Modify**:
- `knhk-cli/src/connector/mod.rs` - Connector registry module
- `knhk-cli/src/connector/registry.rs` - Connector registry
- `knhk-cli/src/connector/factory.rs` - Connector factory
- `knhk-cli/src/commands/connect.rs` - Update to use registry
- `knhk-cli/src/commands/pipeline.rs` - Use connector registry

**Success Criteria**:
- Connectors are instantiated from storage
- Connectors can be used by ETL pipeline
- Connector lifecycle is managed
- Connectors persist between commands

### Phase 3: Hook Registry Integration

**Goal**: Register hooks with system, not just file storage

**Tasks**:
1. Integrate with `knhk-etl::HookRegistry`
2. Load hooks from storage into registry
3. Execute hooks through registry
4. Persist hook execution results
5. Store hooks in Oxigraph (as RDF metadata)

**Files to Create/Modify**:
- `knhk-cli/src/hook/registry.rs` - Hook registry integration
- `knhk-cli/src/hook/store.rs` - Hook storage with Oxigraph
- `knhk-cli/src/commands/hook.rs` - Update to use registry

**Success Criteria**:
- Hooks are registered with system
- Hooks can be executed through registry
- Hook execution results are persisted
- Hooks work with ETL pipeline

### Phase 4: Receipt Store

**Goal**: Persist receipts and enable retrieval using Oxigraph

**Tasks**:
1. Store receipts in Oxigraph (as RDF metadata)
2. Implement receipt indexing (SPARQL queries)
3. Implement receipt linking (Merkle tree)
4. Implement receipt retrieval (SPARQL queries)
5. Link receipts to operations (RDF triples)

**Files to Create/Modify**:
- `knhk-cli/src/receipt/store.rs` - Receipt storage with Oxigraph
- `knhk-cli/src/receipt/indexer.rs` - Receipt indexing (SPARQL)
- `knhk-cli/src/receipt/linker.rs` - Receipt linking
- `knhk-cli/src/commands/receipt.rs` - Update to use store

**Success Criteria**:
- Receipts are stored in Oxigraph
- Receipts can be retrieved by ID
- Receipts are linked in Merkle tree
- Receipts are indexed for fast lookup

### Phase 5: Lockchain Service

**Goal**: Generate and verify Merkle-linked receipts

**Tasks**:
1. Implement Merkle tree builder
2. Implement hash generator (SHA-256)
3. Implement provenance tracker
4. Link receipts in Merkle tree
5. Verify receipt chains
6. Store Merkle tree in Oxigraph

**Files to Create/Modify**:
- `knhk-cli/src/lockchain/mod.rs` - Lockchain module
- `knhk-cli/src/lockchain/merkle.rs` - Merkle tree
- `knhk-cli/src/lockchain/hash.rs` - Hash generation
- `knhk-cli/src/lockchain/provenance.rs` - Provenance tracking
- `knhk-cli/src/lockchain/store.rs` - Merkle tree storage in Oxigraph

**Success Criteria**:
- Merkle trees are generated
- Receipts are linked in Merkle tree
- Receipt chains can be verified
- Provenance is tracked

### Phase 6: Schema Validator

**Goal**: Actually validate O ⊨ Σ using loaded schemas and Oxigraph SPARQL

**Tasks**:
1. Load Σ from Oxigraph
2. Parse schema (Turtle/RDF)
3. Validate O against Σ using SPARQL queries
4. Integrate with ETL pipeline
5. Provide validation errors

**Files to Create/Modify**:
- `knhk-cli/src/validation/schema.rs` - Schema validation
- `knhk-cli/src/validation/parser.rs` - Schema parsing
- `knhk-cli/src/validation/sparql.rs` - SPARQL query generation
- `knhk-cli/src/commands/admit.rs` - Use schema validator

**Success Criteria**:
- Σ is loaded from Oxigraph
- O is validated against Σ using SPARQL
- Validation errors are clear
- Validation integrates with ETL pipeline

### Phase 7: Invariant Enforcer

**Goal**: Enforce Q invariants on operations using Oxigraph SPARQL

**Tasks**:
1. Load Q from Oxigraph
2. Parse invariants
3. Enforce invariants on operations using SPARQL queries
4. Integrate with ETL pipeline
5. Provide enforcement errors

**Files to Create/Modify**:
- `knhk-cli/src/validation/invariant.rs` - Invariant enforcement
- `knhk-cli/src/validation/parser.rs` - Invariant parsing
- `knhk-cli/src/validation/sparql.rs` - SPARQL query generation
- `knhk-cli/src/commands/admit.rs` - Use invariant enforcer

**Success Criteria**:
- Q is loaded from Oxigraph
- Invariants are enforced using SPARQL
- Enforcement errors are clear
- Enforcement integrates with ETL pipeline

### Phase 8: Dependency Checker

**Goal**: Verify prerequisites before operations

**Tasks**:
1. Define command dependencies
2. Check prerequisites (boot init before admit)
3. Validate state consistency
4. Provide clear error messages
5. Integrate with all commands

**Files to Create/Modify**:
- `knhk-cli/src/dependency/mod.rs` - Dependency checking
- `knhk-cli/src/dependency/checker.rs` - Dependency checker
- `knhk-cli/src/dependency/graph.rs` - Dependency graph
- `knhk-cli/src/commands/*.rs` - Add dependency checks

**Success Criteria**:
- Prerequisites are checked before operations
- Clear error messages for missing prerequisites
- State consistency is validated
- Commands fail gracefully with helpful errors

### Phase 9: Error Handling & Recovery

**Goal**: Handle errors gracefully with rollback

**Tasks**:
1. Implement error recovery
2. Implement rollback mechanisms
3. Maintain state consistency on failure
4. Provide clear error messages
5. Log errors for debugging

**Files to Create/Modify**:
- `knhk-cli/src/error/recovery.rs` - Error recovery
- `knhk-cli/src/error/rollback.rs` - Rollback mechanisms
- `knhk-cli/src/error/transaction.rs` - Transaction management
- `knhk-cli/src/commands/*.rs` - Add error handling

**Success Criteria**:
- Errors are handled gracefully
- State is rolled back on failure
- State consistency is maintained
- Clear error messages are provided

### Phase 10: Command Integration

**Goal**: Make commands work together

**Tasks**:
1. Update `boot init` to actually initialize system (save Σ, Q to Oxigraph)
2. Update `admit delta` to load O, merge Δ, validate against Σ
3. Update `pipeline run` to use connector registry
4. Update `hook eval` to use real O from Oxigraph
5. Update `epoch run` to verify Erlang functions exist
6. Ensure state persists between commands

**Files to Modify**:
- `knhk-cli/src/commands/boot.rs` - Use state manager
- `knhk-cli/src/commands/admit.rs` - Load O, merge Δ, validate
- `knhk-cli/src/commands/pipeline.rs` - Use connector registry
- `knhk-cli/src/commands/hook.rs` - Use real O from Oxigraph
- `knhk-cli/src/commands/epoch.rs` - Verify Erlang

**Success Criteria**:
- Commands work together (boot → admit → hook eval)
- State persists between commands
- Commands use shared state
- Commands verify prerequisites

### Phase 11: clap-noun-verb 3.4.0 Integration

**Goal**: Ensure proper use of clap-noun-verb 3.4.0

**Tasks**:
1. Update Cargo.toml to clap-noun-verb 3.4.0
2. Verify noun-verb command structure
3. Ensure auto-discovery works
4. Test all commands
5. Verify #[verb] macros work correctly

**Files to Modify**:
- `knhk-cli/Cargo.toml` - Update clap-noun-verb version
- `knhk-cli/src/main.rs` - Verify integration
- `knhk-cli/src/commands/*.rs` - Verify #[verb] macros

**Success Criteria**:
- clap-noun-verb 3.4.0 is integrated
- All commands are auto-discovered
- Noun-verb structure works correctly
- Commands are properly registered

## Dependencies

### External Dependencies

- **oxigraph**: RDF store for O, Σ, Q, receipts
- **clap-noun-verb 3.4.0**: CLI framework
- **knhk-etl**: ETL pipeline integration
- **knhk-hot**: Hot path operations
- **knhk-warm**: Warm path operations
- **knhk-otel**: Observability

### Internal Dependencies

- Phase 1 (State Management) → All other phases
- Phase 2 (Connector Registry) → Phase 10 (Command Integration)
- Phase 3 (Hook Registry) → Phase 10 (Command Integration)
- Phase 4 (Receipt Store) → Phase 5 (Lockchain Service)
- Phase 5 (Lockchain Service) → Phase 10 (Command Integration)
- Phase 6 (Schema Validator) → Phase 10 (Command Integration)
- Phase 7 (Invariant Enforcer) → Phase 10 (Command Integration)
- Phase 8 (Dependency Checker) → Phase 10 (Command Integration)
- Phase 9 (Error Handling) → Phase 10 (Command Integration)
- Phase 11 (clap-noun-verb) → All phases

## Success Criteria

- ✅ All commands use state manager for O, Σ, Q (stored in Oxigraph)
- ✅ Connectors are actually instantiated and used
- ✅ Hooks are registered with system
- ✅ Receipts are stored and retrievable (in Oxigraph as RDF)
- ✅ Lockchain generates and links receipts
- ✅ Schema validation actually validates O ⊨ Σ (using Oxigraph SPARQL)
- ✅ Invariant enforcement actually enforces Q (using Oxigraph SPARQL)
- ✅ Dependency checking prevents invalid operations
- ✅ Error handling with rollback
- ✅ Commands work together (boot → admit → hook eval)
- ✅ State persists between commands (Oxigraph store)
- ✅ clap-noun-verb 3.4.0 properly integrated

## Implementation Order

1. **Phase 1: State Management** (Foundation for everything)
2. **Phase 11: clap-noun-verb 3.4.0** (Ensure framework is ready)
3. **Phase 2: Connector Registry** (Needed for pipeline)
4. **Phase 3: Hook Registry** (Needed for hooks)
5. **Phase 4: Receipt Store** (Needed for provenance)
6. **Phase 5: Lockchain Service** (Needed for receipts)
7. **Phase 6: Schema Validator** (Needed for validation)
8. **Phase 7: Invariant Enforcer** (Needed for validation)
9. **Phase 8: Dependency Checker** (Needed for safety)
10. **Phase 9: Error Handling** (Needed for robustness)
11. **Phase 10: Command Integration** (Tie everything together)

## Notes

- **Oxigraph**: Use Oxigraph for all RDF storage (O, Σ, Q, receipts, hooks)
- **SPARQL**: Use SPARQL queries for validation and enforcement
- **Transactions**: Use Oxigraph transactions for atomic operations
- **Performance**: Cache frequently accessed data in memory
- **Error Handling**: Always rollback on failure
- **State Consistency**: Always verify state before operations

---

**Document Version**: 1.0.0  
**Last Updated**: 2025-01-XX  
**Status**: Planning

