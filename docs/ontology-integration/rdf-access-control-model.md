# RDF Access Control Model for YAWL Ontology Integration

**Version:** 1.0
**Date:** 2025-11-08
**Status:** Implementation-Ready Design
**Author:** Security Analyst (ULTRATHINK Swarm)
**Ontology:** YAWL 4.0 RDF/OWL
**Framework:** knhk Workflow Engine

---

## Executive Summary

This document defines a comprehensive Role-Based Access Control (RBAC) model for RDF graph data in the knhk YAWL ontology integration. The model provides multi-level security from graph-level isolation to triple-level permissions, with SPARQL query filtering and integration with existing knhk authentication.

**Key Security Objectives:**
1. **Graph-Level Isolation**: Separate workflow specifications by organizational boundaries
2. **Triple-Level Permissions**: Fine-grained control over individual RDF triples
3. **SPARQL Query Filtering**: Automatic permission enforcement in queries
4. **Role-Based Access**: Align with YAWL resource allocation model
5. **Audit Trail Integration**: Lockchain immutable logging of all access

**Threat Model Coverage:**
- Unauthorized workflow specification access
- Resource allocation tampering
- Sensitive metadata exposure
- Cross-workflow information leakage
- Privilege escalation via SPARQL injection

---

## 1. Access Control Architecture

### 1.1 Security Layers

```
┌─────────────────────────────────────────────────────────┐
│ Layer 5: Application Layer (SPARQL Query Interface)    │
│  - Query rewriting with permission filters             │
│  - Result set filtering                                │
│  - Audit logging                                       │
└─────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────┐
│ Layer 4: Graph-Level Access Control                    │
│  - Named graph isolation (yawl:SpecificationSet)       │
│  - Organization-based graph partitioning               │
│  - Graph ownership and delegation                      │
└─────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────┐
│ Layer 3: Class-Level Permissions                       │
│  - Access by RDF class (yawl:Task, yawl:Resourcing)    │
│  - Resource type filtering (Participant/Role)          │
│  - Decomposition visibility control                    │
└─────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────┐
│ Layer 2: Property-Level Permissions                    │
│  - Datatype property filtering (yawl:creator, etc.)    │
│  - Object property access control                      │
│  - Metadata protection (Dublin Core)                   │
└─────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────┐
│ Layer 1: Triple-Level Row Security                     │
│  - Individual triple permissions (S-P-O)               │
│  - Attribute-based access control (ABAC)               │
│  - Context-sensitive filtering                         │
└─────────────────────────────────────────────────────────┘
```

### 1.2 Named Graph Strategy

**Graph Partitioning Scheme:**

Each YAWL specification is stored in a **separate named graph** to enable graph-level isolation:

```turtle
# Graph 1: Specification A (Organization: FinanceCorp, Owner: alice@finance.com)
<http://knhk.io/workflows/spec-a> {
    :spec-a a yawl:Specification ;
        yawl:uri "http://knhk.io/workflows/spec-a" ;
        yawl:hasMetadata [
            yawl:creator "alice@finance.com" ;
            yawl:created "2025-11-08"^^xsd:date ;
            yawl:title "Financial Approval Workflow"
        ] ;
        yawl:hasDecomposition :net-a .

    :net-a a yawl:Net ;
        yawl:isRootNet true ;
        yawl:hasTask :task-approval .

    :task-approval a yawl:Task ;
        yawl:hasResourcing :resourcing-finance .

    :resourcing-finance a yawl:Resourcing ;
        yawl:hasOffer [
            yawl:hasDistributionSet [
                yawl:hasInitialSet [
                    yawl:role "finance-manager"
                ]
            ]
        ] .
}

# Graph 2: Specification B (Organization: HRCorp, Owner: bob@hr.com)
<http://knhk.io/workflows/spec-b> {
    :spec-b a yawl:Specification ;
        yawl:uri "http://knhk.io/workflows/spec-b" ;
        yawl:hasMetadata [
            yawl:creator "bob@hr.com" ;
            yawl:created "2025-11-08"^^xsd:date ;
            yawl:title "Employee Onboarding Workflow"
        ] ;
        yawl:hasDecomposition :net-b .
}
```

**Graph Naming Convention:**
- **Specification Graphs**: `http://knhk.io/workflows/{spec-id}`
- **System Metadata Graph**: `http://knhk.io/system/metadata`
- **Access Control Graph**: `http://knhk.io/system/acl`
- **Audit Log Graph**: `http://knhk.io/system/audit`

### 1.3 Access Control Graph (ACL Graph)

The Access Control Graph stores all permission rules:

