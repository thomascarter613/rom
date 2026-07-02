//! Generated repository control plane artifacts.
//!
//! This crate writes durable generated projections of the Repository Object
//! Model. The initial artifacts are:
//!
//! - `.monad/index.json`
//! - `.monad/graph.json`

use monad_repo_model::{Graph, ResourceIndex};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub const MONAD_DIR: &str = ".monad";
pub const INDEX_JSON: &str = "index.json";
pub const GRAPH_JSON: &str = "graph.json";

/// Standard generated artifact paths for a repository root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactPaths {
    pub monad_dir: PathBuf,
    pub index_json: PathBuf,
    pub graph_json: PathBuf,
}

impl ArtifactPaths {
    pub fn for_root(root: impl AsRef<Path>) -> Self {
        let monad_dir = root.as_ref().join(MONAD_DIR);

        Self {
            index_json: monad_dir.join(INDEX_JSON),
            graph_json: monad_dir.join(GRAPH_JSON),
            monad_dir,
        }
    }
}

/// Summary of generated artifact writes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WrittenArtifacts {
    pub index_json: PathBuf,
    pub graph_json: PathBuf,
}

/// Write all standard repository artifacts.
pub fn write_repository_artifacts(
    root: impl AsRef<Path>,
    index: &ResourceIndex,
) -> io::Result<WrittenArtifacts> {
    let paths = ArtifactPaths::for_root(root);

    fs::create_dir_all(&paths.monad_dir)?;

    write_index_json(&paths.index_json, index)?;

    let graph = index.to_graph();
    write_graph_json(&paths.graph_json, &graph)?;

    Ok(WrittenArtifacts {
        index_json: paths.index_json,
        graph_json: paths.graph_json,
    })
}

/// Write `.monad/index.json` or another caller-provided index path.
pub fn write_index_json(path: impl AsRef<Path>, index: &ResourceIndex) -> io::Result<()> {
    write_json_pretty(path, index)
}

/// Write `.monad/graph.json` or another caller-provided graph path.
pub fn write_graph_json(path: impl AsRef<Path>, graph: &Graph) -> io::Result<()> {
    write_json_pretty(path, graph)
}

/// Read a generated `ResourceIndex`.
pub fn read_index_json(path: impl AsRef<Path>) -> io::Result<ResourceIndex> {
    let content = fs::read_to_string(path)?;
    serde_json::from_str(&content).map_err(json_error)
}

/// Read a generated `Graph`.
pub fn read_graph_json(path: impl AsRef<Path>) -> io::Result<Graph> {
    let content = fs::read_to_string(path)?;
    serde_json::from_str(&content).map_err(json_error)
}

fn write_json_pretty<T>(path: impl AsRef<Path>, value: &T) -> io::Result<()>
where
    T: serde::Serialize,
{
    if let Some(parent) = path.as_ref().parent() {
        fs::create_dir_all(parent)?;
    }

    let json = serde_json::to_string_pretty(value).map_err(json_error)?;
    fs::write(path, format!("{json}\n"))
}

fn json_error(error: serde_json::Error) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use monad_repo_model::{Relationship, Resource, ResourceIndex, ResourceKind};
    use tempfile::tempdir;

    fn sample_index() -> ResourceIndex {
        let mut index = ResourceIndex::new();

        index.add_resource(Resource::new(ResourceKind::Repository, "root", "."));
        index.add_resource(Resource::new(ResourceKind::Application, "web", "apps/web"));
        index.add_relationship(Relationship::contains("repository:root", "application:web"));

        index
    }

    #[test]
    fn resolves_standard_artifact_paths() {
        let paths = ArtifactPaths::for_root("/repo");

        assert_eq!(paths.monad_dir, PathBuf::from("/repo/.monad"));
        assert_eq!(paths.index_json, PathBuf::from("/repo/.monad/index.json"));
        assert_eq!(paths.graph_json, PathBuf::from("/repo/.monad/graph.json"));
    }

    #[test]
    fn writes_index_json() {
        let temp = tempdir().expect("tempdir should be created");
        let path = temp.path().join(".monad/index.json");
        let index = sample_index();

        write_index_json(&path, &index).expect("index should be written");

        let content = fs::read_to_string(&path).expect("index should be readable");

        assert!(content.contains("\"schema_version\""));
        assert!(content.contains("\"application:web\""));
    }

    #[test]
    fn writes_graph_json() {
        let temp = tempdir().expect("tempdir should be created");
        let path = temp.path().join(".monad/graph.json");
        let index = sample_index();
        let graph = index.to_graph();

        write_graph_json(&path, &graph).expect("graph should be written");

        let content = fs::read_to_string(&path).expect("graph should be readable");

        assert!(content.contains("\"nodes\""));
        assert!(content.contains("\"edges\""));
        assert!(content.contains("\"application:web\""));
    }

    #[test]
    fn writes_standard_repository_artifacts() {
        let temp = tempdir().expect("tempdir should be created");
        let index = sample_index();

        let written = write_repository_artifacts(temp.path(), &index)
            .expect("repository artifacts should be written");

        assert!(written.index_json.exists());
        assert!(written.graph_json.exists());

        assert_eq!(written.index_json, temp.path().join(".monad/index.json"));
        assert_eq!(written.graph_json, temp.path().join(".monad/graph.json"));
    }

    #[test]
    fn reads_written_index_json() {
        let temp = tempdir().expect("tempdir should be created");
        let index = sample_index();
        let paths = ArtifactPaths::for_root(temp.path());

        write_repository_artifacts(temp.path(), &index)
            .expect("repository artifacts should be written");

        let read = read_index_json(paths.index_json).expect("index should be read");

        assert_eq!(read.resources.len(), 2);
        assert_eq!(read.relationships.len(), 1);
    }

    #[test]
    fn reads_written_graph_json() {
        let temp = tempdir().expect("tempdir should be created");
        let index = sample_index();
        let paths = ArtifactPaths::for_root(temp.path());

        write_repository_artifacts(temp.path(), &index)
            .expect("repository artifacts should be written");

        let read = read_graph_json(paths.graph_json).expect("graph should be read");

        assert_eq!(read.nodes.len(), 2);
        assert_eq!(read.edges.len(), 1);
    }

    #[test]
    fn read_index_json_returns_error_for_invalid_json() {
        let temp = tempdir().expect("tempdir should be created");
        let path = temp.path().join("index.json");

        fs::write(&path, "{not valid json").expect("invalid json should be written");

        let error = read_index_json(&path).expect_err("invalid json should fail");

        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    }
}
