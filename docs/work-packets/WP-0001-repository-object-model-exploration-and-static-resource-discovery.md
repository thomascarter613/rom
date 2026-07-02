
# WP-0001: Repository Object Model Exploration and Static Resource Discovery

Status: Proposed
Type: Work Packet
Area: Repository Control Plane
Date: 2026-07-01
Depends On: Repository Control Plane Exploration Brief, Repository Object Model Draft

## 1. Summary

Implement the first concrete exploration work packet for the Repository Control Plane concept.

This work packet introduces a minimal Repository Object Model and a static discovery capability that can inspect a repository, identify candidate software resources, generate a resource index, generate a basic relationship graph, validate simple repository policies, and produce human-readable reports.

This is the first proof point for treating a repository as a typed, queryable, governable software resource graph.

## 2. Objective

Build the first local-first, CLI-accessible capability that proves Monad OS can understand a repository as more than a file tree.

The objective is not to build the full Repository Operating System.

The objective is to prove that the repository can be represented as typed resources and relationships in a useful, inspectable, and extensible way.

## 3. Problem Statement

Modern repositories contain many meaningful software objects:

* Applications
* Services
* Packages
* Documentation
* Workflows
* Policies
* Infrastructure
* Tests
* Contracts
* Architecture decisions

Today, these are usually scattered across folders, manifests, documentation, workflows, and implicit conventions.

This makes it difficult to answer basic questions:

* What exists in this repo?
* What kind of thing is each folder?
* Which resources are apps, services, or packages?
* Which resources have owners?
* Which resources are missing docs?
* Which resources have build/test actions?
* Which resources depend on other resources?
* What context should an AI assistant receive for a resource?

WP-0001 begins solving this by creating a static resource discovery and indexing capability.

## 4. Scope

In scope:

* Minimal resource model
* Static directory-based discovery
* Manifest-aware discovery where easy
* Generated resource index
* Generated graph projection
* Basic validation rules
* Markdown report generation
* Resource list/show commands
* Initial AI context pack generation
* Basic plan output for creating a service

Out of scope:

* Web dashboard
* Database projection
* Full policy engine
* OPA/Rego integration
* Full language parsing
* Full dependency graph parsing for all ecosystems
* Automatic refactoring
* Pull request creation
* Remote API
* Plugin system
* Multi-repo support

## 5. Initial Resource Kinds

WP-0001 should support the following resource kinds:

```text
repository
application
service
package
documentation
workflow
policy
infrastructure
```

## 6. Initial Discovery Rules

The scanner should use conservative defaults:

```text
apps/*                  -> application
services/*              -> service
packages/*              -> package
libs/*                  -> package or library later; package for WP-0001
docs/*                  -> documentation
.github/workflows/*.yml -> workflow
.github/workflows/*.yaml -> workflow
infra/*                 -> infrastructure
infrastructure/*         -> infrastructure
policies/*              -> policy
policy-as-code/*         -> policy
```

The scanner should ignore common generated or dependency directories:

```text
.git
node_modules
target
dist
build
coverage
.next
.turbo
.moon/cache
.venv
__pycache__
.DS_Store
```

## 7. Resource ID Rules

Resource IDs should use:

```text
<kind>:<name>
```

Examples:

```text
application:web
service:auth
package:ui
documentation:architecture
workflow:ci
policy:security-baseline
infrastructure:local-dev
```

Names should be generated from directory or file names using lowercase kebab-case.

## 8. Generated Files

WP-0001 should generate:

```text
.monad/index.json
.monad/graph.json
docs/repository/resource-index.md
docs/repository/repository-health-report.md
```

Optional generated files:

```text
.monad/context/
.monad/plans/
```

## 9. CLI Commands

Initial commands should be shaped toward:

```bash
monad repo discover
monad resources list
monad resource show <id>
monad graph
monad validate
monad report health
monad context pack <id>
monad plan create service <name>
```

If current CLI structure is not ready for all commands, implement the smallest available equivalent and document gaps.

## 10. Discovery Output

The generated `.monad/index.json` should include:

```json
{
  "schema_version": "0.1",
  "generated_by": "monad",
  "resources": [],
  "relationships": [],
  "warnings": []
}
```

Each resource should include:

```json
{
  "id": "application:web",
  "kind": "application",
  "name": "web",
  "path": "apps/web",
  "source": "discovered",
  "metadata": {}
}
```

Each relationship should include:

```json
{
  "from": "repository:root",
  "to": "application:web",
  "type": "contains",
  "source": "discovered"
}
```

## 11. Graph Output

The generated `.monad/graph.json` should include nodes and edges.

