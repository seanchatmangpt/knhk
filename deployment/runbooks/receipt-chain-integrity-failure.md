# Runbook: Receipt Chain Integrity Failure

**Severity**: P0 (CRITICAL)
**Owner**: Engineering Manager + Security Team
**Last Updated**: 2025-11-16

## ⚠️ CRITICAL ALERT

Receipt chain integrity failure indicates potential data corruption or tampering. This is the highest severity incident and requires immediate escalation to Engineering Manager and Security Lead.

**DO NOT PROCEED WITHOUT APPROVAL FROM ENGINEERING MANAGER**

## Symptoms

- Receipt chain verification failing
- Alert: `knhk_receipt_chain_breaks_total > 0`
- Signature verification failures
- Missing parent receipts
- Timestamp violations (retrocausation detected)

## Immediate Response (Within 5 minutes)

### 1. STOP ALL WRITES IMMEDIATELY

```bash
# Scale down all application instances to prevent new receipts
kubectl scale deployment knhk-closed-loop --replicas=0 -n knhk

# Verify all pods are terminated
kubectl get pods -n knhk -l app=knhk-closed-loop
```

### 2. Create Emergency Backup

```bash
# Take full database snapshot
pgbackrest --stanza=knhk --type=full backup

# Verify backup completed
pgbackrest --stanza=knhk info

# Export receipt table to separate file
kubectl exec -it postgres-0 -n knhk -- \
    pg_dump -U postgres -d knhk -t receipts -f /backup/receipts-emergency-$(date +%s).sql
```

### 3. Escalate Immediately