```turtle
<http://knhk.io/system/acl> {
    # Graph-level permission
    :acl-spec-a a :GraphPermission ;
        :graph <http://knhk.io/workflows/spec-a> ;
        :grantee "alice@finance.com" ;
        :role "owner" ;
        :permissions [ :read true ; :write true ; :delete true ; :grant true ] .

    :acl-spec-a-reader a :GraphPermission ;
        :graph <http://knhk.io/workflows/spec-a> ;
        :grantee "finance-team" ;
        :role "reader" ;
        :permissions [ :read true ; :write false ; :delete false ; :grant false ] .

    # Class-level permission (restrict access to Resourcing data)
    :acl-resourcing a :ClassPermission ;
        :graph <http://knhk.io/workflows/spec-a> ;
        :class yawl:Resourcing ;
        :grantee "finance-manager" ;
        :permissions [ :read true ; :write true ; :delete false ] .

    # Property-level permission (hide creator metadata from readers)
    :acl-creator a :PropertyPermission ;
        :graph <http://knhk.io/workflows/spec-a> ;
        :property yawl:creator ;
        :grantee "finance-team" ;
        :permissions [ :read false ] .

    # Triple-level permission (specific task access)
    :acl-task-approval a :TriplePermission ;
        :subject :task-approval ;
        :predicate rdf:type ;
        :object yawl:Task ;
        :graph <http://knhk.io/workflows/spec-a> ;
        :grantee "task-executor" ;
        :permissions [ :read true ; :write false ] .
}
```

---

## 2. Role-Based Access Control (RBAC)

### 2.1 YAWL-Aligned Role Hierarchy

**Alignment with YAWL Resourcing Model:**

The YAWL ontology defines resource allocation through:
- `yawl:ResourcingResourceType` (Participant, Role)
- `yawl:ResourcingPrivilege` (canSuspend, canDelegate, etc.)
- `yawl:ResourcingSet` (sets of participants and roles)

**knhk RBAC Roles:**

```yaml
# System-Level Roles (knhk Framework)
SystemRoles:
  - system-admin:
      description: "Full system access (all graphs, all operations)"
      permissions:
        - graph: "*"
          operations: [CREATE, READ, UPDATE, DELETE, GRANT]

  - workflow-architect:
      description: "Create and design workflow specifications"
      permissions:
        - graph: "owned-graphs"
          operations: [CREATE, READ, UPDATE, DELETE]
        - class: [yawl:Specification, yawl:Net, yawl:Task, yawl:Decomposition]
          operations: [CREATE, READ, UPDATE, DELETE]

  - compliance-auditor:
      description: "Read-only access for compliance review"
      permissions:
        - graph: "*"
          operations: [READ]
        - audit-log: true

  - security-manager:
      description: "Manage access control policies"
      permissions:
        - graph: "http://knhk.io/system/acl"
          operations: [CREATE, READ, UPDATE, DELETE]

# Workflow-Level Roles (YAWL Specification)
WorkflowRoles:
  - specification-owner:
      description: "Owner of a specific workflow specification"
      permissions:
        - graph: "{spec-graph}"
          operations: [READ, UPDATE, DELETE, GRANT]

  - specification-editor:
      description: "Edit workflow tasks and configuration"
      permissions:
        - graph: "{spec-graph}"
          operations: [READ, UPDATE]
        - classes: [yawl:Task, yawl:Variable, yawl:FlowsInto]
          operations: [UPDATE]

  - specification-reader:
      description: "View workflow specification (read-only)"
      permissions:
        - graph: "{spec-graph}"
          operations: [READ]

# Task-Level Roles (YAWL Resource Allocation)
TaskRoles:
  - task-performer:
      description: "Execute assigned tasks (mapped from yawl:Participant)"
      permissions:
        - triple-pattern: "?task a yawl:Task . ?task yawl:hasResourcing ?res . ?res yawl:hasOffer/yawl:hasDistributionSet/yawl:hasInitialSet [ yawl:participant '{user-id}' ]"
          operations: [READ, EXECUTE]

  - resource-manager:
      description: "Manage resource allocation (mapped from yawl:Role)"
      permissions:
        - class: yawl:Resourcing
          operations: [READ, UPDATE]
        - properties: [yawl:hasOffer, yawl:hasAllocate, yawl:hasPrivileges]
          operations: [READ, UPDATE]
```

### 2.2 Permission Model

**Permission Structure:**

```rust
// knhk permission representation
pub struct Permission {
    pub subject: PermissionSubject,
    pub actions: Vec<Action>,
    pub conditions: Vec<Condition>,
    pub effect: Effect,  // Allow or Deny
}

pub enum PermissionSubject {
    Graph(String),                    // Named graph URI
    Class(String),                    // RDF class (e.g., yawl:Task)
    Property(String),                 // RDF property (e.g., yawl:creator)
    Triple { s: String, p: String, o: String },  // Specific triple
    TriplePattern(String),            // SPARQL triple pattern
}

pub enum Action {
    Read,      // SPARQL SELECT, CONSTRUCT, DESCRIBE
    Write,     // SPARQL INSERT DATA
    Update,    // SPARQL DELETE/INSERT
    Delete,    // SPARQL DELETE DATA
    Execute,   // Execute workflow task
    Grant,     // Grant permissions to others
}

pub enum Condition {
    TimeRange { start: DateTime, end: DateTime },
    IPAddress { allowed: Vec<IpAddr> },
    OrganizationMatch { org: String },
    OwnershipCheck { property: String },  // e.g., yawl:creator = user
}

pub enum Effect {
    Allow,
    Deny,   // Deny takes precedence over Allow
}
```

