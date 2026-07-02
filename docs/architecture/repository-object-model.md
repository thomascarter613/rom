
# Repository Object Model

Status: Draft v0.1
Date: 2026-07-01
Related Brief: `docs/product/repository-control-plane-exploration-brief.md`

## 1. Purpose

The Repository Object Model defines how a software repository is represented as a typed graph of resources, relationships, actions, policies, lifecycle states, and metadata.

This model is the foundation for the Repository Control Plane.

The model should allow Monad OS to answer:

* What exists in this repository?
* Where does each resource live?
* What kind of thing is each resource?
* What owns what?
* What depends on what?
* What policies apply?
* What actions are valid?
* What context should a human or AI agent receive?
* What changes are safe?
* What changes require planning, review, or approval?

## 2. Design Principles

### 2.1 Git-Native

Git and the filesystem remain canonical.

The Repository Object Model is a semantic layer over Git, not a replacement for Git.

### 2.2 Resource-Oriented

The model should be based on resources, not folder inheritance.

Root-level folders are usually collections or namespaces. The actual meaningful objects are the resources inside or represented by those folders.

### 2.3 Inference First

The system should infer resources from repository structure and manifests wherever possible.

Manual metadata should refine the model, not be required for every useful result.

### 2.4 Stable Identity

Every discovered or declared resource should have a stable ID.

A stable ID allows references, graph edges, reports, policies, audit records, context packs, and plans to remain coherent.

### 2.5 Safe Mutation

Changes should usually go through plans before being applied.

The model should support plan generation, diffs, risk reporting, validation, and future pull-request workflows.

### 2.6 AI-Ready

The model should support context generation for AI coding assistants and future agents.

Context should be scoped by resource, task, dependency neighborhood, recent changes, policy constraints, and relevant docs.

## 3. Core Concepts

## 3.1 Repository

A repository is the top-level version-controlled workspace.

Example resource ID:

```text
repository:root
```

## 3.2 Workspace

A workspace is a logical grouping of projects/resources inside a repository.

Examples:

```text
workspace:default
workspace:apps
workspace:packages
```

## 3.3 Resource

A resource is a typed software object represented in the repository.

Examples:

```text
application:web
service:billing
package:ui
documentation:architecture
workflow:ci
policy:security-baseline
infrastructure:local-dev
```

## 3.4 Relationship

A relationship is a typed edge between resources.

Examples:

```text
application:web depends_on package:ui
application:web calls service:auth
service:billing uses database:billing-db
documentation:architecture documents service:billing
workflow:ci builds application:web
policy:security-baseline validates service:billing
```

## 3.5 Action

An action is an operation that may be run against a resource.

Examples:

```text
build
test
lint
format
deploy
generate-docs
generate-context
deprecate
move
delete
split
```

## 3.6 Policy

A policy is a rule that validates repository state or resource state.

Examples:

```text
Every service must have an owner.
Every package must have a README.
Every application must declare build and test actions.
No resource may depend on a missing resource.
Deprecated resources must have migration notes.
```

## 3.7 Lifecycle State

A lifecycle state describes where a resource is in its lifespan.

Initial states:

```text
proposed
active
experimental
deprecated
archived
removed
```

## 4. Resource Shape

Initial canonical resource shape:

```toml
[[resources]]
id = "service:billing"
kind = "service"
name = "billing"
path = "services/billing"
lifecycle = "active"
owners = ["platform"]

[resources.metadata]
description = "Billing service."

[resources.spec]
language = "go"
runtime = "container"
database = "postgres"

[resources.actions]
build = "go build ./..."
test = "go test ./..."
lint = "golangci-lint run"
```

Required fields:

* `id`
* `kind`
* `name`
* `path`

Recommended fields:

* `lifecycle`
* `owners`
* `metadata.description`
* `spec.language`
* `actions.build`
* `actions.test`

## 5. Relationship Shape

Initial canonical relationship shape:

```toml
[[relationships]]
from = "application:web"
to = "service:auth"
type = "calls"

[[relationships]]
from = "application:web"
to = "package:ui"
type = "depends_on"
```

Required fields:

* `from`
* `to`
* `type`

## 6. Resource ID Convention

Recommended format:

```text
<kind>:<name>
```

Examples:

```text
application:web
application:admin-console
service:auth
service:billing
package:ui
package:config
documentation:architecture
workflow:ci
policy:security-baseline
```

Names should be stable, lowercase, and kebab-case unless a future project standard says otherwise.

