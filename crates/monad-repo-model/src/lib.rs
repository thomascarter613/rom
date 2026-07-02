//! Repository Object Model primitives.
//!
//! This crate defines the durable model layer for the Repository Control Plane.
//! It intentionally contains no filesystem scanning or CLI behavior.
//!
//! The core idea is that a repository can be represented as a typed graph of
//! resources and relationships, while Git and the filesystem remain the source
//! of truth.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Current schema version for generated model artifacts.
pub const SCHEMA_VERSION: &str = "0.1";

/// A typed software resource inside a repository.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Resource {
    pub id: ResourceId,
    pub kind: ResourceKind,
    pub name: String,
    pub path: String,

    #[serde(default)]
    pub lifecycle: LifecycleState,

    #[serde(default)]
    pub owners: Vec<String>,

    #[serde(default)]
    pub source: SourceKind,

    #[serde(default)]
    pub metadata: BTreeMap<String, String>,

    #[serde(default)]
    pub spec: BTreeMap<String, String>,

    #[serde(default)]
    pub actions: BTreeMap<String, String>,
}

impl Resource {
    pub fn new(kind: ResourceKind, name: impl Into<String>, path: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            id: ResourceId::from_kind_and_name(kind, &name),
            kind,
            name,
            path: path.into(),
            lifecycle: LifecycleState::Unknown,
            owners: Vec::new(),
            source: SourceKind::Discovered,
            metadata: BTreeMap::new(),
            spec: BTreeMap::new(),
            actions: BTreeMap::new(),
        }
    }

    pub fn with_source(mut self, source: SourceKind) -> Self {
        self.source = source;
        self
    }

    pub fn with_lifecycle(mut self, lifecycle: LifecycleState) -> Self {
        self.lifecycle = lifecycle;
        self
    }

    pub fn with_owner(mut self, owner: impl Into<String>) -> Self {
        self.owners.push(owner.into());
        self
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    pub fn with_spec(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.spec.insert(key.into(), value.into());
        self
    }

    pub fn with_action(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.actions.insert(key.into(), value.into());
        self
    }
}

/// Stable resource identity.
///
/// Format:
///
/// ```text
/// <kind>:<name>
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ResourceId(pub String);

impl ResourceId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn from_kind_and_name(kind: ResourceKind, name: &str) -> Self {
        Self(format!("{}:{}", kind.as_str(), normalize_name(name)))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ResourceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// Known repository resource kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ResourceKind {
    Repository,
    Workspace,
    Application,
    Service,
    Package,
    Documentation,
    Workflow,
    Policy,
    Infrastructure,
    Library,
    Module,
    Environment,
    Database,
    Migration,
    ApiContract,
    EventContract,
    SecretReference,
    Owner,
    Task,
    Release,
    Risk,
    ContextPack,
    Unknown,
}

impl ResourceKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Repository => "repository",
            Self::Workspace => "workspace",
            Self::Application => "application",
            Self::Service => "service",
            Self::Package => "package",
            Self::Documentation => "documentation",
            Self::Workflow => "workflow",
            Self::Policy => "policy",
            Self::Infrastructure => "infrastructure",
            Self::Library => "library",
            Self::Module => "module",
            Self::Environment => "environment",
            Self::Database => "database",
            Self::Migration => "migration",
            Self::ApiContract => "api-contract",
            Self::EventContract => "event-contract",
            Self::SecretReference => "secret-reference",
            Self::Owner => "owner",
            Self::Task => "task",
            Self::Release => "release",
            Self::Risk => "risk",
            Self::ContextPack => "context-pack",
            Self::Unknown => "unknown",
        }
    }
}

/// Source of a resource or relationship.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SourceKind {
    Discovered,
    Manifest,
    Generated,
    User,
    Unknown,
}

impl Default for SourceKind {
    fn default() -> Self {
        Self::Unknown
    }
}