### 2.3 Permission Evaluation Algorithm

**Deny-Takes-Precedence Policy:**

```python
def evaluate_permission(user: User, action: Action, resource: RdfResource) -> bool:
    """
    Evaluate whether user can perform action on resource.

    Algorithm:
    1. Collect all permissions that apply to (user, action, resource)
    2. If ANY permission is Deny, return False
    3. If ANY permission is Allow, return True
    4. If no permissions match, return False (default deny)
    """

    # Step 1: Collect applicable permissions
    permissions = []

    # User's direct permissions
    permissions.extend(user.direct_permissions)

    # Role-based permissions
    for role in user.roles:
        permissions.extend(role.permissions)

    # Group-based permissions
    for group in user.groups:
        permissions.extend(group.permissions)

    # Filter permissions that match (action, resource)
    applicable = []
    for perm in permissions:
        if matches_action(perm, action) and matches_resource(perm, resource):
            # Evaluate conditions (time, IP, etc.)
            if evaluate_conditions(perm.conditions, user.context):
                applicable.append(perm)

    # Step 2: Check for explicit Deny
    for perm in applicable:
        if perm.effect == Effect.Deny:
            audit_log(user, action, resource, result="DENIED", reason=perm.id)
            return False

    # Step 3: Check for explicit Allow
    for perm in applicable:
        if perm.effect == Effect.Allow:
            audit_log(user, action, resource, result="ALLOWED", reason=perm.id)
            return True

    # Step 4: Default deny
    audit_log(user, action, resource, result="DENIED", reason="default-deny")
    return False

def matches_resource(permission: Permission, resource: RdfResource) -> bool:
    """Check if permission applies to resource."""

    if isinstance(permission.subject, PermissionSubject.Graph):
        return permission.subject.graph == resource.graph or permission.subject.graph == "*"

    elif isinstance(permission.subject, PermissionSubject.Class):
        # Check if resource is instance of permitted class
        return resource.has_type(permission.subject.class_uri)

    elif isinstance(permission.subject, PermissionSubject.Property):
        # Check if resource has the property
        return resource.has_property(permission.subject.property_uri)

    elif isinstance(permission.subject, PermissionSubject.Triple):
        # Exact triple match
        return (resource.subject == permission.subject.s and
                resource.predicate == permission.subject.p and
                resource.object == permission.subject.o)

    elif isinstance(permission.subject, PermissionSubject.TriplePattern):
        # SPARQL pattern match
        return sparql_match(permission.subject.pattern, resource)

    return False
```

---

## 3. SPARQL Query Filtering

### 3.1 Query Rewriting Strategy

**Automatic Permission Enforcement:**

All SPARQL queries are **automatically rewritten** to include permission filters before execution.

**Original Query (from user):**
```sparql
SELECT ?task ?name
WHERE {
    ?task a yawl:Task ;
          yawl:name ?name .
}
```

**Rewritten Query (with permission filters):**
```sparql
SELECT ?task ?name
WHERE {
    # Graph-level filter (only accessible graphs)
    GRAPH ?g {
        ?task a yawl:Task ;
              yawl:name ?name .
    }

    # Permission check filter
    FILTER EXISTS {
        GRAPH <http://knhk.io/system/acl> {
            ?acl a :GraphPermission ;
                :graph ?g ;
                :grantee "alice@finance.com" ;  # Current user
                :permissions [ :read true ] .
        }
    }

    # OR user is system-admin
    UNION {
        GRAPH <http://knhk.io/system/acl> {
            ?user_acl a :UserRole ;
                :user "alice@finance.com" ;
                :role "system-admin" .
        }
    }
}
```

### 3.2 Query Rewriter Implementation

**Query Rewriting Rules:**

```yaml
RewriteRules:
  - name: "graph-level-filter"
    description: "Add GRAPH clause to restrict to accessible graphs"
    pattern: "WHERE { ?s ?p ?o }"
    rewrite: "WHERE { GRAPH ?g { ?s ?p ?o } FILTER(?g IN (${accessible_graphs})) }"

  - name: "class-level-filter"
    description: "Filter RDF classes based on permissions"
    pattern: "?s a ${class_uri} ."
    condition: "user has permission for class_uri"
    rewrite: "?s a ${class_uri} . FILTER EXISTS { ${class_permission_check} }"

  - name: "property-level-filter"
    description: "Remove restricted properties from SELECT"
    pattern: "SELECT ${vars} WHERE { ${pattern} }"
    rewrite: |
      SELECT ${allowed_vars} WHERE {
        ${pattern}
        # Filter out restricted properties
        FILTER NOT EXISTS {
          GRAPH <http://knhk.io/system/acl> {
            ?prop_acl a :PropertyPermission ;
              :property ${restricted_property} ;
              :grantee "${user}" ;
              :permissions [ :read false ] .
          }
        }
      }

  - name: "triple-level-filter"
    description: "Apply triple-level row security"
    pattern: "?s ?p ?o ."
    rewrite: |
      ?s ?p ?o .
      FILTER EXISTS {
        # Check triple-level permission
        GRAPH <http://knhk.io/system/acl> {
          {
            # Explicit triple permission
            ?triple_acl a :TriplePermission ;
              :subject ?s ;
              :predicate ?p ;
              :object ?o ;
              :grantee "${user}" ;
              :permissions [ :read true ] .
          } UNION {
            # OR graph-level permission covers this triple
            ?graph_acl a :GraphPermission ;
              :graph ?g ;
              :grantee "${user}" ;
              :permissions [ :read true ] .
            GRAPH ?g { ?s ?p ?o }
          }
        }
      }
```

