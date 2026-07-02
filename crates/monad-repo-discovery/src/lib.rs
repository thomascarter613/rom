//! Static repository resource discovery.
//!
//! This crate scans a repository filesystem and produces a Repository Object
//! Model index. It intentionally keeps discovery conservative and shallow for v0.
//!
//! Git and the filesystem remain the source of truth. This crate only produces
//! a generated semantic index.

use monad_repo_model::{
    Relationship, Resource, ResourceId, ResourceIndex, ResourceKind, SourceKind,
};
use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Discovery options.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveryOptions {
    /// Include the synthetic root repository resource.
    pub include_repository_resource: bool,

    /// Include `repository:root contains <resource>` relationships.
    pub include_contains_relationships: bool,
}

impl Default for DiscoveryOptions {
    fn default() -> Self {
        Self {
            include_repository_resource: true,
            include_contains_relationships: true,
        }
    }
}

/// Discover repository resources using default options.
pub fn discover_repository(root: impl AsRef<Path>) -> io::Result<ResourceIndex> {
    discover_repository_with_options(root, DiscoveryOptions::default())
}

/// Discover repository resources using explicit options.
pub fn discover_repository_with_options(
    root: impl AsRef<Path>,
    options: DiscoveryOptions,
) -> io::Result<ResourceIndex> {
    let root = root.as_ref();

    if !root.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("repository root does not exist: {}", root.display()),
        ));
    }

    if !root.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("repository root is not a directory: {}", root.display()),
        ));
    }

    let mut index = ResourceIndex::new();

    if options.include_repository_resource {
        let repository = Resource::new(ResourceKind::Repository, "root", ".")
            .with_source(SourceKind::Discovered)
            .with_metadata("description", "Repository root");

        index.add_resource(repository);
    }

    discover_child_directories(
        root,
        "apps",
        ResourceKind::Application,
        &options,
        &mut index,
    )?;

    discover_child_directories(
        root,
        "services",
        ResourceKind::Service,
        &options,
        &mut index,
    )?;

    discover_child_directories(
        root,
        "packages",
        ResourceKind::Package,
        &options,
        &mut index,
    )?;

    discover_child_directories(root, "libs", ResourceKind::Package, &options, &mut index)?;

    discover_docs(root, &options, &mut index)?;

    discover_child_directories(
        root,
        "infra",
        ResourceKind::Infrastructure,
        &options,
        &mut index,
    )?;

    discover_child_directories(
        root,
        "infrastructure",
        ResourceKind::Infrastructure,
        &options,
        &mut index,
    )?;

    discover_child_directories(root, "policies", ResourceKind::Policy, &options, &mut index)?;

    discover_child_directories(
        root,
        "policy-as-code",
        ResourceKind::Policy,
        &options,
        &mut index,
    )?;

    discover_workflows(root, &options, &mut index)?;

    Ok(index)
}

fn discover_child_directories(
    root: &Path,
    collection: &str,
    kind: ResourceKind,
    options: &DiscoveryOptions,
    index: &mut ResourceIndex,
) -> io::Result<()> {
    let collection_path = root.join(collection);

    if !collection_path.exists() {
        return Ok(());
    }

    if !collection_path.is_dir() {
        index
            .warnings
            .push(format!("Expected `{collection}` to be a directory."));
        return Ok(());
    }

    for entry in sorted_entries(&collection_path)? {
        let path = entry.path();
        let file_name = entry.file_name();

        if should_ignore_name(&file_name) {
            continue;
        }

        if !path.is_dir() {
            continue;
        }

        let Some(name) = file_name.to_str() else {
            index.warnings.push(format!(
                "Skipped non-UTF-8 path under `{collection}`: {}",
                path.display()
            ));
            continue;
        };

        let relative_path = format!("{collection}/{name}");
        add_discovered_resource(kind, name, relative_path, options, index);
    }

    Ok(())
}

