# CLI Commands Reference

## Boot - System Initialization

**Initialize Σ and Q**
```bash
knhk boot init <sigma.ttl> <q.sparql>
```

Example:
```bash
knhk boot init schema.ttl invariants.sparql
```

## Connect - Connector Management

**Register Connector**
```bash
knhk connect register <name> <schema> <source>
```

**List Connectors**
```bash
knhk connect list
```

Example:
```bash
knhk connect register kafka-prod urn:knhk:schema:default kafka://localhost:9092/triples
knhk connect list
```

## Cover - Cover Definition

**Define Cover**
```bash
knhk cover define <select> <shard>
```

**List Covers**
```bash
knhk cover list
```

Example:
```bash
knhk cover define "SELECT ?s ?p ?o WHERE { ?s ?p ?o }" "max_run_len 8"
```

## Admit - Delta Admission

**Admit Delta**
```bash
knhk admit delta <delta_file>
```

Example:
```bash
knhk admit delta delta.json
```

## Reflex - Reflex Declaration

**Declare Reflex**
```bash
knhk reflex declare <name> <op> <pred> <off> <len>
```

**List Reflexes**
```bash
knhk reflex list
```

Example:
```bash
knhk reflex declare check-count ASK_SP 0xC0FFEE 0 8
```

Valid operations (H_hot set):
- ASK_SP, COUNT_SP_GE, COUNT_SP_LE, COUNT_SP_EQ
- ASK_SPO, ASK_OP, UNIQUE_SP
- COUNT_OP_GE, COUNT_OP_LE, COUNT_OP_EQ
- COMPARE_O_EQ, COMPARE_O_GT, COMPARE_O_LT, COMPARE_O_GE, COMPARE_O_LE
- CONSTRUCT8

## Epoch - Epoch Operations

**Create Epoch**
```bash
knhk epoch create <id> <tau> <lambda>
```

**Run Epoch**
```bash
knhk epoch run <id>
```

**List Epochs**
```bash
knhk epoch list
```

Example:
```bash
knhk epoch create epoch1 8 "reflex1,reflex2"
knhk epoch run epoch1
```

## Route - Action Routing

**Install Route**
```bash
knhk route install <name> <kind> <target>
```

**List Routes**
```bash
knhk route list
```

Route kinds:
- `webhook` - HTTP webhook (http:// or https://)
- `kafka` - Kafka topic (kafka://brokers/topic)
- `grpc` - gRPC endpoint (grpc://host:port/service/method)
- `lockchain` - Git lockchain (file:// or git://)

Example:
```bash
knhk route install webhook1 webhook https://api.example.com/webhook
knhk route install kafka1 kafka kafka://localhost:9092/actions
```

## Receipt - Receipt Operations

**Get Receipt**
```bash
knhk receipt get <id>
```

**Merge Receipts**
```bash
knhk receipt merge <id1,id2,id3>
```

**List Receipts**
```bash
knhk receipt list
```

## Pipeline - ETL Pipeline

**Run Pipeline**
```bash
knhk pipeline run [--connectors <ids>] [--schema <iri>]
```

**Pipeline Status**
```bash
knhk pipeline status
```

Example:
```bash
knhk pipeline run --connectors kafka-prod
knhk pipeline status
```

## Metrics - OTEL Metrics

**Get Metrics**
```bash
knhk metrics get
```

## Coverage - Dark Matter Coverage

**Get Coverage**
```bash
knhk coverage get
```