Minimum shape:

```json
{
  "schema_version": "0.1",
  "nodes": [],
  "edges": []
}
```

Node shape:

```json
{
  "id": "application:web",
  "kind": "application",
  "label": "web",
  "path": "apps/web"
}
```

Edge shape:

```json
{
  "from": "repository:root",
  "to": "application:web",
  "type": "contains"
}
```

## 12. Validation Rules

Initial built-in validation rules:

1. Every resource must have an ID.
2. Every resource must have a kind.
3. Every resource must have a name.
4. Every resource must have a path.
5. Every resource path must exist.
6. Resource IDs must be unique.
7. Relationship endpoints must refer to existing resources.
8. Applications should have a README.
9. Services should have a README.
10. Packages should have a README.
11. Applications should declare an owner if owner metadata exists.
12. Services should declare an owner if owner metadata exists.
13. Packages should declare an owner if owner metadata exists.

Validation severity levels:

```text
info
warning
error
```

V0 should avoid being too strict. Missing README and missing owner can start as warnings.

## 13. Markdown Resource Index

Generate:

```text
docs/repository/resource-index.md
```

The report should contain:

* Generation timestamp
* Repository name
* Resource count
* Resource table
* Relationship count
* Relationship table
* Warnings

Resource table columns:

```text
ID | Kind | Name | Path | Source
```

Relationship table columns:

```text
From | Type | To | Source
```

## 14. Repository Health Report

Generate:

```text
docs/repository/repository-health-report.md
```

The report should contain:

* Summary
* Resource counts by kind
* Validation results
* Missing README findings
* Missing owner findings
* Broken relationship findings
* Suggested next actions

## 15. AI Context Pack

Implement a simple context pack command if feasible.

Example:

```bash
monad context pack service:billing
```

Generated output may be Markdown initially.

Context pack should include:

* Resource ID
* Kind
* Name
* Path
* Lifecycle state if known
* Owners if known
* Related resources
* Related docs if known
* Build/test actions if known
* Validation findings
* Recommended files to inspect

## 16. Service Creation Plan

Implement or document a basic plan command:

```bash
monad plan create service billing
```

The plan should not need to apply changes in WP-0001.

Minimum output:

```text
Plan: Create service:billing

Would create:
- services/billing/
- services/billing/README.md

Would update:
- workspace.toml or repository manifest if present
- docs/repository/resource-index.md after next discovery

Validation:
- owner missing
- build/test actions missing

Risk:
- Low
```

## 17. Acceptance Criteria

WP-0001 is complete when:

* The repository has formal docs for the Repository Control Plane exploration.
* The repository has a formal Repository Object Model draft.
* The CLI can discover at least basic resources from a real repo.
* The CLI can produce `.monad/index.json`.
* The CLI can produce `.monad/graph.json`.
* The CLI can list discovered resources.
* The CLI can show a discovered resource by ID.
* The CLI can validate basic repository rules.
* The CLI can generate a Markdown resource index.
* The CLI can generate a Markdown health report.
* The implementation has tests for discovery and resource ID generation.
* The implementation has tests for duplicate resource IDs.
* The implementation has tests for broken relationship detection.
* Existing workspace checks continue passing.

## 18. Suggested Implementation Slices

### Slice 1: Model Types

Create internal structs/types for:

* Resource
* ResourceKind
* Relationship
* RelationshipType
* ResourceIndex
* Graph
* ValidationFinding

### Slice 2: Discovery

Implement directory-based discovery for:

* apps
* services
* packages
* libs
* docs
* infra
* policies
* .github/workflows

### Slice 3: Index and Graph Generation

Generate:

* `.monad/index.json`
* `.monad/graph.json`

### Slice 4: CLI Display

Add commands or subcommands for:

* resource listing
* resource show
* graph output

### Slice 5: Validation

Add built-in validation checks.

### Slice 6: Reports

Generate Markdown resource index and repository health report.

### Slice 7: Context Pack

Generate resource-scoped context pack.

### Slice 8: Plan Command

Generate dry-run service creation plan.

## 19. Recommended Commit Message

```text
docs: define repository control plane exploration
```

Implementation commits after this documentation commit may use:

```text
feat: add repository resource discovery
feat: generate repository resource graph
feat: add repository validation report
feat: generate resource context packs
```

## 20. Notes

This work packet should remain deliberately modest.

The goal is proof, not platform sprawl.

The question WP-0001 should answer is:

> Can Monad OS produce useful understanding, validation, and context from a repository by modeling it as a typed resource graph?

If yes, the Repository Control Plane direction should continue.

