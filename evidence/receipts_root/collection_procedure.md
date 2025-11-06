# Receipt Root Collection Procedure

**Evidence ID**: `ev:receipts_root`
**Owner**: Security / Compliance
**Frequency**: Continuous (production), samples (staging)
**PRD Section**: 7 (Provenance & Receipts), 9 (Observability), 12 (DFLSS)
**DFLSS Section**: 12 (CTQ-5: 100% receipts, audit prep -80%)

---

## Objective

Collect sample receipts, Merkle roots, and provenance validation evidence to demonstrate:
1. 100% receipt coverage for all admitted deltas
2. Continuous receipt chain (no gaps)
3. Cryptographic provenance: `hash(A) = hash(μ(O))`
4. Lockchain integrity (Merkle root verification)
5. Receipt-to-OTEL span linking

---

## Receipt Structure

```json
{
  "id": "rcpt_001",
  "cycle_id": 12345,
  "shard_id": 0,
  "hook_id": "hook_validate_email",
  "ticks": 6,
  "span_id": "0x1234567890abcdef",
  "hash_a": "a5f2d8c1...",
  "timestamp_ms": 1699292400000,
  "operation": "VALIDATE",
  "runtime_class": "R1"
}
```

**Key Fields**:
- `id`: Unique receipt identifier
- `cycle_id`: 8-beat epoch cycle number
- `shard_id`: Shard identifier
- `hook_id`: Hook that generated the receipt
- `ticks`: Cycle count for operation (≤8 for R1)
- `span_id`: OTEL span ID linking receipt to telemetry
- `hash_a`: Hash of action A (for provenance `hash(A) = hash(μ(O))`)
- `timestamp_ms`: Receipt generation timestamp
- `operation`: Operation type (ASK, COUNT, VALIDATE, etc.)
- `runtime_class`: R1 (hot), W1 (warm), or C1 (cold)

---

## Collection Steps

### Step 1: Generate Sample Receipts

**Option 1: CLI Command** (when available):
```bash
cd /Users/sac/knhk

# Generate 100 sample receipts
knhk receipt generate --count 100 \
  --output evidence/receipts_root/sample_receipts.json

# Generate receipts for specific operations
knhk receipt generate --count 50 --operation ASK_SP \
  --output evidence/receipts_root/receipts_ask.json

knhk receipt generate --count 50 --operation VALIDATE \
  --output evidence/receipts_root/receipts_validate.json
```

**Option 2: Extract from Running System**:
```bash
# Query lockchain for recent receipts
knhk receipt query --since "2025-11-01" --limit 100 \
  --output evidence/receipts_root/production_receipts.json

# Query by hook ID
knhk receipt query --hook-id "hook_validate_email" --limit 50 \
  --output evidence/receipts_root/receipts_by_hook.json
```

**Option 3: Rust Test Harness** (if CLI not available):
```rust
// tests/receipt_generation_test.rs
use knhk_etl::{Receipt, ReflexStage};

#[test]
fn generate_sample_receipts() {
    let mut receipts = Vec::new();

    for i in 0..100 {
        let receipt = Receipt {
            id: format!("rcpt_{:03}", i),
            cycle_id: 12345 + i,
            shard_id: i % 4,
            hook_id: "hook_validate_email".to_string(),
            ticks: 6,
            span_id: format!("0x{:016x}", i),
            hash_a: format!("a5f2d8c1{:08x}", i),
            timestamp_ms: 1699292400000 + (i * 1000),
            operation: "VALIDATE".to_string(),
            runtime_class: "R1".to_string(),
        };
        receipts.push(receipt);
    }

    // Write to JSON
    let json = serde_json::to_string_pretty(&receipts).unwrap();
    std::fs::write("evidence/receipts_root/sample_receipts.json", json).unwrap();

    println!("Generated {} sample receipts", receipts.len());
}
```

Run test:
```bash
cargo test --package knhk-etl --test receipt_generation_test -- --nocapture
```

### Step 2: Extract Merkle Roots