### 3.3 SPARQL Update Filtering

**INSERT/DELETE Operations:**

```sparql
# User attempts INSERT
INSERT DATA {
    GRAPH <http://knhk.io/workflows/spec-a> {
        :new-task a yawl:Task ;
            yawl:name "New Approval Task" .
    }
}

# Rewritten with permission check (executed BEFORE insert)
ASK {
    GRAPH <http://knhk.io/system/acl> {
        ?acl a :GraphPermission ;
            :graph <http://knhk.io/workflows/spec-a> ;
            :grantee "alice@finance.com" ;
            :permissions [ :write true ] .
    }
}

# If ASK returns false, reject INSERT with error:
# "PermissionDenied: User alice@finance.com does not have WRITE permission on graph <http://knhk.io/workflows/spec-a>"
```

**DELETE Operations with Audit:**

```sparql
# User attempts DELETE
DELETE DATA {
    GRAPH <http://knhk.io/workflows/spec-a> {
        :task-approval yawl:name "Old Name" .
    }
}

# Rewritten with audit logging
DELETE {
    GRAPH <http://knhk.io/workflows/spec-a> {
        :task-approval yawl:name ?old_name .
    }
}
INSERT {
    GRAPH <http://knhk.io/system/audit> {
        [] a :AuditEvent ;
            :timestamp "2025-11-08T10:30:00Z"^^xsd:dateTime ;
            :user "alice@finance.com" ;
            :action :Delete ;
            :subject :task-approval ;
            :predicate yawl:name ;
            :oldValue ?old_name ;
            :graph <http://knhk.io/workflows/spec-a> .
    }
}
WHERE {
    GRAPH <http://knhk.io/workflows/spec-a> {
        :task-approval yawl:name ?old_name .
    }

    # Permission check
    GRAPH <http://knhk.io/system/acl> {
        ?acl a :GraphPermission ;
            :graph <http://knhk.io/workflows/spec-a> ;
            :grantee "alice@finance.com" ;
            :permissions [ :delete true ] .
    }
}
```

---

## 4. Integration with knhk Authentication

### 4.1 Authentication Flow

```
┌──────────┐      ┌─────────────┐      ┌───────────┐      ┌──────────┐
│  User    │      │ knhk Auth   │      │ RDF ACL   │      │ SPARQL   │
│ Request  │─────▶│  Service    │─────▶│ Evaluator │─────▶│ Engine   │
└──────────┘      └─────────────┘      └───────────┘      └──────────┘
     │                   │                    │                 │
     │ 1. SPARQL Query   │                    │                 │
     │──────────────────▶│                    │                 │
     │                   │ 2. Authenticate    │                 │
     │                   │    (JWT/Session)   │                 │
     │                   │                    │                 │
     │                   │ 3. Get User Roles  │                 │
     │                   │    & Permissions   │                 │
     │                   │───────────────────▶│                 │
     │                   │                    │                 │
     │                   │ 4. Evaluate ACL    │                 │
     │                   │    (RBAC + ABAC)   │                 │
     │                   │◀───────────────────│                 │
     │                   │                    │                 │
     │                   │ 5. Rewrite Query   │                 │
     │                   │    (add filters)   │                 │
     │                   │────────────────────────────────────▶│
     │                   │                    │                 │
     │                   │ 6. Execute Query   │                 │
     │                   │◀────────────────────────────────────│
     │                   │                    │                 │
     │ 7. Filtered       │                    │                 │
     │    Results        │                    │                 │
     │◀──────────────────│                    │                 │
```

### 4.2 User Identity Mapping

**knhk User → RDF Identity:**

```turtle
<http://knhk.io/system/acl> {
    # User identity
    :user-alice a :User ;
        :email "alice@finance.com" ;
        :knhk_user_id "user-12345" ;
        :organization "FinanceCorp" ;
        :created "2025-01-01"^^xsd:date ;
        :status "active" .

    # User roles
    :user-alice :hasRole :role-finance-manager .
    :user-alice :hasRole :role-workflow-architect .

    # User groups
    :user-alice :memberOf :group-finance-team .

    # Direct permissions (override role permissions)
    :direct-perm-alice a :GraphPermission ;
        :grantee "alice@finance.com" ;
        :graph <http://knhk.io/workflows/spec-a> ;
        :role "owner" ;
        :permissions [ :read true ; :write true ; :delete true ; :grant true ] .
}
```

**Role Definitions:**

