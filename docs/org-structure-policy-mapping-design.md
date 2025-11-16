# Organizational Structure to Policy Constraint Mapping: Design Analysis

**Version**: 1.0
**Date**: 2025-11-16
**Purpose**: Design analysis for converting organizational hierarchies into dynamic policy constraint fields for autonomous ontology systems

---

## Executive Summary

This document analyzes the design patterns, mapping rules, and implementation strategies for converting organizational structures (org charts) into machine-enforceable policy constraints within an autonomous ontology system. The core challenge: **as organizations evolve, policies must automatically adapt while maintaining consistency, detecting conflicts, and preventing security violations.**

The system must handle:
- Real-time org structure changes (hires, promotions, departures, restructuring)
- Automatic policy constraint generation and validation
- Temporal validity (time-windowed constraints)
- Conflict detection and resolution
- Guard profile generation for protective boundaries
- Integration with the autonomous MapEKCoordinator workflow

---

## 1. Organizational Structure Representation

### 1.1 Core Representation Model: Directed Acyclic Graph (DAG) with Temporal Extensions

**Primary Structure**: Directed Acyclic Graph (DAG)
- **Nodes**: Represent entities (Person, Role, Department, Team)
- **Edges**: Represent relationships (reports-to, member-of, manages, delegates-to)
- **Attributes**: Node and edge properties (clearance, location, validity period)

**Why DAG?**
- Captures hierarchical reporting structures naturally
- Supports multiple reporting relationships (matrix organizations)
- Prevents circular dependencies in approval chains
- Enables graph traversal algorithms for chain construction

**Why NOT just a tree?**
- Real organizations have matrix reporting (engineer reports to both Product Manager AND Tech Lead)
- Employees can have multiple roles simultaneously
- Cross-functional teams violate strict tree structure
- Delegation creates temporary alternate paths

### 1.2 Node Types and Attributes

#### Person Node
```turtle
org:Person a rdfs:Class ;
    rdfs:label "Individual in organization" ;
    org:properties [
        org:employeeId xsd:string ;
        org:name xsd:string ;
        org:email xsd:string ;
        org:clearanceLevel org:ClearanceLevel ;
        org:location org:Location ;
        org:startDate xsd:dateTime ;
        org:endDate xsd:dateTime? ;  # null if active
        org:status org:EmployeeStatus ;  # Active, OnLeave, Departed
    ] .
```

**Attributes**:
- `employeeId`: Unique identifier
- `clearanceLevel`: Public | Internal | Confidential | Secret | TopSecret
- `location`: Geographic location (affects data residency policies)
- `startDate/endDate`: Employment validity window
- `status`: Active | OnLeave | Departed

#### Role Node
```turtle
org:Role a rdfs:Class ;
    rdfs:label "Job function or responsibility" ;
    org:properties [
        org:roleId xsd:string ;
        org:title xsd:string ;
        org:level org:RoleLevel ;  # IC1-IC6, M1-M5, VP, SVP, C-Level
        org:function org:Function ;  # Engineering, Finance, Legal, HR
        org:approvalAuthority xsd:decimal ;  # dollar amount
        org:requiredSkills org:SkillSet ;
    ] .
```

**Attributes**:
- `level`: Career ladder level (Individual Contributor vs Manager vs Executive)
- `function`: Departmental function area
- `approvalAuthority`: Maximum spending authority without escalation

#### Department Node
```turtle
org:Department a rdfs:Class ;
    rdfs:label "Organizational unit" ;
    org:properties [
        org:deptId xsd:string ;
        org:name xsd:string ;
        org:costCenter xsd:string ;
        org:budgetAuthority xsd:decimal ;
        org:headOfDepartment org:Person ;
    ] .
```

#### Team Node
```turtle
org:Team a rdfs:Class ;
    rdfs:label "Project or functional team" ;
    org:properties [
        org:teamId xsd:string ;
        org:name xsd:string ;
        org:type org:TeamType ;  # Permanent, Project, CrossFunctional
        org:startDate xsd:dateTime ;
        org:endDate xsd:dateTime? ;  # null for permanent teams
        org:lead org:Person ;
    ] .
```

### 1.3 Edge Types and Relationships

#### Reports-To (Hierarchical Reporting)
```turtle
org:reportsTo a rdf:Property ;
    rdfs:domain org:Person ;
    rdfs:range org:Person ;
    org:properties [
        org:reportType org:ReportType ;  # Direct, Dotted, Functional
        org:startDate xsd:dateTime ;
        org:endDate xsd:dateTime? ;
        org:weight xsd:decimal ;  # for matrix: 0.7 direct, 0.3 dotted
    ] .
```

**Report Types**:
- **Direct**: Primary manager (solid line in org chart)
- **Dotted**: Secondary manager (dashed line, matrix reporting)
- **Functional**: Specialist reporting for specific domain

#### Has-Role (Person ↔ Role Assignment)
```turtle
org:hasRole a rdf:Property ;
    rdfs:domain org:Person ;
    rdfs:range org:Role ;
    org:properties [
        org:startDate xsd:dateTime ;
        org:endDate xsd:dateTime? ;
        org:isPrimary xsd:boolean ;  # primary vs secondary role
    ] .
```

#### Member-Of (Person ↔ Team/Department)
```turtle
org:memberOf a rdf:Property ;
    rdfs:domain org:Person ;
    rdfs:range [org:Team, org:Department] ;
    org:properties [
        org:startDate xsd:dateTime ;
        org:endDate xsd:dateTime? ;
    ] .
```

#### Delegates-To (Temporary Authority Transfer)
```turtle
org:delegatesTo a rdf:Property ;
    rdfs:domain org:Person ;
    rdfs:range org:Person ;
    org:properties [
        org:scope org:DelegationScope ;  # FullAuthority, ApprovalOnly, ReadOnly
        org:startDate xsd:dateTime ;
        org:endDate xsd:dateTime ;  # MUST be time-bounded
        org:reason xsd:string ;  # "Sabbatical", "Coverage", "Project"
    ] .
```

### 1.4 Temporal Aspects

**Core Principle**: Every org structure element has a validity window `[startDate, endDate)`.

**Temporal Queries**:
- "Who was Alice's manager on 2024-03-15?" → Traverse `reportsTo` edges valid at that timestamp
- "What roles did Bob hold during Q1 2024?" → Filter `hasRole` edges by date range
- "Who had Finance Director authority during audit period?" → Time-windowed role query

**Temporal Constraints**:
- Policy constraints inherit validity from org structure they derive from
- If Alice is Finance Director from 2024-01-01 to 2024-06-30, her approval authority constraint is valid for same period
- When org structure changes, old constraints are deprecated (endDate set), new constraints created

### 1.5 Dynamic Aspects

#### Temporary Assignments
```turtle
org:TemporaryAssignment a rdfs:Class ;
    org:properties [
        org:person org:Person ;
        org:role org:Role ;
        org:duration org:Duration ;  # "3 months", "until project completion"
        org:coverage org:Person? ;  # who covers their regular role
    ] .
```

**Example**: Engineer Alice temporarily assigned to Product Manager role for 3 months while PM is on leave.

**Policy Impact**: Alice gains PM approval authority temporarily, old role constraints suspended.

#### Project Teams (Cross-Functional)
```turtle
org:ProjectTeam a org:Team ;
    org:type "CrossFunctional" ;
    org:members [
        org:engineerFromTeamA,
        org:designerFromTeamB,
        org:pmFromTeamC
    ] ;
    org:resourceAccess org:ProjectResources ;  # special resource access for project
.
```

**Policy Impact**: Project team members may need access to resources outside their department. Policy system must create time-limited access grants.

#### Matrix Organizations
```turtle
# Alice reports to both Tech Lead (70%) and Product Manager (30%)
org:Alice org:reportsTo org:TechLead ;
    org:reportType "Direct" ;
    org:weight 0.7 .

org:Alice org:reportsTo org:ProductManager ;
    org:reportType "Dotted" ;
    org:weight 0.3 .
```

**Policy Impact**: Approval chains may require BOTH managers to approve certain actions. Weight determines primary approval authority.

---

## 2. Policy Field Mapping: Org Structure → Constraint Fields

### 2.1 Mapping Schema

Each org structure property maps to one or more policy constraint fields. Mappings can be:
- **Direct**: Single property → single constraint field
- **Derived**: Multiple properties combined → constraint field
- **Conditional**: Mapping depends on context (role level, department, etc.)

### 2.2 Core Mapping Rules

#### Mapping 1: CEO → Override Authority

**Org Property**: `Person.role = "CEO"`

**Policy Field**: `can_override_approval_limits = true`

**Mapping Logic**:
```sparql
# SPARQL query to extract CEO
SELECT ?person
WHERE {
    ?person org:hasRole ?role .
    ?role org:title "Chief Executive Officer" .
    FILTER NOT EXISTS { ?person org:endDate ?end . FILTER(?end < NOW()) }
}
```

**Generated Constraint**:
```yaml
constraint:
  subject: "CEO"
  field: "can_override_approval_limits"
  value: true
  validFrom: <CEO.startDate>
  validUntil: <CEO.endDate>
  rationale: "C-level executives have universal override authority"
```

**Change Propagation**:
- If CEO changes (new CEO appointed), old constraint gets `validUntil = <departure_date>`, new constraint created for new CEO
- Immediate activation: new CEO has override authority from `startDate`

**Temporal Validity**: Yes - constraint is valid only while person holds CEO role

**Conflict Resolution**: Only one CEO at a time (business rule enforced at org structure level)

---

#### Mapping 2: Role → Required Approvals by Role

**Org Property**: `Person.role.level`, `Person.role.function`

**Policy Field**: `required_approvals_by_role[role] = [list of approver roles]`

**Mapping Logic**:
```python
def generate_approval_requirements(role):
    if role.level in ["IC1", "IC2", "IC3"]:  # Individual contributors
        return {
            "approval_chain": [role.manager, role.department_head],
            "threshold": 5000  # $5K spending limit
        }
    elif role.level in ["M1", "M2"]:  # Managers
        return {
            "approval_chain": [role.department_head, role.functional_vp],
            "threshold": 50000  # $50K
        }
    elif role.level == "VP":
        return {
            "approval_chain": [role.functional_svp],
            "threshold": 500000  # $500K
        }
    elif role.level == "SVP":
        return {
            "approval_chain": [role.cfo],
            "threshold": 5000000  # $5M
        }
    elif role.level == "C-Level":
        return {
            "approval_chain": [],  # No higher approval needed
            "threshold": float('inf')
        }
```