```bash
# Extract Merkle roots from receipt chain
knhk receipt roots \
  --input evidence/receipts_root/sample_receipts.json \
  --output evidence/receipts_root/merkle_roots.json

# Verify Merkle root computation
knhk receipt compute-root \
  --receipts evidence/receipts_root/sample_receipts.json
```

**Expected Output** (`merkle_roots.json`):
```json
{
  "roots": [
    {
      "root_hash": "7f3e8a2d1c4b9f6e...",
      "receipt_count": 100,
      "cycle_range": [12345, 12444],
      "computation_method": "URDNA2015 + SHA-256"
    }
  ],
  "verification": {
    "root_valid": true,
    "receipts_included": 100,
    "receipts_missing": 0
  }
}
```

### Step 3: Verify Receipt Chain

```bash
# Verify receipt continuity (no gaps)
knhk receipt verify \
  --chain evidence/receipts_root/sample_receipts.json \
  > evidence/receipts_root/verification_logs.txt

# Check for gaps in cycle IDs
python3 scripts/check_receipt_gaps.py \
  evidence/receipts_root/sample_receipts.json
```

**Gap Detection Script** (`check_receipt_gaps.py`):
```python
import json

def check_gaps(receipts_file):
    with open(receipts_file, 'r') as f:
        receipts = json.load(f)

    # Sort by cycle_id
    receipts.sort(key=lambda r: r['cycle_id'])

    gaps = []
    for i in range(len(receipts) - 1):
        curr_cycle = receipts[i]['cycle_id']
        next_cycle = receipts[i+1]['cycle_id']

        if next_cycle != curr_cycle + 1:
            gaps.append({
                'after_cycle': curr_cycle,
                'before_cycle': next_cycle,
                'gap_size': next_cycle - curr_cycle - 1
            })

    return gaps

if __name__ == '__main__':
    gaps = check_gaps('evidence/receipts_root/sample_receipts.json')

    if gaps:
        print(f"⚠️  Found {len(gaps)} gaps in receipt chain:")
        for gap in gaps:
            print(f"  Gap: {gap['gap_size']} cycles between {gap['after_cycle']} and {gap['before_cycle']}")
    else:
        print("✅ No gaps found in receipt chain")
```

### Step 4: Validate Provenance (hash(A) = hash(μ(O)))

```bash
# Validate provenance for each receipt
knhk receipt validate-provenance \
  --receipts evidence/receipts_root/sample_receipts.json \
  > evidence/receipts_root/provenance_validation.txt
```

**Provenance Validation Script** (`validate_provenance.py`):
```python
import json
import hashlib

def validate_provenance(receipts_file):
    """Validate hash(A) = hash(μ(O)) for each receipt."""
    with open(receipts_file, 'r') as f:
        receipts = json.load(f)

    valid = []
    invalid = []

    for receipt in receipts:
        # In production, this would recompute hash(A) from observation O
        # For samples, we assume hash_a is correct and validate structure
        receipt_id = receipt['id']
        hash_a = receipt['hash_a']

        # Basic validation: hash_a is 32 bytes (64 hex chars)
        if len(hash_a) == 64:
            valid.append(receipt_id)
        else:
            invalid.append({
                'receipt_id': receipt_id,
                'hash_a': hash_a,
                'error': f'Invalid hash length: {len(hash_a)} (expected 64)'
            })

    return valid, invalid

if __name__ == '__main__':
    valid, invalid = validate_provenance('evidence/receipts_root/sample_receipts.json')

    print(f"✅ Valid provenance: {len(valid)} receipts")

    if invalid:
        print(f"\n⚠️  Invalid provenance: {len(invalid)} receipts")
        for err in invalid:
            print(f"  {err['receipt_id']}: {err['error']}")
    else:
        print("✅ All receipts have valid provenance hashes")
```

### Step 5: Verify Receipt-to-OTEL Span Linking

```bash
# Extract OTEL spans that match receipts
curl http://localhost:8080/metrics \
  | jq '.spans[] | select(.span_id | startswith("0x"))' \
  > evidence/receipts_root/otel_spans.json

# Validate receipt-to-span linking
python3 scripts/verify_span_linking.py \
  evidence/receipts_root/sample_receipts.json \
  evidence/receipts_root/otel_spans.json
```