```turtle
<http://knhk.io/system/acl> {
    :role-finance-manager a :Role ;
        :name "finance-manager" ;
        :description "Financial workflow manager" ;
        :permissions [
            :class yawl:Resourcing ;
            :actions [ :read true ; :write true ]
        ] ;
        :permissions [
            :class yawl:Task ;
            :actions [ :read true ; :write true ]
        ] .

    :role-workflow-architect a :Role ;
        :name "workflow-architect" ;
        :description "Workflow designer" ;
        :permissions [
            :class yawl:Specification ;
            :actions [ :read true ; :write true ; :delete true ]
        ] ;
        :permissions [
            :class yawl:Net ;
            :actions [ :read true ; :write true ]
        ] .
}
```

### 4.3 Session Context

**Runtime Security Context:**

```rust
pub struct SecurityContext {
    pub user: User,
    pub session_id: String,
    pub roles: Vec<Role>,
    pub groups: Vec<Group>,
    pub permissions: Vec<Permission>,
    pub ip_address: IpAddr,
    pub timestamp: DateTime<Utc>,
    pub organization: String,
}

impl SecurityContext {
    pub fn from_session(session_token: &str) -> Result<Self, AuthError> {
        // 1. Validate JWT/session token
        let claims = validate_token(session_token)?;

        // 2. Load user from knhk auth service
        let user = load_user(claims.user_id)?;

        // 3. Load roles and permissions from RDF ACL graph
        let roles = load_user_roles(&user)?;
        let permissions = compute_effective_permissions(&user, &roles)?;

        // 4. Build security context
        Ok(SecurityContext {
            user,
            session_id: claims.session_id,
            roles,
            groups: load_user_groups(&user)?,
            permissions,
            ip_address: claims.ip_address,
            timestamp: Utc::now(),
            organization: user.organization.clone(),
        })
    }

    pub fn can_access_graph(&self, graph_uri: &str) -> bool {
        self.permissions.iter().any(|p| {
            matches!(p.subject, PermissionSubject::Graph(g) if g == graph_uri || g == "*")
                && p.effect == Effect::Allow
        })
    }

    pub fn accessible_graphs(&self) -> Vec<String> {
        let mut graphs = vec![];
        for perm in &self.permissions {
            if let PermissionSubject::Graph(g) = &perm.subject {
                if perm.effect == Effect::Allow {
                    graphs.push(g.clone());
                }
            }
        }
        graphs
    }
}
```

---

## 5. Sensitive Data Protection

### 5.1 YAWL Ontology Sensitive Data Categories

**Category 1: Resource Allocation Data (CRITICAL)**

```yaml
SensitiveClasses:
  - yawl:Resourcing
  - yawl:ResourcingOffer
  - yawl:ResourcingAllocate
  - yawl:ResourcingSet
  - yawl:ResourcingPrivileges

SensitiveProperties:
  - yawl:participant      # Human resource identifiers
  - yawl:role             # Organizational role assignments
  - yawl:familiarParticipant  # Relationship data

ThreatModel:
  - Unauthorized access to resource allocation reveals organizational structure
  - Privilege escalation via resourcing rule tampering
  - Separation of Duties (SOD) bypass

Controls:
  - Restrict yawl:Resourcing class to resource-manager role
  - Audit all changes to yawl:ResourcingPrivileges
  - Enforce SOD rules at class-level permissions
```

**Category 2: Workflow Metadata (HIGH)**

```yaml
SensitiveProperties:
  - yawl:creator          # User identity
  - yawl:contributor      # Collaborator identities
  - yawl:created          # Temporal information
  - yawl:validFrom        # Business timing
  - yawl:validUntil       # Expiration data

ThreatModel:
  - Metadata exposure reveals workflow ownership
  - Temporal data enables timing attacks
  - Creator information aids social engineering

Controls:
  - Hide yawl:creator from non-owners
  - Redact Dublin Core metadata for readers
  - Property-level ACL on yawl:Metadata class
```

**Category 3: Service Endpoints (HIGH)**

```yaml
SensitiveClasses:
  - yawl:WebServiceGateway
  - yawl:YAWLService

SensitiveProperties:
  - yawl:wsdlLocation     # Service endpoint URLs
  - yawl:operationName    # API operation names
  - yawl:codelet          # Custom code references

ThreatModel:
  - Service endpoint exposure enables reconnaissance
  - WSDL location reveals infrastructure topology
  - Codelet names leak implementation details

Controls:
  - Restrict yawl:WebServiceGateway to system-admin
  - Encrypt yawl:wsdlLocation in storage
  - Property-level deny for non-admin users
```

**Category 4: Variable Mappings (MEDIUM)**

```yaml
SensitiveClasses:
  - yawl:VarMapping
  - yawl:Expression

SensitiveProperties:
  - yawl:query            # XQuery expressions (may contain business logic)
  - yawl:initialValue     # Default data values

ThreatModel:
  - XQuery expressions reveal business rules
  - Initial values may contain sensitive defaults
  - Expression injection via tampering

Controls:
  - Validate all yawl:query expressions for injection
  - Restrict write access to yawl:Expression class
  - Audit all expression modifications
```