fn discover_docs(
    root: &Path,
    options: &DiscoveryOptions,
    index: &mut ResourceIndex,
) -> io::Result<()> {
    let docs_path = root.join("docs");

    if !docs_path.exists() {
        return Ok(());
    }

    if !docs_path.is_dir() {
        index
            .warnings
            .push("Expected `docs` to be a directory.".to_string());
        return Ok(());
    }

    for entry in sorted_entries(&docs_path)? {
        let path = entry.path();
        let file_name = entry.file_name();

        if should_ignore_name(&file_name) {
            continue;
        }

        if path.is_dir() {
            let Some(name) = file_name.to_str() else {
                index
                    .warnings
                    .push(format!("Skipped non-UTF-8 docs path: {}", path.display()));
                continue;
            };

            add_discovered_resource(
                ResourceKind::Documentation,
                name,
                format!("docs/{name}"),
                options,
                index,
            );

            continue;
        }

        if is_markdown_file(&path) {
            let Some(stem) = path.file_stem().and_then(OsStr::to_str) else {
                index.warnings.push(format!(
                    "Skipped markdown file with invalid name: {}",
                    path.display()
                ));
                continue;
            };

            let Some(file_name) = file_name.to_str() else {
                index.warnings.push(format!(
                    "Skipped non-UTF-8 markdown file: {}",
                    path.display()
                ));
                continue;
            };

            add_discovered_resource(
                ResourceKind::Documentation,
                stem,
                format!("docs/{file_name}"),
                options,
                index,
            );
        }
    }

    Ok(())
}

fn discover_workflows(
    root: &Path,
    options: &DiscoveryOptions,
    index: &mut ResourceIndex,
) -> io::Result<()> {
    let workflow_path = root.join(".github").join("workflows");

    if !workflow_path.exists() {
        return Ok(());
    }

    if !workflow_path.is_dir() {
        index
            .warnings
            .push("Expected `.github/workflows` to be a directory.".to_string());
        return Ok(());
    }

    for entry in sorted_entries(&workflow_path)? {
        let path = entry.path();

        if !path.is_file() || !is_yaml_file(&path) {
            continue;
        }

        let Some(stem) = path.file_stem().and_then(OsStr::to_str) else {
            index
                .warnings
                .push(format!("Skipped invalid workflow path: {}", path.display()));
            continue;
        };

        let Some(file_name) = path.file_name().and_then(OsStr::to_str) else {
            index.warnings.push(format!(
                "Skipped non-UTF-8 workflow path: {}",
                path.display()
            ));
            continue;
        };

        add_discovered_resource(
            ResourceKind::Workflow,
            stem,
            format!(".github/workflows/{file_name}"),
            options,
            index,
        );
    }

    Ok(())
}

fn add_discovered_resource(
    kind: ResourceKind,
    name: &str,
    path: String,
    options: &DiscoveryOptions,
    index: &mut ResourceIndex,
) {
    let resource = Resource::new(kind, name, path).with_source(SourceKind::Discovered);
    let id = resource.id.clone();

    index.add_resource(resource);

    if options.include_repository_resource && options.include_contains_relationships {
        index.add_relationship(
            Relationship::contains(ResourceId::new("repository:root").as_str(), id.as_str())
                .with_source(SourceKind::Discovered),
        );
    }
}

fn sorted_entries(path: &Path) -> io::Result<Vec<fs::DirEntry>> {
    let mut entries = fs::read_dir(path)?.collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.file_name());
    Ok(entries)
}

fn should_ignore_name(name: &OsStr) -> bool {
    let Some(name) = name.to_str() else {
        return false;
    };

    matches!(
        name,
        ".git"
            | "node_modules"
            | "target"
            | "dist"
            | "build"
            | "coverage"
            | ".next"
            | ".turbo"
            | ".venv"
            | "__pycache__"
            | ".DS_Store"
    )
}

fn is_markdown_file(path: &Path) -> bool {
    path.extension()
        .and_then(OsStr::to_str)
        .map(|extension| extension.eq_ignore_ascii_case("md"))
        .unwrap_or(false)
}

fn is_yaml_file(path: &Path) -> bool {
    path.extension()
        .and_then(OsStr::to_str)
        .map(|extension| {
            extension.eq_ignore_ascii_case("yml") || extension.eq_ignore_ascii_case("yaml")
        })
        .unwrap_or(false)
}

