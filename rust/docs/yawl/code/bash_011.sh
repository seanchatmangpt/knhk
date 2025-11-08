# Submit workflow
curl -X POST http://localhost:8080/api/v1/workflows \
  -H "Content-Type: application/json" \
  -H "Authorization: PQC signature=<sig> keyid=<key>" \
  -d '{
    "pattern_sequence": [1, 2, 3],
    "deadline": "2025-08-22T01:00:00Z",
    "priority": "normal",
    "input_data": {"param": "value"}
  }'

# Check status
curl http://localhost:8080/api/v1/workflows/wf_01HX8K9... \
  -H "Authorization: PQC signature=<sig> keyid=<key>"

# Get metrics
curl http://localhost:8080/api/v1/metrics \
  -H "Authorization: PQC signature=<sig> keyid=<key>"