### 5.2 Data Redaction Strategy

**Property-Level Redaction:**

```sparql
# Query with redaction
SELECT ?spec ?title ?creator
WHERE {
    GRAPH ?g {
        ?spec a yawl:Specification ;
              yawl:hasMetadata [
                  yawl:title ?title ;
                  yawl:creator ?creator_raw
              ] .
    }

    # Redact creator if user is not owner
    BIND(
        IF(EXISTS {
            GRAPH <http://knhk.io/system/acl> {
                ?acl a :GraphPermission ;
                    :graph ?g ;
                    :grantee "alice@finance.com" ;
                    :role "owner" .
            }
        }, ?creator_raw, "[REDACTED]")
        AS ?creator
    )
}
```

**Triple-Level Masking:**

```rust
pub fn apply_masking(triple: &Triple, user: &User) -> Triple {
    let mut masked = triple.clone();

    // Mask sensitive properties
    if triple.predicate == "http://www.yawlfoundation.org/yawlschema#creator" {
        if !user.has_permission(Permission::ViewCreator) {
            masked.object = Literal::new("[REDACTED]");
        }
    }

    // Mask service endpoints
    if triple.predicate == "http://www.yawlfoundation.org/yawlschema#wsdlLocation" {
        if !user.has_role("system-admin") {
            masked.object = Literal::new("[RESTRICTED]");
        }
    }

    masked
}
```

---

## 6. Privilege Escalation Prevention

### 6.1 Delegation Model

**Controlled Privilege Delegation:**

```turtle
<http://knhk.io/system/acl> {
    # Alice (owner) delegates READ permission to Bob
    :delegation-1 a :PermissionDelegation ;
        :delegator "alice@finance.com" ;
        :delegatee "bob@finance.com" ;
        :graph <http://knhk.io/workflows/spec-a> ;
        :permissions [ :read true ] ;
        :expiry "2025-12-31T23:59:59Z"^^xsd:dateTime ;
        :revocable true .

    # Constraint: Cannot delegate more permissions than delegator has
    # Constraint: Cannot delegate GRANT permission (prevents transitive delegation)
}
```

**Delegation Validation Rules:**

```python
def validate_delegation(delegator: User, delegatee: User, permission: Permission) -> bool:
    """Prevent privilege escalation via delegation."""

    # Rule 1: Delegator must have the permission they're delegating
    if not delegator.has_permission(permission):
        raise PermissionError("Cannot delegate permission you don't have")

    # Rule 2: Cannot delegate GRANT permission (prevents transitive escalation)
    if Action.Grant in permission.actions:
        raise PermissionError("Cannot delegate GRANT permission")

    # Rule 3: Cannot delegate to users in different organizations (unless admin)
    if delegator.organization != delegatee.organization:
        if not delegator.has_role("system-admin"):
            raise PermissionError("Cannot delegate across organizations")

    # Rule 4: Delegation inherits delegator's maximum permissions (no escalation)
    max_permissions = get_max_permissions(delegator, permission.subject)
    if permission.actions.difference(max_permissions.actions):
        raise PermissionError("Cannot delegate more permissions than you have")

    return True
```

### 6.2 Permission Boundary Enforcement

**Maximum Permission Sets:**

```rust
pub struct PermissionBoundary {
    pub role: String,
    pub max_permissions: Vec<Permission>,
    pub deny_overrides: Vec<Permission>,
}

impl PermissionBoundary {
    pub fn enforce(&self, requested: &Permission) -> Result<Permission, PermissionError> {
        // Check deny overrides first (e.g., never allow DELETE on system graphs)
        for deny in &self.deny_overrides {
            if deny.matches(requested) {
                return Err(PermissionError::DenyOverride(deny.clone()));
            }
        }

        // Check against maximum permissions
        for max_perm in &self.max_permissions {
            if max_perm.subject == requested.subject {
                let allowed_actions: HashSet<_> = requested.actions.iter()
                    .filter(|a| max_perm.actions.contains(a))
                    .cloned()
                    .collect();

                if allowed_actions.len() < requested.actions.len() {
                    return Err(PermissionError::ExceedsMaximum);
                }
            }
        }

        Ok(requested.clone())
    }
}

// Example: workflow-architect role boundary
let boundary = PermissionBoundary {
    role: "workflow-architect".to_string(),
    max_permissions: vec![
        Permission {
            subject: PermissionSubject::Class("yawl:Specification".to_string()),
            actions: vec![Action::Read, Action::Write, Action::Delete],
            effect: Effect::Allow,
            conditions: vec![],
        },
    ],
    deny_overrides: vec![
        Permission {
            subject: PermissionSubject::Graph("http://knhk.io/system/*".to_string()),
            actions: vec![Action::Delete],
            effect: Effect::Deny,
            conditions: vec![],
        },
    ],
};
```

---

## 7. Performance Considerations

### 7.1 Permission Caching

**Cache Strategy:**