**Generated Constraint**:
```yaml
constraint:
  subject: "Finance Manager"
  field: "spending_approval_limit"
  value: 50000
  required_approvals:
    - "Finance Director"
    - "CFO"
  validFrom: <role_assignment_date>
```

**Change Propagation**:
- If person promoted (IC3 → M1), old constraint deprecated, new constraint with higher threshold created
- If manager changes, approval chain updated to point to new manager

**Conflict Resolution**: If person has multiple roles, use **highest authority** level (maximal permissions).

---

#### Mapping 3: Department → Segregation of Duties

**Org Property**: `Department.function`

**Policy Field**: `segregation_of_duties[dept_A, dept_B] = incompatible`

**Mapping Logic**:
```python
# Segregation of Duties Rules (SOD Matrix)
SOD_MATRIX = {
    ("Finance", "Accounting"): "INCOMPATIBLE",  # Can't approve AND audit
    ("Engineering", "Security"): "REQUIRES_REVIEW",  # Code requires security review
    ("Sales", "Finance"): "REQUIRES_SEPARATION",  # Sales can't approve own commissions
    ("HR", "Payroll"): "COMPATIBLE_WITH_CONTROLS",  # Same person OK with audit trail
}

def generate_sod_constraints(person):
    departments = person.memberOf(Department)
    roles = person.hasRole()

    conflicts = []
    for dept_a, dept_b in itertools.combinations(departments, 2):
        if SOD_MATRIX.get((dept_a.function, dept_b.function)) == "INCOMPATIBLE":
            conflicts.append({
                "type": "VIOLATION",
                "departments": [dept_a, dept_b],
                "person": person,
                "remediation": "BLOCK_ASSIGNMENT"
            })

    return conflicts
```

**Generated Constraint**:
```yaml
constraint:
  type: "segregation_of_duties"
  rule: "NO_SINGLE_PERSON_IN_INCOMPATIBLE_DEPTS"
  incompatible_pairs:
    - ["Finance Approver", "Finance Auditor"]
    - ["Sales Executive", "Revenue Accountant"]
    - ["Engineering", "Security Auditor"]
  violation_action: "BLOCK"
```

**Change Propagation**:
- If org restructure merges Finance + Accounting into "Finance Operations", SOD rule becomes obsolete
- System detects: previous two departments no longer exist → deprecate constraint

**Conflict Detection**: If new hire assigned to conflicting departments, **block assignment** and require manual override.

---

#### Mapping 4: Clearance Level → Resource Access Limits

**Org Property**: `Person.clearanceLevel`

**Policy Field**: `resource_access_limits[clearance_level] = [allowed_resources]`

**Mapping Logic**:
```python
CLEARANCE_HIERARCHY = {
    "TopSecret": ["TopSecret", "Secret", "Confidential", "Internal", "Public"],
    "Secret": ["Secret", "Confidential", "Internal", "Public"],
    "Confidential": ["Confidential", "Internal", "Public"],
    "Internal": ["Internal", "Public"],
    "Public": ["Public"]
}

def generate_access_constraints(person):
    clearance = person.clearanceLevel
    allowed_classifications = CLEARANCE_HIERARCHY[clearance]

    return {
        "subject": person,
        "field": "data_access_filter",
        "allowed_classifications": allowed_classifications,
        "denied_classifications": set(ALL_CLASSIFICATIONS) - set(allowed_classifications)
    }
```

**Generated Constraint**:
```yaml
constraint:
  subject: "Alice (Clearance: Confidential)"
  field: "data_access_classification"
  allowed: ["Confidential", "Internal", "Public"]
  denied: ["TopSecret", "Secret"]
  validFrom: <clearance_grant_date>
  reviewDate: <annual_review_date>
```

**Change Propagation**:
- If Alice's clearance downgraded (Secret → Confidential), constraint updated immediately
- **Critical**: System must detect if Alice currently has open documents classified "Secret" → force close or revoke access

**Temporal Validity**: Clearances expire annually → constraint has `reviewDate`, auto-deprecated if not renewed

---

#### Mapping 5: Geographic Location → Data Residency Constraints

**Org Property**: `Person.location`, `Department.location`

**Policy Field**: `data_residency_constraints[location] = [allowed_data_regions]`

**Mapping Logic**:
```python
DATA_RESIDENCY_RULES = {
    "EU": {
        "allowed_data_regions": ["EU", "UK"],  # GDPR compliance
        "prohibited_regions": ["US", "APAC"],
        "requires_consent": ["US"]  # US transfer requires explicit consent
    },
    "US": {
        "allowed_data_regions": ["US", "Global"],
        "prohibited_regions": []
    },
    "China": {
        "allowed_data_regions": ["China"],  # Data must stay in China
        "prohibited_regions": ["US", "EU", "APAC"]
    }
}

def generate_residency_constraints(person):
    location = person.location
    rules = DATA_RESIDENCY_RULES[location.region]

    return {
        "subject": person,
        "field": "data_access_region",
        "allowed_regions": rules["allowed_data_regions"],
        "prohibited_regions": rules["prohibited_regions"]
    }
```

**Generated Constraint**:
```yaml
constraint:
  subject: "EU Employee"
  field: "data_residency"
  allowed_regions: ["EU", "UK"]
  prohibited_access: ["customer_data_us", "customer_data_apac"]
  compliance: "GDPR"
```

**Change Propagation**:
- If employee transfers from EU office to US office, constraint updated
- **Critical**: System must check if employee currently accessing EU-restricted data → transfer ownership or revoke access

---

#### Mapping 6: Cost Center → Budget Authority Limits

**Org Property**: `Department.costCenter`, `Department.budgetAuthority`

**Policy Field**: `budget_authority_limits[cost_center] = max_amount`

**Mapping Logic**:
```python
def generate_budget_constraints(department):
    return {
        "cost_center": department.costCenter,
        "total_budget": department.budgetAuthority,
        "spending_limits": {
            "department_head": department.budgetAuthority * 0.50,  # 50% of budget
            "managers": department.budgetAuthority * 0.10,  # 10% per manager
            "ics": 5000  # Fixed $5K for ICs
        },
        "requires_approval_above": department.budgetAuthority * 0.75  # CFO approval for >75% of budget
    }
```

**Generated Constraint**:
```yaml
constraint:
  cost_center: "ENG-001"
  total_budget: 10000000  # $10M annual budget
  spending_limits:
    VP_Engineering: 5000000
    Engineering_Managers: 1000000
    Individual_Contributors: 5000
  approval_chain_above_limit: ["CFO", "CEO"]
```

---

### 2.3 Multi-Property Derived Mappings

Some constraints require combining multiple org properties:

#### Example: Approval Authority = f(Role Level, Department Budget, Tenure)

```python
def calculate_approval_authority(person):
    base_authority = ROLE_AUTHORITY[person.role.level]

    # Tenure multiplier (longer tenure = higher trust)
    tenure_years = (NOW - person.startDate).years
    if tenure_years < 1:
        tenure_multiplier = 0.5  # New hires get 50% authority
    elif tenure_years < 3:
        tenure_multiplier = 0.75
    else:
        tenure_multiplier = 1.0

    # Department budget constraint
    dept_budget = person.department.budgetAuthority
    dept_cap = dept_budget * 0.10  # No single person can approve >10% of dept budget

    # Final authority = min(base * tenure, dept_cap)
    final_authority = min(
        base_authority * tenure_multiplier,
        dept_cap
    )

    return {
        "spending_approval_limit": final_authority,
        "validFrom": person.startDate,
        "reviewDate": person.startDate.add(years=1)  # Annual review
    }
```

**Key Insight**: Constraints are **multi-dimensional functions** of org properties, not simple 1:1 mappings.

---

## 3. Role-Based Access Control (RBAC) Adaptation

### 3.1 Role Extraction from Org Chart

**Goal**: Extract a role hierarchy that mirrors organizational hierarchy for RBAC.

**Algorithm**:
```python
def extract_role_hierarchy(org_chart):
    roles = {}

    # Step 1: Collect all unique roles
    for person in org_chart.persons:
        for role in person.hasRole():
            if role.id not in roles:
                roles[role.id] = {
                    "title": role.title,
                    "level": role.level,
                    "function": role.function,
                    "inherits_from": []
                }

    # Step 2: Build inheritance hierarchy
    for role_id, role_data in roles.items():
        # Role inherits permissions from roles at lower levels in same function
        parent_roles = find_parent_roles(role_data["level"], role_data["function"])
        role_data["inherits_from"] = parent_roles

    return roles

def find_parent_roles(level, function):
    """
    M1 (Manager Level 1) inherits from IC3 (Senior IC)
    VP inherits from M2
    SVP inherits from VP
    C-Level inherits from SVP
    """
    level_hierarchy = {
        "IC1": [],
        "IC2": ["IC1"],
        "IC3": ["IC2"],
        "M1": ["IC3"],  # Managers inherit IC permissions
        "M2": ["M1"],
        "VP": ["M2"],
        "SVP": ["VP"],
        "C-Level": ["SVP"]
    }

    parent_levels = level_hierarchy.get(level, [])
    return [f"{function}_{pl}" for pl in parent_levels]
```

**Result**: RBAC role hierarchy
```yaml
roles:
  Engineering_IC1:
    permissions: [read_code, write_code, submit_pr]
    inherits_from: []

  Engineering_IC2:
    permissions: [read_code, write_code, submit_pr, review_pr]
    inherits_from: [Engineering_IC1]

  Engineering_M1:
    permissions: [approve_pr, manage_team, approve_expenses_5k]
    inherits_from: [Engineering_IC3]

  Engineering_VP:
    permissions: [approve_headcount, approve_expenses_500k, override_decisions]
    inherits_from: [Engineering_M2]
```

### 3.2 Mapping Role Hierarchy to Approval Chains

**Approval Chain = Path through org hierarchy**