## 7. Initial Resource Kinds

V0 should support:

```text
repository
workspace
application
service
package
documentation
workflow
policy
infrastructure
```

Later versions may support:

```text
library
module
environment
database
migration
api-contract
event-contract
secret-reference
owner
task
release
risk
context-pack
```

## 8. Initial Relationship Types

V0 should support:

```text
contains
depends_on
calls
owns
documents
validates
builds
deploys
uses
```

Later versions may support:

```text
exposes
consumes
publishes
subscribes_to
migrates
tests
generates
protects
replaces
deprecates
```

## 9. Discovery Sources

The scanner should discover resources from:

```text
root directories
workspace manifests
package manifests
Dockerfiles
compose files
CI workflows
README files
ADR directories
CODEOWNERS
OpenAPI files
AsyncAPI files
Terraform directories
Pulumi directories
policy directories
```

Initial discovery heuristics:

* `apps/*` may become `application:*`
* `services/*` may become `service:*`
* `packages/*` may become `package:*`
* `libs/*` may become `package:*` or `library:*`
* `docs/*` may become `documentation:*`
* `.github/workflows/*` may become `workflow:*`
* `infra/*` may become `infrastructure:*`
* `policies/*` or `policy-as-code/*` may become `policy:*`

## 10. Manifest Strategy

The model may be represented in a repo manifest such as:

```text
workspace.toml
```

or later:

```text
monad.toml
```

For current compatibility with existing Monad direction, `workspace.toml` should be acceptable as the initial canonical manifest.

The manifest should contain explicit resources and relationships. The scanner may also generate an index of inferred resources.

Recommended split:

```text
workspace.toml       = user-authored or tool-updated source intent
.monad/index.json    = generated resource index
.monad/graph.json    = generated graph projection
.monad/plans/        = generated plans
```

## 11. Generated Index

The generated resource index should be machine-readable.

Example:

```json
{
  "schema_version": "0.1",
  "resources": [
    {
      "id": "application:web",
      "kind": "application",
      "name": "web",
      "path": "apps/web",
      "source": "discovered"
    }
  ],
  "relationships": [
    {
      "from": "application:web",
      "to": "package:ui",
      "type": "depends_on",
      "source": "manifest"
    }
  ]
}
```

## 12. Policy Model

V0 policies should be simple built-in checks.

Initial policy checks:

* Every resource has an ID.
* Every resource has a kind.
* Every resource has a name.
* Every resource has a path.
* Every resource path exists.
* Resource IDs are unique.
* Relationship endpoints refer to existing resources.
* Applications, services, and packages should have README files.
* Applications, services, and packages should declare owners.
* Applications, services, and packages should declare build/test actions where applicable.

Policy output should be suitable for both humans and CI.

## 13. Context Pack Model

A context pack is a generated bundle of repository knowledge for a human or AI assistant.

A resource-scoped context pack should include:

* Resource identity
* Resource purpose
* Path
* Owners
* Lifecycle state
* Important files
* Dependencies
* Dependents
* Related docs
* Related ADRs
* Relevant policies
* Build/test/lint commands
* Known issues or missing metadata
* Recent changes later
* Suggested next actions

## 14. Planning Model

A plan is a proposed repository mutation.

Example:

```text
Plan: Create service:billing

Will create:
- services/billing/
- services/billing/README.md
- services/billing/src/
- services/billing/tests/

Will update:
- workspace.toml
- docs/repository/resource-index.md

Validation:
- owner missing
- README included
- build/test actions missing

Risk:
- Low
```

A plan should be reviewable before it mutates the repository.

## 15. Open Questions

1. Should `workspace.toml` or `monad.toml` be canonical?
2. Should resources be declared centrally, locally, or both?
3. Should every resource eventually have a local `resource.toml`?
4. Should lifecycle state live in the manifest, generated index, or both?
5. How much should be inferred versus explicitly declared?
6. How should package-manager dependency graphs be merged with semantic resource relationships?
7. How should the model handle polyrepos later?
8. How should repo resources map into Backstage catalog entities later?
9. How should OPA/Rego integration be introduced?
10. How should AI context packs be validated for completeness and usefulness?

## 16. V0 Recommendation

V0 should implement the smallest useful version:

* Discover resources from common directories.
* Generate `.monad/index.json`.
* Generate `.monad/graph.json`.
* Render a Markdown resource index.
* Validate basic policies.
* Generate one resource-scoped context pack.
* Generate a plan for creating one service.