```rust
pub struct PermissionCache {
    cache: Arc<RwLock<LruCache<(String, String, String), CachedPermission>>>,
    ttl: Duration,
}

#[derive(Clone)]
pub struct CachedPermission {
    pub allowed: bool,
    pub cached_at: Instant,
    pub reason: String,
}

impl PermissionCache {
    pub fn check(&self, user: &str, action: &str, resource: &str) -> Option<bool> {
        let cache = self.cache.read().unwrap();
        if let Some(cached) = cache.get(&(user.to_string(), action.to_string(), resource.to_string())) {
            if cached.cached_at.elapsed() < self.ttl {
                return Some(cached.allowed);
            }
        }
        None
    }

    pub fn store(&self, user: &str, action: &str, resource: &str, allowed: bool, reason: String) {
        let mut cache = self.cache.write().unwrap();
        cache.put(
            (user.to_string(), action.to_string(), resource.to_string()),
            CachedPermission {
                allowed,
                cached_at: Instant::now(),
                reason,
            },
        );
    }

    pub fn invalidate_user(&self, user: &str) {
        let mut cache = self.cache.write().unwrap();
        cache.clear();  // Simple approach: clear all on role change
    }
}
```

**Cache Invalidation Events:**

```yaml
CacheInvalidation:
  - event: user_role_changed
    action: invalidate_user_cache(user_id)

  - event: permission_granted
    action: invalidate_user_cache(grantee_id)

  - event: permission_revoked
    action: invalidate_user_cache(grantee_id)

  - event: acl_rule_modified
    action: clear_all_cache()  # Conservative approach

  - event: user_logout
    action: invalidate_user_cache(user_id)
```

### 7.2 Query Optimization

**Pre-Computed Permission Views:**

```sparql
# Create materialized view of user permissions
CREATE VIEW user_accessible_graphs AS
SELECT ?user ?graph
WHERE {
    GRAPH <http://knhk.io/system/acl> {
        ?acl a :GraphPermission ;
            :grantee ?user ;
            :graph ?graph ;
            :permissions [ :read true ] .
    }
}
UNION
SELECT ?user ?graph
WHERE {
    GRAPH <http://knhk.io/system/acl> {
        ?user_role a :UserRole ;
            :user ?user ;
            :role ?role .
        ?role_perm a :RolePermission ;
            :role ?role ;
            :graph ?graph ;
            :permissions [ :read true ] .
    }
}

# Use materialized view in queries (faster than JOIN)
SELECT ?spec ?title
WHERE {
    GRAPH ?g {
        ?spec a yawl:Specification ;
              yawl:hasMetadata [ yawl:title ?title ] .
    }

    # Fast lookup in materialized view
    FILTER EXISTS {
        VALUES (?user ?accessible_graph) { (<alice@finance.com> ?g) }
        GRAPH <http://knhk.io/system/views> {
            <alice@finance.com> :canAccessGraph ?g .
        }
    }
}
```

---

## 8. Implementation Roadmap

### Phase 1: Foundation (Weeks 1-2)
- [ ] Define ACL graph schema
- [ ] Implement Permission and SecurityContext structs
- [ ] Build permission evaluation algorithm
- [ ] Create user-role-permission mapping

### Phase 2: SPARQL Integration (Weeks 3-4)
- [ ] Implement query rewriter for SELECT queries
- [ ] Add INSERT/UPDATE/DELETE filtering
- [ ] Build permission cache
- [ ] Add audit logging to all queries

### Phase 3: RBAC (Weeks 5-6)
- [ ] Define system roles (admin, architect, auditor)
- [ ] Define workflow roles (owner, editor, reader)
- [ ] Implement role hierarchy
- [ ] Build delegation model

### Phase 4: Advanced Features (Weeks 7-8)
- [ ] Triple-level permissions
- [ ] Property-level redaction
- [ ] Attribute-based access control (ABAC)
- [ ] Lockchain integration for audit

### Phase 5: Testing & Optimization (Weeks 9-10)
- [ ] Performance benchmarking
- [ ] Security penetration testing
- [ ] Query optimization
- [ ] Documentation

---

## 9. Testing & Validation

### 9.1 Security Test Cases

