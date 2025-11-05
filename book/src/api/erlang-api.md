# Erlang API Reference

## Modules

### knhk_sigma
Schema registry:
```erlang
knhk_sigma:load(SchemaTTL) -> ok | {error, Reason}.
knhk_sigma:validate(O, Sigma) -> boolean().
```

### knhk_q
Invariant registry:
```erlang
knhk_q:load(QSPARQL) -> ok | {error, Reason}.
knhk_q:preserve(Q, O) -> boolean().
```

### knhk_ingest
Delta ingestion:
```erlang
knhk_ingest:admit(Delta) -> {ok, OPrime} | {error, Reason}.
```

### knhk_lockchain
Receipt storage:
```erlang
knhk_lockchain:append(Receipt) -> {ok, Hash} | {error, Reason}.
knhk_lockchain:get(ReceiptID) -> {ok, Receipt} | {error, not_found}.
knhk_lockchain:merge(ReceiptID1, ReceiptID2) -> {ok, MergedReceipt}.
```

### knhk_hooks
Hook management:
```erlang
knhk_hooks:install(HookIR) -> {ok, HookID} | {error, Reason}.
knhk_hooks:list() -> [HookID].
```

### knhk_epoch
Epoch scheduling:
```erlang
knhk_epoch:create(EpochID, Tau, Lambda) -> ok | {error, Reason}.
knhk_epoch:run(EpochID) -> {ok, Receipts} | {error, Reason}.
```

### knhk_route
Action routing:
```erlang
knhk_route:install(RouteID, Kind, Target) -> ok | {error, Reason}.
knhk_route:route(Action, RouteID) -> ok | {error, Reason}.
```

### knhk_unrdf
Cold path SPARQL stub:
```erlang
knhk_unrdf:query(SparqlQuery) -> {ok, #{endpoint => Endpoint, query => SparqlQuery}}.
```

## See Also

- [C API](c-api.md) - C API reference
- [Rust API](rust-api.md) - Rust API reference

