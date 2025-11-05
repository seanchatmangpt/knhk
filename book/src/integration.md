# Integration Guide

## Overview

This guide explains how to integrate KNHK into your system.

## End-to-End Integration

Full pipeline: Connector → ETL → Hot Path → Lockchain → OTEL

### 1. Register Connectors

```bash
knhk connect register kafka-prod urn:knhk:schema:default kafka://localhost:9092/triples
```

### 2. Configure ETL Pipeline

```bash
knhk boot init schema.ttl invariants.sparql
```

### 3. Define Covers and Reflexes

```bash
knhk cover define "SELECT ?s ?p ?o WHERE { ?s ?p ?o }" "max_run_len 8"
knhk reflex declare check-count ASK_SP 0xC0FFEE 0 8
```

### 4. Run Pipeline

```bash
knhk pipeline run --connectors kafka-prod
```

## Connector Integration

See [Connectors](connectors.md) for details.

## ETL Pipeline Integration

See [ETL Pipeline](pipeline.md) for details.

## Lockchain Integration

See [Lockchain](lockchain.md) for details.