```yaml
TestSuites:
  - name: "Graph-Level Access Control"
    tests:
      - test: "Owner can read their specification"
        setup: "alice owns spec-a"
        query: "SELECT * WHERE { GRAPH <spec-a> { ?s ?p ?o } }"
        user: "alice"
        expected: "SUCCESS with results"

      - test: "Non-owner cannot read specification"
        setup: "bob does not have permission on spec-a"
        query: "SELECT * WHERE { GRAPH <spec-a> { ?s ?p ?o } }"
        user: "bob"
        expected: "SUCCESS with empty results (silent failure)"

      - test: "Reader can read but not write"
        setup: "bob has READ permission on spec-a"
        query: "INSERT DATA { GRAPH <spec-a> { :task a yawl:Task } }"
        user: "bob"
        expected: "ERROR: PermissionDenied (WRITE required)"

  - name: "Class-Level Permissions"
    tests:
      - test: "Resource manager can access Resourcing class"
        setup: "alice has resource-manager role"
        query: "SELECT * WHERE { ?r a yawl:Resourcing }"
        user: "alice"
        expected: "SUCCESS with Resourcing instances"

      - test: "Regular user cannot access Resourcing class"
        setup: "bob has no special roles"
        query: "SELECT * WHERE { ?r a yawl:Resourcing }"
        user: "bob"
        expected: "SUCCESS with empty results"

  - name: "Property-Level Redaction"
    tests:
      - test: "Owner sees creator metadata"
        setup: "alice owns spec-a with creator=alice"
        query: "SELECT ?creator WHERE { ?s yawl:creator ?creator }"
        user: "alice"
        expected: "SUCCESS with creator=alice@finance.com"

      - test: "Reader sees redacted creator"
        setup: "bob has READ on spec-a (not owner)"
        query: "SELECT ?creator WHERE { ?s yawl:creator ?creator }"
        user: "bob"
        expected: "SUCCESS with creator=[REDACTED]"

  - name: "Privilege Escalation Prevention"
    tests:
      - test: "Cannot delegate GRANT permission"
        setup: "alice has GRANT on spec-a"
        action: "alice.delegate(bob, Permission(GRANT))"
        expected: "ERROR: Cannot delegate GRANT permission"

      - test: "Cannot delegate more than you have"
        setup: "alice has READ on spec-a (not WRITE)"
        action: "alice.delegate(bob, Permission(WRITE))"
        expected: "ERROR: Cannot delegate permission you don't have"
```

### 9.2 Performance Benchmarks

```yaml
PerformanceBenchmarks:
  - name: "Permission Check Latency"
    target: "< 5ms for cached, < 50ms for uncached"
    test: "Measure time from request to permission decision"

  - name: "Query Rewrite Overhead"
    target: "< 10% query execution time overhead"
    test: "Compare original vs. rewritten query execution time"

  - name: "Cache Hit Rate"
    target: "> 95% for repeated queries"
    test: "Measure permission cache hit rate over 1000 queries"

  - name: "Concurrent User Scalability"
    target: "> 100 concurrent users with <100ms p99 latency"
    test: "Simulate 100 users querying different graphs"
```

---

## 10. Appendix

### 10.1 Permission Schema (Turtle)

```turtle
@prefix : <http://knhk.io/security#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

# Permission Classes
:Permission a rdfs:Class ;
    rdfs:label "Permission" ;
    rdfs:comment "Base class for all permission types" .

:GraphPermission rdfs:subClassOf :Permission ;
    rdfs:label "Graph Permission" ;
    rdfs:comment "Permission for entire named graph" .

:ClassPermission rdfs:subClassOf :Permission ;
    rdfs:label "Class Permission" ;
    rdfs:comment "Permission for RDF class instances" .

:PropertyPermission rdfs:subClassOf :Permission ;
    rdfs:label "Property Permission" ;
    rdfs:comment "Permission for specific RDF properties" .

:TriplePermission rdfs:subClassOf :Permission ;
    rdfs:label "Triple Permission" ;
    rdfs:comment "Permission for individual RDF triples" .

# Permission Properties
:grantee a rdf:Property ;
    rdfs:domain :Permission ;
    rdfs:range xsd:string ;
    rdfs:comment "User or role receiving permission" .

:graph a rdf:Property ;
    rdfs:domain :Permission ;
    rdfs:range xsd:anyURI ;
    rdfs:comment "Target named graph" .

:class a rdf:Property ;
    rdfs:domain :ClassPermission ;
    rdfs:range xsd:anyURI ;
    rdfs:comment "Target RDF class" .

:property a rdf:Property ;
    rdfs:domain :PropertyPermission ;
    rdfs:range xsd:anyURI ;
    rdfs:comment "Target RDF property" .

:permissions a rdf:Property ;
    rdfs:domain :Permission ;
    rdfs:range :PermissionSet ;
    rdfs:comment "Set of allowed actions" .

# Permission Set Structure
:PermissionSet a rdfs:Class .

:read a rdf:Property ;
    rdfs:domain :PermissionSet ;
    rdfs:range xsd:boolean .

:write a rdf:Property ;
    rdfs:domain :PermissionSet ;
    rdfs:range xsd:boolean .

:delete a rdf:Property ;
    rdfs:domain :PermissionSet ;
    rdfs:range xsd:boolean .

:grant a rdf:Property ;
    rdfs:domain :PermissionSet ;
    rdfs:range xsd:boolean .
```

### 10.2 References

1. **W3C Security and Privacy Group**: RDF Dataset Access Control
2. **SPARQL 1.1 Query Language**: https://www.w3.org/TR/sparql11-query/
3. **knhk Authentication Service**: (internal documentation)
4. **YAWL 4.0 Specification**: http://www.yawlfoundation.org/
5. **Lockchain Audit Trail**: (knhk internal, see ADR-002)

---

**Document Status:** Implementation-Ready
**Next Steps:** Begin Phase 1 implementation (ACL graph schema)
**Security Review Required:** Yes (before production deployment)