/// Resource lifecycle state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LifecycleState {
    Proposed,
    Active,
    Experimental,
    Deprecated,
    Archived,
    Removed,
    Unknown,
}

impl Default for LifecycleState {
    fn default() -> Self {
        Self::Unknown
    }
}

/// A typed edge between two resources.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Relationship {
    pub from: ResourceId,
    pub to: ResourceId,

    #[serde(rename = "type")]
    pub relationship_type: RelationshipType,

    #[serde(default)]
    pub source: SourceKind,

    #[serde(default)]
    pub metadata: BTreeMap<String, String>,
}

impl Relationship {
    pub fn new(
        from: impl Into<String>,
        to: impl Into<String>,
        relationship_type: RelationshipType,
    ) -> Self {
        Self {
            from: ResourceId::new(from),
            to: ResourceId::new(to),
            relationship_type,
            source: SourceKind::Discovered,
            metadata: BTreeMap::new(),
        }
    }

    pub fn contains(from: impl Into<String>, to: impl Into<String>) -> Self {
        Self::new(from, to, RelationshipType::Contains)
    }

    pub fn with_source(mut self, source: SourceKind) -> Self {
        self.source = source;
        self
    }
}

/// Known relationship types between repository resources.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RelationshipType {
    Contains,
    DependsOn,
    Calls,
    Owns,
    Documents,
    Validates,
    Builds,
    Deploys,
    Uses,
    Exposes,
    Consumes,
    Publishes,
    SubscribesTo,
    Migrates,
    Tests,
    Generates,
    Protects,
    Replaces,
    Deprecates,
    Unknown,
}

impl RelationshipType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Contains => "contains",
            Self::DependsOn => "depends-on",
            Self::Calls => "calls",
            Self::Owns => "owns",
            Self::Documents => "documents",
            Self::Validates => "validates",
            Self::Builds => "builds",
            Self::Deploys => "deploys",
            Self::Uses => "uses",
            Self::Exposes => "exposes",
            Self::Consumes => "consumes",
            Self::Publishes => "publishes",
            Self::SubscribesTo => "subscribes-to",
            Self::Migrates => "migrates",
            Self::Tests => "tests",
            Self::Generates => "generates",
            Self::Protects => "protects",
            Self::Replaces => "replaces",
            Self::Deprecates => "deprecates",
            Self::Unknown => "unknown",
        }
    }
}

/// Generated resource index.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceIndex {
    pub schema_version: String,
    pub generated_by: String,

    #[serde(default)]
    pub resources: Vec<Resource>,

    #[serde(default)]
    pub relationships: Vec<Relationship>,

    #[serde(default)]
    pub warnings: Vec<String>,
}

impl Default for ResourceIndex {
    fn default() -> Self {
        Self {
            schema_version: SCHEMA_VERSION.to_string(),
            generated_by: "monad".to_string(),
            resources: Vec::new(),
            relationships: Vec::new(),
            warnings: Vec::new(),
        }
    }
}

impl ResourceIndex {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_resource(&mut self, resource: Resource) {
        self.resources.push(resource);
    }

    pub fn add_relationship(&mut self, relationship: Relationship) {
        self.relationships.push(relationship);
    }

    pub fn resource_ids(&self) -> BTreeSet<ResourceId> {
        self.resources
            .iter()
            .map(|resource| resource.id.clone())
            .collect()
    }

    pub fn duplicate_resource_ids(&self) -> Vec<ResourceId> {
        let mut seen = BTreeSet::new();
        let mut duplicates = BTreeSet::new();

        for resource in &self.resources {
            if !seen.insert(resource.id.clone()) {
                duplicates.insert(resource.id.clone());
            }
        }

        duplicates.into_iter().collect()
    }

    pub fn broken_relationships(&self) -> Vec<&Relationship> {
        let ids = self.resource_ids();

        self.relationships
            .iter()
            .filter(|relationship| {
                !ids.contains(&relationship.from) || !ids.contains(&relationship.to)
            })
            .collect()
    }