- **Page**: Engineering Manager (P0 alert)
- **Page**: Security Lead (P0 alert)
- **Notify**: CTO, Legal (if customer data affected)
- **Create**: Incident channel in Slack (#incident-receipt-chain-YYYYMMDD)

### 4. Run Integrity Verification

```bash
# Run comprehensive chain verification
kubectl exec -it deploy/knhk-closed-loop -n knhk -- \
    knhk receipts verify-chain --verbose --format json > /tmp/chain-verification.json

# Parse results
cat /tmp/chain-verification.json | jq '{
    total_receipts: .total_count,
    verified_receipts: .verified_count,
    signature_failures: .signature_failures,
    chain_breaks: .chain_breaks,
    retrocausation_violations: .retrocausation_violations
}'
```

Expected output:
```json
{
  "total_receipts": 1000000,
  "verified_receipts": 999950,
  "signature_failures": [],
  "chain_breaks": ["receipt-abc123", "receipt-def456"],
  "retrocausation_violations": []
}
```

## Diagnosis

### Identify Break Point

```bash
# Find first broken receipt
FIRST_BREAK=$(cat /tmp/chain-verification.json | jq -r '.chain_breaks[0]')

echo "First broken receipt: $FIRST_BREAK"

# Get details of broken receipt and neighbors
kubectl exec -it postgres-0 -n knhk -- \
    psql -U postgres -d knhk -c "
        SELECT id, created_at, parent_id, parent_hash, hash, signature
        FROM receipts
        WHERE id IN (
            SELECT id FROM receipts WHERE id = '$FIRST_BREAK'
            UNION
            SELECT parent_id FROM receipts WHERE id = '$FIRST_BREAK'
            UNION
            SELECT id FROM receipts WHERE parent_id = '$FIRST_BREAK'
        )
        ORDER BY created_at;
    "
```

### Check for Common Causes

**1. Key Rotation Issue**
```bash
# Check if break coincides with key rotation
kubectl exec -it postgres-0 -n knhk -- \
    psql -U postgres -d knhk -c "
        SELECT r.id, r.created_at, r.signature_key_version
        FROM receipts r
        WHERE r.id = '$FIRST_BREAK'
           OR r.created_at BETWEEN (
               SELECT created_at - INTERVAL '1 hour'
               FROM receipts
               WHERE id = '$FIRST_BREAK'
           ) AND (
               SELECT created_at + INTERVAL '1 hour'
               FROM receipts
               WHERE id = '$FIRST_BREAK'
           )
        ORDER BY r.created_at;
    "
```

**2. Database Corruption**
```bash
# Check PostgreSQL table integrity
kubectl exec -it postgres-0 -n knhk -- \
    psql -U postgres -c "
        SELECT schemaname, tablename, n_tup_ins, n_tup_upd, n_tup_del
        FROM pg_stat_user_tables
        WHERE tablename = 'receipts';
    "

# If n_tup_upd or n_tup_del > 0, receipts were modified (SHOULD NEVER HAPPEN)
```

**3. Clock Skew / Time Synchronization**
```bash
# Check for timestamp anomalies
kubectl exec -it postgres-0 -n knhk -- \
    psql -U postgres -d knhk -c "
        SELECT id, created_at, parent_id,
               (SELECT created_at FROM receipts WHERE id = parent_id) as parent_created_at,
               created_at - (SELECT created_at FROM receipts WHERE id = parent_id) as time_delta
        FROM receipts
        WHERE parent_id IS NOT NULL
          AND created_at < (SELECT created_at FROM receipts WHERE id = parent_id)
        LIMIT 10;
    "
```

## Resolution Options

### Option 1: Acceptable Break (False Positive)

**Use when**: Break is due to expected system behavior (e.g., parallel ingestion, test data)

```bash
# Document the break as acceptable
kubectl exec -it deploy/knhk-closed-loop -n knhk -- \
    knhk receipts mark-exception \
    --receipt-id "$FIRST_BREAK" \
    --reason "Parallel ingestion race condition" \
    --approved-by "$USER"

# Resume operations
kubectl scale deployment knhk-closed-loop --replicas=3 -n knhk
```

### Option 2: Point-in-Time Recovery (Data Loss)

**Use when**: Corruption is localized and can be rolled back

**⚠️ REQUIRES BUSINESS APPROVAL - DATA LOSS**

```bash
# Identify last known good timestamp
LAST_GOOD_TIME=$(cat /tmp/chain-verification.json | jq -r '.last_verified_timestamp')

echo "Last verified receipt: $LAST_GOOD_TIME"
echo "⚠️  WARNING: Will lose all receipts after $LAST_GOOD_TIME"
read -p "Type 'RESTORE' to confirm: " CONFIRM

if [ "$CONFIRM" == "RESTORE" ]; then
    # Restore to last good state
    ./deployment/postgres/restore-pitr.sh "$LAST_GOOD_TIME"
fi
```

### Option 3: Manual Repair (Advanced)

**Use when**: Break can be manually fixed (e.g., incorrect parent hash)

**⚠️ ONLY WITH ENGINEERING MANAGER APPROVAL**

```bash
# Example: Fix incorrect parent hash
# This should NEVER be done lightly - receipts are immutable

kubectl exec -it postgres-0 -n knhk -- \
    psql -U postgres -d knhk -c "
        -- Verify current state
        SELECT id, parent_hash, hash
        FROM receipts
        WHERE id = '$FIRST_BREAK';

        -- ONLY if approved by Engineering Manager:
        -- UPDATE receipts
        -- SET parent_hash = '<correct_hash>'
        -- WHERE id = '$FIRST_BREAK';
    "
```

## Post-Incident Actions

### 1. Root Cause Analysis

Create post-incident review document:
- Timeline of events
- Root cause (technical failure, human error, attack)
- Impact assessment (data lost, customer impact)
- Prevention measures

### 2. Customer Notification

If customer data affected:
```markdown
Subject: Security Incident Notification - Receipt Chain Integrity

Dear [Customer],

We are writing to inform you of a security incident that affected the integrity
of audit trail receipts in the KNHK system between [START_TIME] and [END_TIME].

Impact:
- [X] receipts were affected
- No customer data was lost
- Audit trail was temporarily disrupted

Resolution:
- Issue was identified at [TIME]
- System was secured at [TIME]
- Normal operations resumed at [TIME]

Actions taken:
- [List of remediation steps]

We apologize for any inconvenience this may have caused.

For questions, contact: security@company.com
```

### 3. Enhanced Monitoring

Add additional safeguards:

```yaml
# Prometheus alert for ANY chain break
- alert: KNHKReceiptChainBreakDetected
  expr: increase(knhk_receipt_chain_breaks_total[5m]) > 0
  for: 1m
  labels:
    severity: critical
  annotations:
    summary: "CRITICAL: Receipt chain break detected"
    description: "{{ $value }} chain breaks in last 5 minutes"
```

### 4. Audit Log Review

```bash
# Review all audit logs during incident window
kubectl exec -it postgres-0 -n knhk -- \
    psql -U postgres -d knhk -c "
        SELECT timestamp, event_type, actor_id, action, resource_id, outcome
        FROM audit_log
        WHERE timestamp BETWEEN '$INCIDENT_START' AND '$INCIDENT_END'
        ORDER BY timestamp;
    " > /tmp/incident-audit-log.txt
```

## Prevention Measures

1. **Automated Daily Chain Verification**
   ```bash
   # Add to cron: 0 2 * * * /deployment/scripts/verify-receipt-chain.sh
   ```

2. **Immutable Receipts Enforcement**
   ```sql
   -- Ensure receipts table has no UPDATE/DELETE rules
   CREATE RULE receipts_no_update AS ON UPDATE TO receipts DO INSTEAD NOTHING;
   CREATE RULE receipts_no_delete AS ON DELETE TO receipts DO INSTEAD NOTHING;
   ```

3. **Multi-Region Replication**
   - Replicate receipts to separate read-only database
   - Use for integrity verification

4. **Hardware Security Module (HSM)**
   - Store signing keys in HSM (Vault + CloudHSM)
   - Prevent key compromise

## Related Runbooks

- [Data Corruption Recovery](data-corruption.md)
- [Point-in-Time Recovery](postgres-pitr.md)
- [Security Incident Response](security-incident.md)

## Emergency Contacts

- **Engineering Manager**: [Name] - [Phone]
- **Security Lead**: [Name] - [Phone]
- **CTO**: [Name] - [Phone]
- **Legal**: [Name] - [Email]
- **PagerDuty**: +1-XXX-XXX-XXXX
