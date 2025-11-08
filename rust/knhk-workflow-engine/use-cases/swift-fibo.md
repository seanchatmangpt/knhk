# SWIFT FIBO Case Study

Enterprise financial workflow case study using SWIFT and FIBO.

## Overview

This case study demonstrates workflow patterns in enterprise financial contexts:

- **SWIFT**: Society for Worldwide Interbank Financial Telecommunication
- **FIBO**: Financial Industry Business Ontology
- **Multi-party Transactions**: Financial transaction workflows
- **Compliance**: Regulatory requirements
- **Audit Trails**: Provenance tracking

## Use Cases

### Payment Processing

SWIFT payment message processing workflows:

```rust
let spec = create_swift_payment_workflow();
let spec_id = engine.register_workflow(spec).await?;
```

### Compliance Checks

FIBO compliance validation workflows:

```rust
let spec = create_fibo_compliance_workflow();
let spec_id = engine.register_workflow(spec).await?;
```

## Patterns Used

- **Sequence**: Payment processing steps
- **Parallel Split**: Multiple compliance checks
- **Synchronization**: Wait for all checks
- **Exclusive Choice**: Approval/rejection paths

## Next Steps

- [Full Case Study](../docs/SWIFT_FIBO_CASE_STUDY.md) - Complete documentation
- [Fortune 5 Use Cases](fortune5.md) - Enterprise features