    pub fn validate_basic(&self) -> Vec<ValidationFinding> {
        let mut findings = Vec::new();

        for duplicate_id in self.duplicate_resource_ids() {
            findings.push(ValidationFinding::error(
                "duplicate-resource-id",
                format!("Duplicate resource ID: {duplicate_id}"),
                Some(duplicate_id),
            ));
        }

        for relationship in self.broken_relationships() {
            let message = format!(
                "Broken relationship: {} {} {}",
                relationship.from,
                relationship.relationship_type.as_str(),
                relationship.to
            );

            findings.push(ValidationFinding::error(
                "broken-relationship",
                message,
                None,
            ));
        }

        for resource in &self.resources {
            if resource.name.trim().is_empty() {
                findings.push(ValidationFinding::error(
                    "missing-resource-name",
                    format!("Resource {} has an empty name", resource.id),
                    Some(resource.id.clone()),
                ));
            }

            if resource.path.trim().is_empty() {
                findings.push(ValidationFinding::error(
                    "missing-resource-path",
                    format!("Resource {} has an empty path", resource.id),
                    Some(resource.id.clone()),
                ));
            }
        }

        findings
    }

    pub fn to_graph(&self) -> Graph {
        let nodes = self
            .resources
            .iter()
            .map(|resource| GraphNode {
                id: resource.id.clone(),
                kind: resource.kind,
                label: resource.name.clone(),
                path: resource.path.clone(),
            })
            .collect();

        let edges = self
            .relationships
            .iter()
            .map(|relationship| GraphEdge {
                from: relationship.from.clone(),
                to: relationship.to.clone(),
                relationship_type: relationship.relationship_type,
            })
            .collect();

        Graph {
            schema_version: self.schema_version.clone(),
            nodes,
            edges,
        }
    }
}

/// Serializable graph projection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Graph {
    pub schema_version: String,

    #[serde(default)]
    pub nodes: Vec<GraphNode>,

    #[serde(default)]
    pub edges: Vec<GraphEdge>,
}

/// Serializable graph node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: ResourceId,
    pub kind: ResourceKind,
    pub label: String,
    pub path: String,
}

/// Serializable graph edge.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphEdge {
    pub from: ResourceId,
    pub to: ResourceId,

    #[serde(rename = "type")]
    pub relationship_type: RelationshipType,
}

/// Validation finding severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ValidationSeverity {
    Info,
    Warning,
    Error,
}

/// Validation finding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationFinding {
    pub code: String,
    pub severity: ValidationSeverity,
    pub message: String,

    #[serde(default)]
    pub resource_id: Option<ResourceId>,
}

impl ValidationFinding {
    pub fn info(
        code: impl Into<String>,
        message: impl Into<String>,
        resource_id: Option<ResourceId>,
    ) -> Self {
        Self {
            code: code.into(),
            severity: ValidationSeverity::Info,
            message: message.into(),
            resource_id,
        }
    }

    pub fn warning(
        code: impl Into<String>,
        message: impl Into<String>,
        resource_id: Option<ResourceId>,
    ) -> Self {
        Self {
            code: code.into(),
            severity: ValidationSeverity::Warning,
            message: message.into(),
            resource_id,
        }
    }

    pub fn error(
        code: impl Into<String>,
        message: impl Into<String>,
        resource_id: Option<ResourceId>,
    ) -> Self {
        Self {
            code: code.into(),
            severity: ValidationSeverity::Error,
            message: message.into(),
            resource_id,
        }
    }
}