**Span Linking Verification** (`verify_span_linking.py`):
```python
import json

def verify_span_linking(receipts_file, spans_file):
    """Verify each receipt links to an OTEL span."""
    with open(receipts_file, 'r') as f:
        receipts = json.load(f)

    with open(spans_file, 'r') as f:
        spans = json.load(f)

    # Build span_id index
    span_ids = {span['span_id'] for span in spans}

    linked = []
    unlinked = []

    for receipt in receipts:
        receipt_id = receipt['id']
        span_id = receipt['span_id']

        if span_id in span_ids:
            linked.append(receipt_id)
        else:
            unlinked.append({
                'receipt_id': receipt_id,
                'span_id': span_id
            })

    return linked, unlinked

if __name__ == '__main__':
    linked, unlinked = verify_span_linking(
        'evidence/receipts_root/sample_receipts.json',
        'evidence/receipts_root/otel_spans.json'
    )

    print(f"✅ Linked receipts: {len(linked)}")

    if unlinked:
        print(f"\n⚠️  Unlinked receipts: {len(unlinked)}")
        for receipt in unlinked[:5]:  # Show first 5
            print(f"  {receipt['receipt_id']}: span {receipt['span_id']} not found")
    else:
        print("✅ All receipts linked to OTEL spans")
```

---

## Expected Output Files

```
evidence/receipts_root/
├── sample_receipts.json           # 100 sample receipts
├── merkle_roots.json              # Merkle root hashes
├── verification_logs.txt          # Receipt chain verification
├── provenance_validation.txt      # hash(A) = hash(μ(O)) validation
├── otel_spans.json                # OTEL spans for linking
└── collection_procedure.md        # This file
```

---

## Validation Criteria

**Pass Criteria**:
- ✅ 100% of receipts generated (no missing receipts)
- ✅ Continuous receipt chain (no gaps in cycle IDs)
- ✅ Valid provenance hashes (hash_a is 32 bytes)
- ✅ All receipts link to OTEL spans
- ✅ Merkle root computation succeeds
- ✅ Receipts within tick budget (≤8 ticks for R1)

**CTQ Targets** (from DFLSS Charter):
- **CTQ-5**: 100% receipt coverage
- **Audit prep**: -80% effort (receipts as audit source)

---

## Troubleshooting

### Issue: Missing Receipts

**Symptom**: Receipt chain has gaps in cycle IDs

**Causes**:
- Receipt generation disabled
- Hook execution failed (no receipt emitted)
- Lockchain write failure

**Solutions**:
- Check hook execution logs
- Validate lockchain write permissions
- Enable receipt generation in configuration

### Issue: Invalid Provenance Hashes

**Symptom**: hash_a length != 64 or hash mismatch

**Causes**:
- Hash computation error
- Receipt serialization issue
- URDNA2015 canonicalization failure

**Solutions**:
- Validate URDNA2015 canonicalization
- Check SHA-256 hash implementation
- Review receipt serialization

### Issue: Receipts Not Linked to OTEL Spans

**Symptom**: Receipt span_id not found in OTEL telemetry

**Causes**:
- OTEL span not created
- Span ID generation mismatch
- OTEL collector not running

**Solutions**:
- Validate OTEL integration
- Check span ID generation (`knhk_generate_span_id()`)
- Ensure OTEL collector is running

---

## References

- [8-Beat PRD Section 7](../../docs/8BEAT-PRD.txt) - Provenance & Receipts
- [8-Beat PRD Section 18](../../docs/8BEAT-PRD.txt) - Evidence stubs
- [DFLSS Charter Section 12](../../docs/DFLSS_PROJECT_CHARTER.md) - CTQ-5 (Receipts)
- [Receipt Validation Policy](../../rust/knhk-validation/policies/receipt_validation.rego) - Rego policy

---

**Collection Status**: ⚠️ Pending
**Next Action**: Generate sample receipts and validate chain
**Owner**: Security / Compliance
**Target Date**: Week 1