**Algorithm**:
```python
def construct_approval_chain(requester, request_type, request_amount):
    """
    Approval chain construction based on:
    - Requester's position in org
    - Type of request (expense, code review, data access)
    - Amount/severity of request
    """
    chain = []

    # Step 1: Determine threshold escalation levels
    thresholds = get_approval_thresholds(request_type)

    # Step 2: Walk up org hierarchy until reaching sufficient authority
    current_person = requester
    current_authority = 0

    while current_authority < request_amount:
        # Get next approver in hierarchy
        manager = current_person.reportsTo(primary=True)

        if manager is None:
            # Reached top of org (CEO)
            break

        # Add manager to chain
        chain.append(manager)
        current_authority = manager.role.approvalAuthority
        current_person = manager

    # Step 3: Add functional approvers (e.g., Legal for contracts)
    if request_type == "contract":
        chain.append(get_legal_approver())

    if request_type == "data_access" and requester.clearanceLevel < request.dataClassification:
        chain.append(get_security_approver())

    return chain
```

**Example**:
- **Request**: Engineer Alice requests $75K cloud spend
- **Alice's authority**: $5K
- **Alice's manager (Engineering Manager)**: $50K authority
- **Engineering Director**: $500K authority
- **Approval chain**: [Alice's Manager, Engineering Director]

**Generated Policy**:
```yaml
approval_chain:
  requester: "Alice (IC3)"
  request: "Cloud Spend $75K"
  chain:
    - approver: "Engineering Manager"
      authority: 50000
      status: "pending"
    - approver: "Engineering Director"
      authority: 500000
      status: "pending"
  approval_mode: "sequential"  # Manager approves first, then Director
```

### 3.3 Handling Matrix Reporting

**Problem**: Alice reports to **both** Tech Lead (70%) and Product Manager (30%). Which approval chain?

**Solution**: **Primary approver + secondary notification**

```python
def construct_approval_chain_matrix(requester):
    primary_manager = requester.reportsTo(primary=True)  # weight > 0.5
    secondary_managers = requester.reportsTo(primary=False)  # all dotted line reports

    # Primary manager is in approval chain
    chain = [primary_manager]

    # Secondary managers are notified but don't block approval
    notifications = secondary_managers

    return {
        "approval_chain": chain,
        "notify": notifications,
        "approval_mode": "primary_approves_with_notification"
    }
```

**Alternative for critical actions**: **Require ALL managers to approve**

```python
if request.criticality == "high":
    approval_mode = "all_managers_must_approve"
    chain = [primary_manager] + secondary_managers
```

### 3.4 Handling Delegation

**Scenario**: Engineering Director on sabbatical for 3 months, delegates authority to Deputy Director.

**Delegation Constraint**:
```yaml
delegation:
  from: "Engineering Director"
  to: "Deputy Director"
  scope: "full_approval_authority"
  startDate: "2024-06-01"
  endDate: "2024-08-31"
  reason: "Sabbatical"
```

**Approval Chain Adjustment**:
- During delegation period, approval chains that require Engineering Director → automatically route to Deputy Director
- After delegation ends, chains revert to original Engineering Director

**Implementation**:
```python
def get_active_approver(role, timestamp):
    """
    Returns the person actively holding approval authority for role at given time.
    Accounts for delegations.
    """
    primary_holder = get_person_in_role(role, timestamp)

    # Check if primary holder has active delegation
    delegation = get_active_delegation(primary_holder, timestamp)

    if delegation:
        return delegation.delegateTo  # Return delegate
    else:
        return primary_holder  # Return primary
```

---

## 4. Segregation of Duties (SOD) Generation

### 4.1 SOD Principles

**Segregation of Duties**: No single person should control all phases of a critical process.

**Classic Example (Finance)**:
- **Phase 1**: Initiate transaction (Requester)
- **Phase 2**: Approve transaction (Manager)
- **Phase 3**: Execute transaction (Finance)
- **Phase 4**: Audit transaction (Accounting)

**SOD Rule**: Same person cannot perform Phase 2 AND Phase 4 (can't approve AND audit own approval).

### 4.2 Generating SOD Rules from Org Structure

**Algorithm**:
```python
def generate_sod_rules(org_chart):
    sod_rules = []

    # Rule 1: Finance separation
    finance_approvers = get_roles_by_function("Finance", min_level="M1")
    finance_auditors = get_roles_by_function("Accounting")

    sod_rules.append({
        "rule": "NO_FINANCE_APPROVER_AS_AUDITOR",
        "incompatible_roles": [finance_approvers, finance_auditors],
        "severity": "CRITICAL",
        "violation_action": "BLOCK"
    })

    # Rule 2: Sales commission separation
    sales_executives = get_roles_by_function("Sales")
    revenue_accountants = get_roles_by_function("Revenue Accounting")

    sod_rules.append({
        "rule": "NO_SALES_EXEC_APPROVING_OWN_COMMISSION",
        "incompatible_roles": [sales_executives, revenue_accountants],
        "severity": "HIGH",
        "violation_action": "REQUIRE_APPROVAL"
    })

    # Rule 3: Code review separation (Engineering)
    engineers = get_roles_by_function("Engineering", level="IC")

    sod_rules.append({
        "rule": "NO_SELF_APPROVAL_OF_CODE",
        "constraint": "author != reviewer",
        "severity": "MEDIUM",
        "violation_action": "BLOCK"
    })

    return sod_rules
```

### 4.3 SOD Matrix

**SOD Matrix**: Defines compatible/incompatible role pairs

```yaml
sod_matrix:
  Finance_Approver:
    incompatible_with:
      - Finance_Auditor
      - Revenue_Accountant
    requires_separation: true

  Engineering_IC:
    incompatible_with:
      - Security_Auditor  # Engineers can't audit their own code for security
    requires_review_from:
      - Engineering_Peer  # Peer review required

  HR_Manager:
    incompatible_with:
      - Payroll_Processor  # HR can't process own payroll changes
    exception_with_controls:
      - Small_Company_Exception  # <50 employees, same person OK with audit log
```

### 4.4 Handling Multi-Level Roles

**Problem**: Finance has hierarchy: Finance Analyst < Finance Manager < Finance Director < CFO

**Question**: Can Finance Manager approve transactions that Finance Analyst initiated?

**Answer**: **Yes, IF they're at different levels in approval chain.**

**SOD Rule Refinement**:
```yaml
sod_rule:
  name: "FINANCE_HIERARCHICAL_SEPARATION"
  constraint: |
    IF initiator.role.function == "Finance" AND approver.role.function == "Finance"
    THEN approver.role.level > initiator.role.level
  rationale: "Hierarchical approval within same function is allowed"
```

**Example**:
- ✅ Finance Analyst initiates → Finance Manager approves → OK
- ❌ Finance Manager initiates → Finance Manager approves → VIOLATION
- ❌ Finance Manager initiates → Finance Analyst approves → VIOLATION (lower level can't approve higher)

### 4.5 Handling Temporary Violations

**Scenario**: Company hires first Finance Director. Until second person hired, same person handles both approval AND audit.

**Solution**: **Time-limited exception with compensating controls**

```yaml
sod_exception:
  rule: "NO_FINANCE_APPROVER_AS_AUDITOR"
  exception_granted: true
  valid_until: "2024-12-31"  # Must hire second person by end of year
  compensating_controls:
    - "External audit quarterly"
    - "CEO reviews all transactions >$10K"
    - "Audit log reviewed by Board"
  review_frequency: "monthly"
```

**System tracks**:
- Exception expiration date
- Progress toward remediation (job posting, interviews)
- Compensating controls compliance

---

## 5. Approval Chain Construction

### 5.1 Basic Approval Chain Algorithm

**Goal**: Construct approval chain from requester up through hierarchy until sufficient authority reached.

```python
def construct_approval_chain(requester, request_amount, request_type):
    chain = []
    current_person = requester
    accumulated_authority = 0

    # Walk up reporting hierarchy
    while accumulated_authority < request_amount:
        # Get next manager
        manager = get_primary_manager(current_person)

        if manager is None:
            # Reached top of org (CEO/founder)
            if accumulated_authority < request_amount:
                # Even CEO doesn't have enough authority (e.g., $100M acquisition)
                chain.append({"approver": "Board of Directors", "authority": float('inf')})
            break

        # Add manager to chain
        manager_authority = get_approval_authority(manager, request_type)
        chain.append({
            "approver": manager,
            "role": manager.role.title,
            "authority": manager_authority
        })

        accumulated_authority = max(accumulated_authority, manager_authority)
        current_person = manager

    return {
        "chain": chain,
        "total_authority": accumulated_authority,
        "approval_mode": "sequential"
    }
```

**Example**:
- **Request**: $250K cloud infrastructure purchase
- **Requester**: Engineering IC3 (authority: $5K)
- **Chain**:
  1. Engineering Manager M1 (authority: $50K)
  2. Engineering Director M2 (authority: $500K) ← Sufficient authority
- **Result**: Two-level approval chain

### 5.2 Exception Handling: CEO Override

**Special Rule**: CEO can approve anything, regardless of amount.

```python
def construct_approval_chain_with_ceo_exception(requester, request_amount):
    # Check if requester IS the CEO
    if requester.role.title == "CEO":
        return {
            "chain": [],
            "approval_mode": "self_approved",
            "rationale": "CEO has universal approval authority"
        }

    # Normal chain construction
    chain = construct_approval_chain(requester, request_amount, request_type)

    # CEO can override any step in chain
    chain["override_authority"] = "CEO"

    return chain
```

### 5.3 Bypass Scenarios: Manager Unavailable

**Problem**: Engineering Manager on vacation, approval needed urgently.

**Solutions**:

#### Option 1: Delegation (Pre-planned)
Manager delegates authority to peer before vacation:
```yaml
delegation:
  from: "Engineering Manager A"
  to: "Engineering Manager B"
  scope: "approval_authority"
  duration: "2024-07-01 to 2024-07-14"
```

**Chain adjustment**: Automatically routes to Engineering Manager B during vacation.

#### Option 2: Escalation (Ad-hoc)
Requester escalates to next level:
```python
def handle_unavailable_approver(chain, unavailable_person):
    # Remove unavailable person from chain
    chain = [approver for approver in chain if approver != unavailable_person]

    # Escalate to next higher level
    next_approver = get_next_higher_authority(unavailable_person)
    chain.insert(0, next_approver)

    # Log escalation for audit
    log_escalation(unavailable_person, next_approver, reason="Unavailable")

    return chain
```

#### Option 3: Timeout Auto-Escalation
After 48 hours with no response, automatically escalate:
```yaml
auto_escalation:
  timeout: "48 hours"
  action: "escalate_to_next_level"
  notification:
    - original_approver  # "Your approval request timed out and was escalated"
    - escalated_approver  # "Approval escalated to you due to timeout"
```

### 5.4 Parallel Approvals vs Sequential

**Sequential Approval**: Each approver must approve in order (A → B → C)
- **Use case**: Hierarchical approvals (IC → Manager → Director)
- **Advantage**: Clear chain of command
- **Disadvantage**: Slow (each step waits for previous)

**Parallel Approval**: Multiple approvers must approve, but order doesn't matter (A + B + C, any order)
- **Use case**: Cross-functional reviews (Engineering + Legal + Finance)
- **Advantage**: Faster (no waiting)
- **Disadvantage**: Coordination complexity

**Example**: Contract Review
```yaml
approval_chain:
  request: "Customer Contract $500K"
  mode: "parallel"
  required_approvals:
    - Engineering_VP  # Technical feasibility
    - Finance_Director  # Budget approval
    - Legal_Counsel  # Contract terms
  minimum_approvals: 3  # All must approve
```

**Implementation**:
```python
def construct_parallel_approval_chain(request):
    approvers = []

    # Add functional approvers based on request type
    if request.type == "contract":
        approvers.extend([
            get_functional_head("Engineering"),
            get_functional_head("Finance"),
            get_functional_head("Legal")
        ])

    return {
        "chain": approvers,
        "approval_mode": "parallel",
        "minimum_required": len(approvers),  # All must approve
        "timeout": "7 days"
    }
```

### 5.5 Conditional Approvals

**Scenario**: Large purchase requires Finance approval, UNLESS it's pre-budgeted.

```yaml
conditional_approval:
  request: "Software License $100K"
  conditions:
    - IF request.budgeted == true THEN approval_chain: [Engineering_Director]
    - ELSE approval_chain: [Engineering_Director, Finance_Director, CFO]
```

**Implementation**:
```python
def construct_conditional_chain(request):
    base_chain = [request.requester.manager]

    # Check conditions
    if request.amount > request.requester.department.budget * 0.10:
        # Exceeds 10% of department budget → requires CFO
        base_chain.append(get_cfo())

    if request.budgeted == False:
        # Not pre-budgeted → requires Finance review
        base_chain.append(get_functional_head("Finance"))

    if request.vendor.country in ["China", "Russia"]:
        # Foreign vendor → requires Security review
        base_chain.append(get_functional_head("Security"))

    return base_chain
```

---

## 6. Clearance & Sensitivity Mapping

### 6.1 Clearance Level Hierarchy

**Clearance Levels** (lowest to highest):
1. **Public**: Publicly available information
2. **Internal**: Internal company information (not public)
3. **Confidential**: Sensitive business information (financial data, strategic plans)
4. **Secret**: Highly sensitive (M&A plans, unreleased products)
5. **Top Secret**: Exceptionally sensitive (national security, if applicable)

**Inheritance**: Higher clearance grants access to all lower levels.

### 6.2 Data Classification

All data/resources have a **classification label**:

```yaml
resource:
  id: "customer_financial_records"
  classification: "Confidential"
  owner: "Finance Department"
  required_clearance: "Confidential"
```

### 6.3 Clearance to Data Access Mapping

**Rule**: `Person.clearanceLevel >= Resource.classification` → Access granted

```python
def check_access_authorization(person, resource):
    clearance_hierarchy = {
        "Public": 0,
        "Internal": 1,
        "Confidential": 2,
        "Secret": 3,
        "TopSecret": 4
    }

    person_level = clearance_hierarchy[person.clearanceLevel]
    resource_level = clearance_hierarchy[resource.classification]

    if person_level >= resource_level:
        return {"access": "GRANTED", "reason": "Sufficient clearance"}
    else:
        return {
            "access": "DENIED",
            "reason": f"Requires {resource.classification} clearance, person has {person.clearanceLevel}",
            "action": "REQUEST_EXCEPTION"
        }
```

### 6.4 Dynamic Classification

**Challenge**: Data classification can change over time.

**Example**: Product roadmap is "Secret" before launch, "Internal" after public announcement.

```yaml
resource:
  id: "product_roadmap_2025"
  classification_rules:
    - IF current_date < launch_date THEN "Secret"
    - IF current_date >= launch_date THEN "Internal"
  launch_date: "2025-06-01"
```

**Policy Impact**: People with "Internal" clearance gain access automatically on launch date.

### 6.5 Clearance Downgrade Scenarios

**Scenario**: Alice's clearance downgraded from "Secret" to "Confidential" (left critical project).

**System Actions**:
1. **Immediate**: Revoke access to all "Secret" resources
2. **Force Close**: Any open documents classified "Secret" → force close/lock
3. **Audit**: Log all Alice's recent access to "Secret" resources
4. **Notification**: Notify Alice and her manager of access changes

```python
def handle_clearance_downgrade(person, old_clearance, new_clearance):
    # Find all resources person currently has access to
    current_access = get_accessible_resources(person, old_clearance)
    new_access = get_accessible_resources(person, new_clearance)

    # Revoke access to resources no longer authorized
    revoked_resources = set(current_access) - set(new_access)

    for resource in revoked_resources:
        revoke_access(person, resource)
        log_access_change(person, resource, action="REVOKED", reason="Clearance downgrade")

        # Force close any active sessions
        active_sessions = get_active_sessions(person, resource)
        for session in active_sessions:
            force_close_session(session, reason="Clearance downgraded")

    # Notify stakeholders
    notify(person, f"Your clearance has been downgraded to {new_clearance}. Access to {len(revoked_resources)} resources revoked.")
    notify(person.manager, f"{person.name}'s clearance downgraded. {len(revoked_resources)} resources revoked.")
```

---

## 7. Temporal Aspects

### 7.1 Time-Stamped Org Structure Changes

**Every org structure change is timestamped**:

```yaml
org_event:
  type: "ROLE_ASSIGNMENT"
  person: "Alice"
  role: "Engineering Manager"
  startDate: "2024-06-01T00:00:00Z"
  endDate: null  # null = ongoing
  previousRole: "Senior Engineer"
  previousEndDate: "2024-05-31T23:59:59Z"
```

**Policy constraints inherit timestamps**:

```yaml
constraint:
  subject: "Alice"
  field: "approval_authority"
  value: 50000
  validFrom: "2024-06-01T00:00:00Z"  # Same as role.startDate
  validUntil: null  # Ongoing while in role
  derivedFrom: "Role: Engineering Manager"
```

### 7.2 Policy Constraint Validity Windows

**Time-Windowed Query**: "What was Alice's approval authority on 2024-04-15?"

```sparql
SELECT ?authority
WHERE {
    ?constraint policy:subject "Alice" .
    ?constraint policy:field "approval_authority" .
    ?constraint policy:value ?authority .
    ?constraint policy:validFrom ?start .
    ?constraint policy:validUntil ?end .

    FILTER (?start <= "2024-04-15"^^xsd:date)
    FILTER (!BOUND(?end) || ?end > "2024-04-15"^^xsd:date)
}
```

**Result**: Alice was Senior Engineer (authority: $5K) on 2024-04-15, promoted to Manager (authority: $50K) on 2024-06-01.

### 7.3 New Employee Onboarding: Progressive Authority

**Principle**: New employees have restricted authority during probationary period.

**Policy**:
```yaml
onboarding_policy:
  duration: "90 days"
  constraints:
    - period: "0-30 days"
      approval_authority: 0  # No approval authority
      code_review_authority: false
      data_access: "Public only"

    - period: "31-60 days"
      approval_authority: 1000  # $1K limit
      code_review_authority: true  # Can review peer code
      data_access: "Internal"

    - period: "61-90 days"
      approval_authority: 5000  # $5K limit
      code_review_authority: true
      data_access: "Confidential"

    - period: "91+ days"
      approval_authority: "ROLE_BASED"  # Full role authority
      code_review_authority: "ROLE_BASED"
      data_access: "CLEARANCE_BASED"
```

**Implementation**:
```python
def get_effective_authority(person, timestamp):
    tenure = (timestamp - person.startDate).days

    if tenure < 30:
        return {"approval_authority": 0, "data_access": "Public"}
    elif tenure < 60:
        return {"approval_authority": 1000, "data_access": "Internal"}
    elif tenure < 90:
        return {"approval_authority": 5000, "data_access": "Confidential"}
    else:
        # Full role-based authority
        return get_role_based_authority(person.role)
```

### 7.4 Departing Employee: Immediate Revocation

**Critical**: When employee departs, ALL authority revoked immediately.

```python
def handle_employee_departure(person, departure_date):
    # Set person status to "Departed"
    person.status = "Departed"
    person.endDate = departure_date

    # Revoke all access
    revoke_all_access(person)

    # Deprecate all policy constraints
    constraints = get_active_constraints(person)
    for constraint in constraints:
        constraint.validUntil = departure_date
        constraint.status = "DEPRECATED"

    # Transfer ownership of resources
    owned_resources = get_owned_resources(person)
    for resource in owned_resources:
        transfer_ownership(resource, person.manager, reason="Employee departure")

    # Close all active sessions
    active_sessions = get_active_sessions(person)
    for session in active_sessions:
        force_close_session(session, reason="Employee departed")

    # Audit log
    log_departure(person, departure_date, actions_taken)
```

### 7.5 Leave of Absence: Temporary Suspension

**Scenario**: Manager takes 3-month maternity leave.

**Delegation**:
```yaml
leave_of_absence:
  person: "Alice (Engineering Manager)"
  startDate: "2024-07-01"
  endDate: "2024-09-30"
  status: "OnLeave"
  coverage:
    person: "Bob (Deputy Engineering Manager)"
    delegation_scope: "full_authority"
    validFrom: "2024-07-01"
    validUntil: "2024-09-30"
```

**Policy Impact**:
- Alice's approval authority constraints: **suspended** (validUntil set to leave start date)
- Bob's constraints: **temporary elevation** (time-windowed authority during coverage)
- After Alice returns: Bob's temporary authority **auto-expires**, Alice's authority **reactivates**

```python
def handle_leave_of_absence(person, coverage_person, start_date, end_date):
    # Suspend person's constraints
    constraints = get_active_constraints(person)
    for constraint in constraints:
        create_suspended_constraint(
            constraint,
            suspension_start=start_date,
            suspension_end=end_date,
            reason="Leave of absence"
        )

    # Create temporary constraints for coverage person
    for constraint in constraints:
        create_temporary_constraint(
            subject=coverage_person,
            field=constraint.field,
            value=constraint.value,
            validFrom=start_date,
            validUntil=end_date,
            reason=f"Coverage for {person.name}"
        )

    # On return, auto-reactivate original constraints
    schedule_constraint_reactivation(person, end_date)
```

---

## 8. Conflict Detection in Policies

### 8.1 Types of Conflicts

#### Conflict Type 1: SOD Violation
**Scenario**: New hire assigned to both Finance Approver AND Finance Auditor roles.

**Detection**:
```python
def detect_sod_violation(person):
    roles = person.hasRole()
    sod_matrix = get_sod_matrix()

    for role_a, role_b in itertools.combinations(roles, 2):
        if sod_matrix.are_incompatible(role_a, role_b):
            return {
                "conflict_type": "SOD_VIOLATION",
                "roles": [role_a, role_b],
                "severity": "CRITICAL",
                "action": "BLOCK_ASSIGNMENT"
            }

    return None  # No conflict
```

**Mitigation**:
- **Block assignment**: Prevent assignment to incompatible roles
- **Require exception**: Allow with manual approval from Compliance
- **Remediate**: Remove one of the conflicting roles

---

#### Conflict Type 2: Approval Chain Loop

**Scenario**: Alice reports to Bob, Bob reports to Charlie, Charlie reports to Alice (circular).

**Detection**:
```python
def detect_approval_loop(org_chart):
    # Use cycle detection algorithm (DFS)
    visited = set()
    rec_stack = set()

    def has_cycle(person):
        visited.add(person)
        rec_stack.add(person)

        manager = person.reportsTo(primary=True)
        if manager:
            if manager not in visited:
                if has_cycle(manager):
                    return True
            elif manager in rec_stack:
                return True  # Cycle detected

        rec_stack.remove(person)
        return False

    for person in org_chart.persons:
        if person not in visited:
            if has_cycle(person):
                return {"conflict_type": "APPROVAL_LOOP", "severity": "CRITICAL"}

    return None
```

**Mitigation**:
- **Block org change**: Prevent creating circular reporting structure
- **Detect at time of change**: Validate org structure before committing

---

#### Conflict Type 3: Clearance Downgrade with Active Access

**Scenario**: Alice's clearance downgraded from "Secret" to "Confidential", but she has 5 open "Secret" documents.

**Detection**:
```python
def detect_clearance_access_conflict(person, new_clearance):
    current_clearance = person.clearanceLevel
    accessible_resources = get_accessible_resources(person, current_clearance)
    new_accessible_resources = get_accessible_resources(person, new_clearance)

    lost_access = set(accessible_resources) - set(new_accessible_resources)

    # Check for active sessions on lost resources
    conflicts = []
    for resource in lost_access:
        active_sessions = get_active_sessions(person, resource)
        if active_sessions:
            conflicts.append({
                "resource": resource,
                "active_sessions": len(active_sessions),
                "action_required": "FORCE_CLOSE"
            })

    if conflicts:
        return {
            "conflict_type": "CLEARANCE_ACCESS_CONFLICT",
            "severity": "HIGH",
            "conflicts": conflicts,
            "mitigation": "Force close active sessions on downgraded resources"
        }

    return None
```

**Mitigation**:
- **Force close sessions**: Immediately close all active access to downgraded resources
- **Notify user**: Alert that access is being revoked
- **Audit log**: Record all forced closures

---

#### Conflict Type 4: Budget Authority Exceeds Department Budget

**Scenario**: Engineering Manager has $100K approval authority, but department budget is only $50K.

**Detection**:
```python
def detect_budget_authority_conflict(person):
    approval_authority = person.role.approvalAuthority
    department_budget = person.department.budgetAuthority

    if approval_authority > department_budget:
        return {
            "conflict_type": "AUTHORITY_EXCEEDS_BUDGET",
            "severity": "MEDIUM",
            "person_authority": approval_authority,
            "dept_budget": department_budget,
            "action": "CAP_AUTHORITY_TO_BUDGET"
        }

    return None
```

**Mitigation**:
- **Cap authority**: Automatically limit approval authority to department budget
- **Notify finance**: Alert Finance team to budget discrepancy

---

### 8.2 Conflict Resolution Strategies

**Strategy 1: BLOCK** - Prevent the org change that would create conflict
- Use for: SOD violations, approval loops
- Tradeoff: May prevent legitimate org changes

**Strategy 2: AUTO-REMEDIATE** - Automatically fix the conflict
- Use for: Budget authority caps, clearance downgrades
- Tradeoff: May make unexpected changes

**Strategy 3: REQUIRE MANUAL APPROVAL** - Allow with exception process
- Use for: Temporary SOD violations (small company exception)
- Tradeoff: Slows down org changes

**Strategy 4: NOTIFY AND LOG** - Allow but track as policy exception
- Use for: Low-severity conflicts
- Tradeoff: Creates compliance risk if not monitored

---

### 8.3 Conflict Detection Workflow

```python
def validate_org_change(proposed_change):
    """
    Validate org change before committing.
    Detects conflicts and determines mitigation strategy.
    """
    conflicts = []

    # Apply proposed change to temporary org structure
    temp_org = apply_change_to_temp(org_structure, proposed_change)

    # Run conflict detection algorithms
    conflicts.extend(detect_sod_violations(temp_org))
    conflicts.extend(detect_approval_loops(temp_org))
    conflicts.extend(detect_clearance_conflicts(temp_org))
    conflicts.extend(detect_budget_conflicts(temp_org))

    # Determine overall action
    if any(c["severity"] == "CRITICAL" for c in conflicts):
        return {
            "status": "REJECTED",
            "conflicts": conflicts,
            "action": "BLOCK_CHANGE"
        }
    elif any(c["severity"] == "HIGH" for c in conflicts):
        return {
            "status": "REQUIRES_APPROVAL",
            "conflicts": conflicts,
            "action": "REQUEST_EXCEPTION_FROM_COMPLIANCE"
        }
    else:
        # Medium/low severity - auto-remediate
        return {
            "status": "APPROVED_WITH_REMEDIATION",
            "conflicts": conflicts,
            "action": "APPLY_AUTOMATIC_MITIGATIONS"
        }
```

---

## 9. Guard Profile Generation

### 9.1 What are Guard Profiles?

**Guard Profile**: Machine-readable specification of **what a role/person can and cannot do**.

**Purpose**: Feed into MapEKCoordinator's validation phase to enforce protective boundaries.

**Structure**:
```yaml
guard_profile:
  subject: "Engineering Manager"
  permissions:
    - action: "approve_expense"
      max_amount: 50000
      requires: []  # No additional approvals needed

    - action: "approve_code_review"
      constraints:
        - "author != approver"  # Can't approve own code
        - "approver.seniority >= author.seniority"  # Can't approve more senior engineer

    - action: "access_data"
      allowed_classifications: ["Public", "Internal", "Confidential"]
      denied_classifications: ["Secret", "TopSecret"]

    - action: "create_policy"
      allowed: false  # Only C-level can create policies

  prohibitions:
    - action: "approve_own_expense"
      reason: "Segregation of duties"

    - action: "audit_own_approvals"
      reason: "Conflict of interest"
```

### 9.2 Generating Guard Profiles from Org Structure

**Algorithm**:
```python
def generate_guard_profile(person):
    profile = {
        "subject": person.name,
        "role": person.role.title,
        "permissions": [],
        "prohibitions": []
    }

    # Derive permissions from role
    role_permissions = get_role_permissions(person.role)
    profile["permissions"].extend(role_permissions)

    # Apply clearance-based data access
    clearance_permissions = get_clearance_permissions(person.clearanceLevel)
    profile["permissions"].extend(clearance_permissions)

    # Apply budget authority
    budget_permission = {
        "action": "approve_expense",
        "max_amount": person.role.approvalAuthority,
        "requires": construct_approval_chain(person, person.role.approvalAuthority)
    }
    profile["permissions"].append(budget_permission)

    # Apply SOD-based prohibitions
    sod_prohibitions = get_sod_prohibitions(person)
    profile["prohibitions"].extend(sod_prohibitions)

    # Apply tenure-based restrictions (new employees)
    tenure_restrictions = get_tenure_restrictions(person)
    profile["prohibitions"].extend(tenure_restrictions)

    return profile
```

### 9.3 Role-Specific Guard Profiles

#### CEO Guard Profile
```yaml
guard_profile:
  subject: "CEO"
  permissions:
    - action: "approve_expense"
      max_amount: UNLIMITED

    - action: "override_policy"
      scope: "any"
      rationale: "C-level override authority"

    - action: "create_policy"
      allowed: true

    - action: "access_data"
      allowed_classifications: ["ALL"]  # CEO can access any data

  prohibitions: []  # CEO has no prohibitions (except legal/regulatory)
```

#### CFO Guard Profile
```yaml
guard_profile:
  subject: "CFO"
  permissions:
    - action: "approve_expense"
      max_amount: 5000000  # $5M limit
      above_limit_requires: ["Board of Directors"]

    - action: "approve_budget"
      scope: "all_departments"

    - action: "access_financial_data"
      allowed: true

  prohibitions:
    - action: "audit_own_approvals"
      reason: "Must be audited by external auditor"
```

#### Finance Auditor Guard Profile
```yaml
guard_profile:
  subject: "Finance Auditor"
  permissions:
    - action: "access_financial_data"
      scope: "read_only"  # Auditors can read, not write

    - action: "access_audit_trail"
      scope: "all_transactions"

    - action: "flag_transaction"
      scope: "suspicious_transactions"

  prohibitions:
    - action: "approve_transaction"
      reason: "Auditors cannot approve transactions (SOD)"

    - action: "modify_financial_record"
      reason: "Read-only access for audit integrity"
```

#### Intern Guard Profile
```yaml
guard_profile:
  subject: "Engineering Intern"
  permissions:
    - action: "write_code"
      scope: "non_production"  # Can only commit to dev branches

    - action: "submit_pr"
      requires: ["Mentor approval before submission"]

    - action: "access_data"
      allowed_classifications: ["Public", "Internal"]

  prohibitions:
    - action: "approve_code_review"
      reason: "Insufficient experience"

    - action: "deploy_to_production"
      reason: "Interns cannot deploy"

    - action: "access_customer_data"
      reason: "Requires Confidential clearance (intern has Internal)"
```

### 9.4 Guard Validation in MapEKCoordinator

**Integration Point**: Guards are checked during **Validation** phase.

```python
# MapEKCoordinator validation with guards
def validate_proposed_action(action, actor):
    # Load actor's guard profile
    guard = load_guard_profile(actor)

    # Check permissions
    if not guard.permits(action):
        return {
            "validation": "FAILED",
            "reason": f"Actor {actor} does not have permission for action {action}",
            "guard_violation": guard.get_prohibition(action)
        }

    # Check constraints
    constraints = guard.get_constraints(action)
    for constraint in constraints:
        if not evaluate_constraint(constraint, action, actor):
            return {
                "validation": "FAILED",
                "reason": f"Constraint violated: {constraint}",
                "constraint": constraint
            }

    return {"validation": "PASSED"}
```

**Example**:
- **Action**: Alice (Engineering Manager) tries to approve $100K expense
- **Guard Check**: Alice's guard permits "approve_expense" up to $50K
- **Validation Result**: FAILED - exceeds approval authority
- **Required**: Escalate to Engineering Director

---

## 10. Case Study Scenarios

### 10.1 Scenario A: New Hire

**Event**: Bob joins as Software Engineer on 2024-06-01

**Org Chart Update**:
```yaml
person:
  id: "bob_001"
  name: "Bob Smith"
  role: "Software Engineer IC2"
  department: "Engineering"
  manager: "Alice (Engineering Manager)"
  location: "San Francisco"
  clearanceLevel: "Public"  # Initial clearance
  startDate: "2024-06-01"
```

**Policy Impact**:

1. **Approval Authority**: $0 (0-30 days probation)
```yaml
constraint:
  subject: "Bob Smith"
  field: "approval_authority"
  value: 0
  validFrom: "2024-06-01"
  validUntil: "2024-07-01"  # 30-day probation
  rationale: "New hire probationary period"
```

2. **Code Review Authority**: False (cannot approve code reviews for 30 days)
```yaml
constraint:
  subject: "Bob Smith"
  field: "code_review_approval"
  value: false
  validFrom: "2024-06-01"
  validUntil: "2024-07-01"
```

3. **Data Access**: Public only
```yaml
constraint:
  subject: "Bob Smith"
  field: "data_access_classification"
  allowed: ["Public"]
  validFrom: "2024-06-01"
  validUntil: "2024-09-01"  # Upgrade to Internal after 90 days
```

4. **Guard Profile**:
```yaml
guard_profile:
  subject: "Bob Smith"
  permissions:
    - action: "write_code"
      scope: "feature_branches_only"
    - action: "submit_pr"
      requires: ["Peer review"]

  prohibitions:
    - action: "approve_code_review"
      duration: "30 days"
    - action: "access_customer_data"
      reason: "Insufficient clearance"
```

**Validation in Autonomous System**:
- **Observation**: HR system emits "new_hire_event"
- **Pattern**: Detected as "org_structure_changed"
- **Proposer**: Org-to-policy mapper generates constraints above
- **Validator**: Checks no SOD violations, no conflicts
- **Snapshot**: New constraints promoted to active policy snapshot
- **Guard**: Bob's guard profile activated

---

### 10.2 Scenario B: Promotion

**Event**: Alice promoted from IC3 (Senior Engineer) to M1 (Engineering Manager) on 2024-06-01

**Org Chart Update**:
```yaml
# Old role ends
role_assignment:
  person: "Alice"
  role: "Senior Engineer IC3"
  endDate: "2024-05-31T23:59:59Z"

# New role starts
role_assignment:
  person: "Alice"
  role: "Engineering Manager M1"
  startDate: "2024-06-01T00:00:00Z"
  reportsTo: "Engineering Director"
  manages: ["Bob", "Charlie", "Diana"]  # Now manages team
```

**Policy Impact**:

1. **Clearance Upgraded**: Internal → Confidential
```yaml
constraint:
  subject: "Alice"
  field: "clearance_level"
  old_value: "Internal"
  new_value: "Confidential"
  effective: "2024-06-01"
```

2. **Approval Authority Increased**: $5K → $50K
```yaml
# Old constraint deprecated
constraint:
  subject: "Alice"
  field: "approval_authority"
  value: 5000
  validUntil: "2024-05-31T23:59:59Z"
  status: "DEPRECATED"

# New constraint activated
constraint:
  subject: "Alice"
  field: "approval_authority"
  value: 50000
  validFrom: "2024-06-01T00:00:00Z"
  derivedFrom: "Role: Engineering Manager M1"
```

3. **SOD Constraints Change**: Previous constraints no longer apply (she was part of "individual contributor" SOD group, now in "manager" group)
```yaml
# Old SOD rule: IC cannot approve peer code
sod_constraint:
  subject: "Alice"
  rule: "IC_CANNOT_SELF_APPROVE"
  validUntil: "2024-05-31T23:59:59Z"
  status: "DEPRECATED"

# New SOD rule: Manager cannot audit own team
sod_constraint:
  subject: "Alice"
  rule: "MANAGER_CANNOT_AUDIT_OWN_TEAM"
  validFrom: "2024-06-01T00:00:00Z"
```

4. **New Approval Chain**: Alice is now IN approval chains for her reports
```yaml
approval_chain:
  requester: "Bob (reports to Alice)"
  chain:
    - "Alice (Manager)"  # New approver
    - "Engineering Director"
```

**Validation**:
- **Observation**: Promotion event detected
- **Proposer**: Generate new manager-level constraints
- **Validator**: Check Alice doesn't have conflicting roles, no SOD violations
- **Snapshot**: Old IC constraints deprecated, new Manager constraints activated
- **Guard**: Alice's guard profile updated with manager permissions

---

### 10.3 Scenario C: Manager Leave

**Event**: Finance Director takes 3-month sabbatical (2024-07-01 to 2024-09-30)

**Org Chart Update**:
```yaml
person:
  id: "finance_director"
  name: "Carol (Finance Director)"
  status: "OnLeave"
  leave:
    startDate: "2024-07-01"
    endDate: "2024-09-30"
    coverage:
      person: "David (Deputy Finance Director)"
      scope: "full_authority"
```

**Policy Impact**:

1. **Suspend Carol's Approval Authority**:
```yaml
constraint:
  subject: "Carol"
  field: "approval_authority"
  value: 500000
  suspension:
    start: "2024-07-01"
    end: "2024-09-30"
    reason: "Sabbatical leave"
  status: "SUSPENDED"
```

2. **Activate David's Temporary Authority**:
```yaml
constraint:
  subject: "David (Deputy)"
  field: "approval_authority"
  value: 500000  # Same as Carol's
  validFrom: "2024-07-01"
  validUntil: "2024-09-30"  # Time-limited
  derivedFrom: "Coverage for Carol (Finance Director)"
  temporary: true
```

3. **Redirect Approval Chains**:
```yaml
approval_chain_redirect:
  from: "Carol (Finance Director)"
  to: "David (Deputy Finance Director)"
  validFrom: "2024-07-01"
  validUntil: "2024-09-30"
  reason: "Sabbatical coverage"
```

**Example Approval Chain**:
- **Before Leave** (June 2024): Expense >$50K → Finance Manager → Carol (Finance Director) → CFO
- **During Leave** (August 2024): Expense >$50K → Finance Manager → **David (Deputy)** → CFO
- **After Return** (October 2024): Expense >$50K → Finance Manager → Carol (Finance Director) → CFO

**Validation**:
- **Observation**: Leave of absence event
- **Proposer**: Generate temporary delegation constraints
- **Validator**: Verify David has competency to cover (e.g., minimum level, clearance)
- **Snapshot**: Time-windowed constraints activated
- **Guard**: David's guard profile temporarily elevated

**Auto-Reactivation**:
```python
# Scheduled task on 2024-09-30
def reactivate_after_leave():
    # Expire David's temporary constraints
    expire_constraint("David approval_authority temporary")

    # Reactivate Carol's constraints
    reactivate_constraint("Carol approval_authority")

    # Remove approval chain redirect
    remove_redirect("Carol -> David")

    # Notify stakeholders
    notify("Carol", "Welcome back! Your approval authority has been reactivated.")
    notify("David", "Temporary coverage authority has expired. Thank you!")
```

---

### 10.4 Scenario D: Restructuring

**Event**: Finance and Accounting departments merge into "Finance Operations" on 2024-06-01

**Org Chart Changes**:
```yaml
# Old structure (deprecated)
department:
  id: "finance"
  name: "Finance"
  endDate: "2024-05-31T23:59:59Z"

department:
  id: "accounting"
  name: "Accounting"
  endDate: "2024-05-31T23:59:59Z"

# New structure (activated)
department:
  id: "finance_ops"
  name: "Finance Operations"
  startDate: "2024-06-01"
  headOfDepartment: "Carol (VP Finance Operations)"
  consolidates: ["finance", "accounting"]
```

**Policy Impact**:

1. **SOD Rules Between Finance/Accounting Now Invalid**:

Old rule:
```yaml
sod_rule:
  name: "NO_FINANCE_APPROVER_AS_ACCOUNTING_AUDITOR"
  incompatible_depts: ["Finance", "Accounting"]
  validUntil: "2024-05-31T23:59:59Z"
  status: "DEPRECATED"
  reason: "Departments merged"
```

New reality: Same person CAN be both approver and auditor if they're in same "Finance Operations" dept → **Requires new SOD rule**:
```yaml
sod_rule:
  name: "NO_FINANCE_OPS_SELF_AUDIT"
  constraint: "approver != auditor"  # Same person, not same dept
  scope: "Finance Operations"
  validFrom: "2024-06-01"
```

2. **Approval Chain Reconstruction**:

Old chain (Finance expense):
```
Requester → Finance Manager → Finance Director → CFO
```

Old chain (Accounting expense):
```
Requester → Accounting Manager → Accounting Director → CFO
```

New chain (Finance Operations expense):
```
Requester → Finance Ops Manager → VP Finance Operations → CFO
```

Hierarchy flattened: Two director-level roles merged into one VP role.

3. **Budget Authority Consolidation**:
```yaml
# Old budgets
finance_budget: 5000000  # $5M
accounting_budget: 2000000  # $2M

# New consolidated budget
finance_ops_budget: 7000000  # $7M

# New approval authorities
constraint:
  subject: "VP Finance Operations"
  field: "budget_authority"
  value: 7000000  # Can approve up to full dept budget
  validFrom: "2024-06-01"
```

**Conflict Detection**:

During validation, system detects:
```yaml
conflict_detection:
  conflicts:
    - type: "SOD_RULE_OBSOLETE"
      old_rule: "NO_FINANCE_APPROVER_AS_ACCOUNTING_AUDITOR"
      reason: "Departments no longer exist"
      action: "DEPRECATE_RULE"

    - type: "APPROVAL_CHAIN_BROKEN"
      affected_chains: 47  # 47 active approval requests
      reason: "Finance Director and Accounting Director roles eliminated"
      action: "REDIRECT_TO_VP_FINANCE_OPS"

    - type: "PERSON_ROLE_AMBIGUITY"
      persons: ["Alice", "Bob", "Charlie"]  # Were Finance Managers
      issue: "Old role 'Finance Manager' no longer exists, but persons still assigned"
      action: "REQUIRE_MANUAL_REASSIGNMENT"
```

**Resolution Actions**:
1. **Deprecate old SOD rule** → Create new rule for within-dept separation
2. **Redirect active approval chains** → 47 pending approvals now routed to VP Finance Ops
3. **Require role reassignment** → HR must manually assign new roles to affected people

**Validation**:
- **Observation**: Org restructure event
- **Proposer**: Generate new dept-level constraints
- **Validator**: **Detects conflicts** (SOD obsolete, approval chains broken)
- **Action**: **Manual resolution required** - present conflicts to HR/Compliance for approval
- **Snapshot**: New constraints activated ONLY AFTER conflicts resolved

---

## 11. Integration with Autonomous System (MapEKCoordinator)

### 11.1 Observation Phase: Detecting Org Changes

**Integration Point**: Org changes are **observations** that trigger the MAPEK loop.

**Sources of Observations**:
1. **HR System Events**: New hire, promotion, departure, leave
2. **Manager Actions**: Reassignment, role change, delegation
3. **Scheduled Events**: Annual clearance review, probation end

**Observation Schema**:
```yaml
observation:
  id: "obs_20240601_001"
  timestamp: "2024-06-01T09:00:00Z"
  type: "org_structure_changed"
  subtype: "promotion"
  details:
    person: "Alice"
    old_role: "Senior Engineer IC3"
    new_role: "Engineering Manager M1"
    effective_date: "2024-06-01"
    source: "HR_System"
```

**Pattern Detection**:
```python
def detect_org_change_pattern(observation):
    """
    Determine what kind of org change occurred and what policies need updating.
    """
    if observation.type == "org_structure_changed":
        if observation.subtype == "new_hire":
            return {
                "pattern": "NEW_EMPLOYEE_ONBOARDING",
                "required_policies": ["approval_authority", "data_access", "onboarding_restrictions"]
            }

        elif observation.subtype == "promotion":
            return {
                "pattern": "ROLE_ELEVATION",
                "required_policies": ["approval_authority_upgrade", "clearance_upgrade", "sod_review"]
            }

        elif observation.subtype == "departure":
            return {
                "pattern": "EMPLOYEE_OFFBOARDING",
                "required_policies": ["revoke_all_access", "transfer_ownership", "deprecate_constraints"]
            }

        elif observation.subtype == "leave_of_absence":
            return {
                "pattern": "TEMPORARY_DELEGATION",
                "required_policies": ["suspend_authority", "activate_coverage", "time_limited_constraints"]
            }

        elif observation.subtype == "restructure":
            return {
                "pattern": "ORG_RESTRUCTURE",
                "required_policies": ["rebuild_approval_chains", "review_sod_rules", "consolidate_budgets"]
            }

    return None
```

### 11.2 Proposer Phase: Generating Policy Constraints

**Integration Point**: Org-to-policy mapper is a **Proposer** agent.

**Proposer Algorithm**:
```python
class OrgToPolicyProposer:
    def propose_policy_changes(self, observation, pattern):
        """
        Generate policy constraint proposals based on org change observation.
        """
        proposals = []

        if pattern == "NEW_EMPLOYEE_ONBOARDING":
            proposals.extend(self.generate_onboarding_constraints(observation.person))

        elif pattern == "ROLE_ELEVATION":
            proposals.extend(self.generate_promotion_constraints(observation.person))

        elif pattern == "EMPLOYEE_OFFBOARDING":
            proposals.extend(self.generate_offboarding_constraints(observation.person))

        elif pattern == "TEMPORARY_DELEGATION":
            proposals.extend(self.generate_delegation_constraints(
                observation.person,
                observation.coverage_person,
                observation.duration
            ))

        elif pattern == "ORG_RESTRUCTURE":
            proposals.extend(self.generate_restructure_constraints(observation.restructure_details))

        return proposals

    def generate_promotion_constraints(self, person):
        """Generate constraints for promotion scenario."""
        new_role = person.role
        old_constraints = get_active_constraints(person)

        proposals = []

        # Deprecate old role-based constraints
        for old_constraint in old_constraints:
            if old_constraint.derivedFrom.startswith("Role:"):
                proposals.append({
                    "action": "DEPRECATE",
                    "constraint": old_constraint,
                    "effective": person.role.startDate
                })

        # Create new role-based constraints
        new_authority = calculate_approval_authority(person)
        proposals.append({
            "action": "CREATE",
            "constraint": {
                "subject": person,
                "field": "approval_authority",
                "value": new_authority,
                "validFrom": new_role.startDate,
                "derivedFrom": f"Role: {new_role.title}"
            }
        })

        # Upgrade clearance if needed
        new_clearance = calculate_required_clearance(new_role)
        if new_clearance > person.clearanceLevel:
            proposals.append({
                "action": "CREATE",
                "constraint": {
                    "subject": person,
                    "field": "clearance_level",
                    "value": new_clearance,
                    "validFrom": new_role.startDate
                }
            })

        # Check SOD rules
        sod_review = check_sod_violations(person, new_role)
        if sod_review.has_violations:
            proposals.append({
                "action": "FLAG_FOR_REVIEW",
                "issue": "SOD_VIOLATION",
                "details": sod_review
            })

        return proposals
```

### 11.3 Validator Phase: Policy Consistency Checking

**Integration Point**: Policy validator checks proposed constraints for conflicts.

**Validator Algorithm**:
```python
class PolicyConsistencyValidator:
    def validate_proposals(self, proposals):
        """
        Validate proposed policy changes for consistency and conflicts.
        """
        validation_results = []

        for proposal in proposals:
            result = {
                "proposal": proposal,
                "conflicts": [],
                "validation": "PENDING"
            }

            # Check SOD violations
            sod_conflicts = self.check_sod_violations(proposal)
            if sod_conflicts:
                result["conflicts"].extend(sod_conflicts)

            # Check approval chain integrity
            chain_conflicts = self.check_approval_chain_integrity(proposal)
            if chain_conflicts:
                result["conflicts"].extend(chain_conflicts)

            # Check clearance consistency
            clearance_conflicts = self.check_clearance_consistency(proposal)
            if clearance_conflicts:
                result["conflicts"].extend(clearance_conflicts)

            # Check budget authority limits
            budget_conflicts = self.check_budget_limits(proposal)
            if budget_conflicts:
                result["conflicts"].extend(budget_conflicts)

            # Determine validation outcome
            if not result["conflicts"]:
                result["validation"] = "PASSED"
            elif any(c["severity"] == "CRITICAL" for c in result["conflicts"]):
                result["validation"] = "FAILED"
            else:
                result["validation"] = "REQUIRES_MANUAL_REVIEW"

            validation_results.append(result)

        return validation_results

    def check_sod_violations(self, proposal):
        """Check if proposal creates SOD violations."""
        if proposal["action"] == "CREATE" and proposal["constraint"]["field"] == "role_assignment":
            person = proposal["constraint"]["subject"]
            new_role = proposal["constraint"]["value"]

            existing_roles = person.hasRole()
            sod_matrix = get_sod_matrix()

            for existing_role in existing_roles:
                if sod_matrix.are_incompatible(existing_role, new_role):
                    return [{
                        "type": "SOD_VIOLATION",
                        "severity": "CRITICAL",
                        "details": f"Person {person} cannot hold both {existing_role} and {new_role}",
                        "mitigation": "BLOCK_ASSIGNMENT"
                    }]

        return []
```

### 11.4 Snapshot Phase: Promoting Validated Constraints

**Integration Point**: Validated constraints are promoted to active policy snapshot.

**Snapshot Algorithm**:
```python
class PolicySnapshotManager:
    def promote_validated_constraints(self, validated_proposals):
        """
        Promote validated constraints to active policy snapshot.
        Create new snapshot version.
        """
        # Create new snapshot
        new_snapshot = {
            "snapshot_id": generate_snapshot_id(),
            "timestamp": NOW,
            "previous_snapshot": self.current_snapshot.id,
            "changes": []
        }

        for proposal in validated_proposals:
            if proposal["validation"] == "PASSED":
                # Apply the proposal
                if proposal["action"] == "CREATE":
                    constraint = create_constraint(proposal["constraint"])
                    new_snapshot["changes"].append({
                        "action": "ADDED",
                        "constraint": constraint
                    })

                elif proposal["action"] == "DEPRECATE":
                    deprecated_constraint = deprecate_constraint(proposal["constraint"])
                    new_snapshot["changes"].append({
                        "action": "DEPRECATED",
                        "constraint": deprecated_constraint
                    })

                elif proposal["action"] == "MODIFY":
                    modified_constraint = modify_constraint(proposal["constraint"])
                    new_snapshot["changes"].append({
                        "action": "MODIFIED",
                        "constraint": modified_constraint
                    })

        # Activate new snapshot
        self.current_snapshot = new_snapshot
        self.snapshot_history.append(new_snapshot)

        # Notify stakeholders
        notify_policy_update(new_snapshot)

        return new_snapshot
```

### 11.5 Continuous Loop: Org Change → Policy Update

**The Full MAPEK Loop for Org Changes**:

```
1. MONITOR (Observation)
   ↓
   HR System detects: "Alice promoted to Engineering Manager"
   ↓
   Observation: { type: "org_structure_changed", subtype: "promotion", person: "Alice" }

2. ANALYZE (Pattern Detection)
   ↓
   Pattern: "ROLE_ELEVATION"
   Required Policies: ["approval_authority_upgrade", "clearance_upgrade", "sod_review"]

3. PLAN (Proposer)
   ↓
   OrgToPolicyProposer generates:
   - Deprecate old IC3 constraints
   - Create new M1 approval authority ($50K)
   - Upgrade clearance to Confidential
   - Review SOD rules (manager cannot audit own team)

4. EXECUTE (Validator + Snapshot)
   ↓
   PolicyConsistencyValidator checks:
   - No SOD violations ✓
   - Approval chain integrity maintained ✓
   - Budget limits not exceeded ✓
   ↓
   PolicySnapshotManager promotes:
   - New constraints activated
   - Old constraints deprecated
   - Guard profile updated
   - Snapshot version incremented

5. KNOWLEDGE (Feedback Loop)
   ↓
   System learns:
   - "Promotion pattern successfully handled"
   - "No conflicts detected"
   - "Validation rules effective"
   ↓
   Next org change → More efficient processing
```

---

## 12. Implementation Strategy

### 12.1 Org Structure as RDF Graph

**Why RDF?**
- **Standard**: W3C standard for knowledge representation
- **Flexible**: Can represent complex relationships (matrix reporting, temporary assignments)
- **Queryable**: SPARQL enables powerful queries
- **Extensible**: Easy to add new node/edge types

**Core Ontology**:
```turtle
@prefix org: <http://www.w3.org/ns/org#> .
@prefix foaf: <http://xmlns.com/foaf/0.1/> .
@prefix time: <http://www.w3.org/2006/time#> .
@prefix policy: <http://example.com/policy#> .

# Person
org:Person a rdfs:Class ;
    rdfs:label "Person in organization" .

# Role
org:Role a rdfs:Class ;
    rdfs:label "Job role" .

# Department
org:Organization a rdfs:Class ;
    rdfs:label "Organizational unit" .

# Relationships
org:reportsTo a rdf:Property ;
    rdfs:domain org:Person ;
    rdfs:range org:Person .

org:hasRole a rdf:Property ;
    rdfs:domain org:Person ;
    rdfs:range org:Role .

org:memberOf a rdf:Property ;
    rdfs:domain org:Person ;
    rdfs:range org:Organization .

# Temporal validity
time:hasBeginning a rdf:Property ;
    rdfs:domain org:Person ;
    rdfs:range xsd:dateTime .

time:hasEnd a rdf:Property ;
    rdfs:domain org:Person ;
    rdfs:range xsd:dateTime .
```

**Example RDF Instance**:
```turtle
# Alice's promotion
:alice a org:Person ;
    foaf:name "Alice Smith" ;
    org:employeeId "EMP001" ;
    org:clearanceLevel :Confidential ;
    org:location :SanFrancisco ;
    time:hasBeginning "2020-01-15"^^xsd:dateTime .

# Old role (deprecated)
:alice_ic3_role a org:Role ;
    org:title "Senior Engineer IC3" ;
    org:level :IC3 ;
    org:function :Engineering ;
    org:approvalAuthority 5000 ;
    time:hasBeginning "2020-01-15"^^xsd:dateTime ;
    time:hasEnd "2024-05-31"^^xsd:dateTime .

:alice org:hasRole :alice_ic3_role .

# New role (active)
:alice_m1_role a org:Role ;
    org:title "Engineering Manager M1" ;
    org:level :M1 ;
    org:function :Engineering ;
    org:approvalAuthority 50000 ;
    time:hasBeginning "2024-06-01"^^xsd:dateTime .

:alice org:hasRole :alice_m1_role .

# Reporting relationship
:alice org:reportsTo :engineering_director .
```

### 12.2 SPARQL Query Language for Policy Extraction

**Query 1: Find all people with approval authority >$10K**
```sparql
PREFIX org: <http://www.w3.org/ns/org#>
PREFIX time: <http://www.w3.org/2006/time#>

SELECT ?person ?role ?authority
WHERE {
    ?person a org:Person ;
            org:hasRole ?roleAssignment .

    ?roleAssignment org:approvalAuthority ?authority ;
                    time:hasBeginning ?start .

    FILTER(?authority > 10000)
    FILTER NOT EXISTS { ?roleAssignment time:hasEnd ?end . FILTER(?end < NOW()) }
}
```

**Query 2: Find approval chain for person**
```sparql
PREFIX org: <http://www.w3.org/ns/org#>

SELECT ?level ?manager ?authority
WHERE {
    :alice org:reportsTo+ ?manager .  # Transitive closure
    ?manager org:hasRole ?role .
    ?role org:approvalAuthority ?authority .

    BIND(COUNT(?intermediate) AS ?level)  # Level in hierarchy
}
ORDER BY ?level
```

**Query 3: Detect SOD violations**
```sparql
PREFIX org: <http://www.w3.org/ns/org#>
PREFIX policy: <http://example.com/policy#>

SELECT ?person ?role1 ?role2
WHERE {
    ?person org:hasRole ?role1 ;
            org:hasRole ?role2 .

    ?role1 org:function ?func1 .
    ?role2 org:function ?func2 .

    ?sodRule policy:incompatibleFunctions (?func1 ?func2) .

    FILTER(?role1 != ?role2)
    FILTER NOT EXISTS { ?role1 time:hasEnd ?end1 }
    FILTER NOT EXISTS { ?role2 time:hasEnd ?end2 }
}
```

### 12.3 Rule Engine for Policy Generation

**Forward Chaining Rule Engine**: Apply rules to derive policy constraints from org structure.

**Example Rules**:
```yaml
rules:
  - name: "MANAGER_APPROVAL_AUTHORITY"
    condition: "person.role.level IN ['M1', 'M2', 'M3']"
    action: |
      CREATE constraint {
        subject: person,
        field: "approval_authority",
        value: person.role.approvalAuthority,
        validFrom: person.role.startDate
      }

  - name: "CEO_UNIVERSAL_OVERRIDE"
    condition: "person.role.title == 'CEO'"
    action: |
      CREATE constraint {
        subject: person,
        field: "can_override_approval_limits",
        value: true
      }

  - name: "NEW_HIRE_PROBATION"
    condition: "(NOW - person.startDate).days < 90"
    action: |
      CREATE constraint {
        subject: person,
        field: "approval_authority",
        value: min(person.role.approvalAuthority * 0.5, 5000),
        validUntil: person.startDate + 90 days
      }

  - name: "CLEARANCE_DATA_ACCESS"
    condition: "person.clearanceLevel IS DEFINED"
    action: |
      CREATE constraint {
        subject: person,
        field: "data_access_classification",
        allowed: CLEARANCE_HIERARCHY[person.clearanceLevel],
        denied: ALL_CLASSIFICATIONS - CLEARANCE_HIERARCHY[person.clearanceLevel]
      }
```

**Rule Execution**:
```python
class PolicyRuleEngine:
    def __init__(self, rules):
        self.rules = rules

    def apply_rules(self, org_graph):
        """Apply all rules to org structure to generate policy constraints."""
        generated_constraints = []

        for rule in self.rules:
            # Find all entities matching rule condition
            matches = self.evaluate_condition(rule.condition, org_graph)

            for match in matches:
                # Execute rule action
                constraint = self.execute_action(rule.action, match)
                generated_constraints.append(constraint)

        return generated_constraints

    def evaluate_condition(self, condition, org_graph):
        """Evaluate rule condition against org graph using SPARQL."""
        # Convert condition to SPARQL query
        query = self.condition_to_sparql(condition)

        # Execute query
        results = org_graph.query(query)

        return results
```

### 12.4 Storage: Dual Model (Org + Policy)

**Storage Strategy**: Keep BOTH org structure AND generated policy constraints.

**Why?**
- **Traceability**: Can trace each policy constraint back to org structure that generated it
- **Temporal Queries**: Can query "what were policies on date X?" (reconstruct from org structure at that time)
- **Audit Trail**: Complete history of org changes → policy changes

**Storage Architecture**:
```
┌─────────────────────┐
│  Org Structure DB   │  (RDF triple store)
│  - Persons          │
│  - Roles            │
│  - Departments      │
│  - Relationships    │
│  - Temporal data    │
└──────────┬──────────┘
           │
           │ (Rule engine)
           ↓
┌─────────────────────┐
│  Policy Constraints │  (Generated from org structure)
│  DB                 │
│  - Approval limits  │
│  - SOD rules        │
│  - Data access      │
│  - Guard profiles   │
└─────────────────────┘
```

**Benefits**:
- **Separation of Concerns**: Org structure is source of truth, policies are derived
- **Regeneration**: Can regenerate all policies from org structure if needed
- **Versioning**: Both org structure and policies are versioned independently

---

## Conclusion

This design analysis provides a comprehensive framework for converting organizational structures into dynamic policy constraints within an autonomous ontology system. The key insights:

1. **Org structure is a temporal DAG** with nodes (Person, Role, Dept) and edges (reportsTo, hasRole, memberOf)
2. **Policy constraints are derived** via mapping rules from org properties
3. **Approval chains are constructed** by walking the org hierarchy
4. **SOD rules are generated** from department/function incompatibilities
5. **Guard profiles protect** boundaries by encoding what each role can/cannot do
6. **Temporal validity** ensures constraints are time-windowed and automatically expire
7. **Conflict detection** prevents invalid org changes (SOD violations, approval loops)
8. **Integration with MAPEK** enables continuous org change → policy update loop

**Next Steps** (Implementation):
1. Implement RDF org structure schema
2. Build SPARQL query engine for policy extraction
3. Create rule engine for constraint generation
4. Integrate with MapEKCoordinator (Observation → Proposer → Validator → Snapshot)
5. Build guard profile validator
6. Implement conflict detection algorithms
7. Create temporal query interface
8. Build audit trail and versioning system

The system must handle the **continuous evolution** of organizations while maintaining policy consistency, detecting conflicts, and preventing security violations—all automatically, without manual policy updates.