/// Normalize a human/resource name into a stable kebab-case-ish identifier.
///
/// This intentionally stays conservative for v0.
pub fn normalize_name(input: &str) -> String {
    let mut output = String::new();
    let mut previous_was_separator = false;

    for character in input.trim().chars() {
        if character.is_ascii_alphanumeric() {
            output.push(character.to_ascii_lowercase());
            previous_was_separator = false;
        } else if !previous_was_separator {
            output.push('-');
            previous_was_separator = true;
        }
    }

    output.trim_matches('-').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_names_for_stable_ids() {
        assert_eq!(normalize_name("Admin Console"), "admin-console");
        assert_eq!(normalize_name("admin_console"), "admin-console");
        assert_eq!(normalize_name("  Billing API  "), "billing-api");
        assert_eq!(normalize_name("ui"), "ui");
    }

    #[test]
    fn creates_resource_id_from_kind_and_name() {
        let id = ResourceId::from_kind_and_name(ResourceKind::Application, "Admin Console");
        assert_eq!(id.as_str(), "application:admin-console");
    }

    #[test]
    fn creates_resource_with_expected_defaults() {
        let resource = Resource::new(ResourceKind::Service, "billing", "services/billing");

        assert_eq!(resource.id.as_str(), "service:billing");
        assert_eq!(resource.kind, ResourceKind::Service);
        assert_eq!(resource.name, "billing");
        assert_eq!(resource.path, "services/billing");
        assert_eq!(resource.lifecycle, LifecycleState::Unknown);
        assert_eq!(resource.source, SourceKind::Discovered);
    }

    #[test]
    fn detects_duplicate_resource_ids() {
        let mut index = ResourceIndex::new();

        index.add_resource(Resource::new(
            ResourceKind::Service,
            "billing",
            "services/billing",
        ));
        index.add_resource(Resource::new(
            ResourceKind::Service,
            "billing",
            "services/billing-v2",
        ));

        let duplicates = index.duplicate_resource_ids();

        assert_eq!(duplicates.len(), 1);
        assert_eq!(duplicates[0].as_str(), "service:billing");
    }

    #[test]
    fn detects_broken_relationships() {
        let mut index = ResourceIndex::new();

        index.add_resource(Resource::new(ResourceKind::Repository, "root", "."));
        index.add_relationship(Relationship::contains("repository:root", "service:missing"));

        let broken = index.broken_relationships();

        assert_eq!(broken.len(), 1);
        assert_eq!(broken[0].to.as_str(), "service:missing");
    }

    #[test]
    fn validates_duplicate_ids_and_broken_relationships() {
        let mut index = ResourceIndex::new();

        index.add_resource(Resource::new(
            ResourceKind::Service,
            "billing",
            "services/billing",
        ));
        index.add_resource(Resource::new(
            ResourceKind::Service,
            "billing",
            "services/billing-v2",
        ));
        index.add_relationship(Relationship::contains("repository:root", "service:billing"));

        let findings = index.validate_basic();

        assert_eq!(findings.len(), 2);
        assert!(findings
            .iter()
            .any(|finding| finding.code == "duplicate-resource-id"));
        assert!(findings
            .iter()
            .any(|finding| finding.code == "broken-relationship"));
    }

    #[test]
    fn converts_index_to_graph() {
        let mut index = ResourceIndex::new();

        index.add_resource(Resource::new(ResourceKind::Repository, "root", "."));
        index.add_resource(Resource::new(ResourceKind::Application, "web", "apps/web"));
        index.add_relationship(Relationship::contains("repository:root", "application:web"));

        let graph = index.to_graph();

        assert_eq!(graph.nodes.len(), 2);
        assert_eq!(graph.edges.len(), 1);
        assert_eq!(graph.edges[0].relationship_type, RelationshipType::Contains);
    }

    #[test]
    fn serializes_resource_index() {
        let mut index = ResourceIndex::new();
        index.add_resource(Resource::new(ResourceKind::Application, "web", "apps/web"));

        let json = serde_json::to_string_pretty(&index).expect("index should serialize");

        assert!(json.contains("\"schema_version\""));
        assert!(json.contains("\"application:web\""));
        assert!(json.contains("\"application\""));
    }
}