/// Return a normalized resource path string using forward slashes.
///
/// This helper is currently reserved for future deeper scanning where paths may
/// come from platform-specific filesystem traversal.
pub fn normalize_path(path: impl Into<PathBuf>) -> String {
    path.into()
        .components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

#[cfg(test)]
mod tests {
    use super::*;
    use monad_repo_model::{RelationshipType, ResourceKind};
    use std::collections::BTreeSet;
    use tempfile::tempdir;

    fn mkdir(path: &Path) {
        fs::create_dir_all(path).expect("test directory should be created");
    }

    fn touch(path: &Path) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("parent directory should be created");
        }

        fs::write(path, "").expect("test file should be written");
    }

    fn ids(index: &ResourceIndex) -> BTreeSet<String> {
        index
            .resources
            .iter()
            .map(|resource| resource.id.to_string())
            .collect()
    }

    #[test]
    fn discovers_common_repository_resources() {
        let temp = tempdir().expect("tempdir should be created");
        let root = temp.path();

        mkdir(&root.join("apps/web"));
        mkdir(&root.join("services/auth"));
        mkdir(&root.join("packages/ui"));
        mkdir(&root.join("libs/core"));
        mkdir(&root.join("docs/product"));
        touch(&root.join("docs/00-index.md"));
        mkdir(&root.join("infra/local-dev"));
        mkdir(&root.join("policies/security-baseline"));
        touch(&root.join(".github/workflows/ci.yml"));

        let index = discover_repository(root).expect("repository should be discovered");
        let ids = ids(&index);

        assert!(ids.contains("repository:root"));
        assert!(ids.contains("application:web"));
        assert!(ids.contains("service:auth"));
        assert!(ids.contains("package:ui"));
        assert!(ids.contains("package:core"));
        assert!(ids.contains("documentation:product"));
        assert!(ids.contains("documentation:00-index"));
        assert!(ids.contains("infrastructure:local-dev"));
        assert!(ids.contains("policy:security-baseline"));
        assert!(ids.contains("workflow:ci"));
    }

    #[test]
    fn creates_contains_relationships_from_repository_root() {
        let temp = tempdir().expect("tempdir should be created");
        let root = temp.path();

        mkdir(&root.join("apps/web"));

        let index = discover_repository(root).expect("repository should be discovered");

        assert!(index.relationships.iter().any(|relationship| {
            relationship.from.as_str() == "repository:root"
                && relationship.to.as_str() == "application:web"
                && relationship.relationship_type == RelationshipType::Contains
        }));
    }

    #[test]
    fn can_disable_repository_root_and_contains_relationships() {
        let temp = tempdir().expect("tempdir should be created");
        let root = temp.path();

        mkdir(&root.join("apps/web"));

        let index = discover_repository_with_options(
            root,
            DiscoveryOptions {
                include_repository_resource: false,
                include_contains_relationships: false,
            },
        )
        .expect("repository should be discovered");

        let ids = ids(&index);

        assert!(!ids.contains("repository:root"));
        assert!(ids.contains("application:web"));
        assert!(index.relationships.is_empty());
    }

    #[test]
    fn ignores_non_yaml_workflow_files() {
        let temp = tempdir().expect("tempdir should be created");
        let root = temp.path();

        touch(&root.join(".github/workflows/ci.yml"));
        touch(&root.join(".github/workflows/notes.txt"));

        let index = discover_repository(root).expect("repository should be discovered");
        let ids = ids(&index);

        assert!(ids.contains("workflow:ci"));
        assert!(!ids.contains("workflow:notes"));
    }

    #[test]
    fn warns_when_expected_collection_is_a_file() {
        let temp = tempdir().expect("tempdir should be created");
        let root = temp.path();

        touch(&root.join("apps"));

        let index = discover_repository(root).expect("repository should be discovered");

        assert!(index
            .warnings
            .iter()
            .any(|warning| warning.contains("Expected `apps` to be a directory")));
    }

    #[test]
    fn returns_error_for_missing_root() {
        let temp = tempdir().expect("tempdir should be created");
        let missing = temp.path().join("missing");

        let error = discover_repository(&missing).expect_err("missing root should error");

        assert_eq!(error.kind(), io::ErrorKind::NotFound);
    }

    #[test]
    fn returns_error_when_root_is_not_directory() {
        let temp = tempdir().expect("tempdir should be created");
        let file = temp.path().join("not-a-directory.txt");
        touch(&file);

        let error = discover_repository(&file).expect_err("file root should error");

        assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
    }

    #[test]
    fn normalizes_platform_paths_to_forward_slashes() {
        let path = PathBuf::from("apps").join("web").join("src");

        assert_eq!(normalize_path(path), "apps/web/src");
    }

    #[test]
    fn discovered_resources_have_expected_kinds_and_paths() {
        let temp = tempdir().expect("tempdir should be created");
        let root = temp.path();

        mkdir(&root.join("services/billing"));

        let index = discover_repository(root).expect("repository should be discovered");
        let billing = index
            .resources
            .iter()
            .find(|resource| resource.id.as_str() == "service:billing")
            .expect("service should be discovered");

        assert_eq!(billing.kind, ResourceKind::Service);
        assert_eq!(billing.name, "billing");
        assert_eq!(billing.path, "services/billing");
        assert_eq!(billing.source, SourceKind::Discovered);
    }
}
