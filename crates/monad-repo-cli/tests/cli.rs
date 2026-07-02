use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::Path;
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

#[test]
fn repo_discover_writes_artifacts() {
    let temp = tempdir().expect("tempdir should be created");

    mkdir(&temp.path().join("apps/web"));
    mkdir(&temp.path().join("services/auth"));

    Command::cargo_bin("monad")
        .expect("monad binary should exist")
        .arg("--root")
        .arg(temp.path())
        .args(["repo", "discover"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Repository discovery complete."));

    assert!(temp.path().join(".monad/index.json").exists());
    assert!(temp.path().join(".monad/graph.json").exists());
}

#[test]
fn resources_list_prints_discovered_resources() {
    let temp = tempdir().expect("tempdir should be created");

    mkdir(&temp.path().join("apps/web"));

    Command::cargo_bin("monad")
        .expect("monad binary should exist")
        .arg("--root")
        .arg(temp.path())
        .args(["repo", "discover"])
        .assert()
        .success();

    Command::cargo_bin("monad")
        .expect("monad binary should exist")
        .arg("--root")
        .arg(temp.path())
        .args(["resources", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("application:web"));
}

#[test]
fn resource_show_prints_resource_details() {
    let temp = tempdir().expect("tempdir should be created");

    mkdir(&temp.path().join("services/billing"));

    Command::cargo_bin("monad")
        .expect("monad binary should exist")
        .arg("--root")
        .arg(temp.path())
        .args(["repo", "discover"])
        .assert()
        .success();

    Command::cargo_bin("monad")
        .expect("monad binary should exist")
        .arg("--root")
        .arg(temp.path())
        .args(["resource", "show", "service:billing"])
        .assert()
        .success()
        .stdout(predicate::str::contains("ID: service:billing"))
        .stdout(predicate::str::contains("Path: services/billing"));
}

#[test]
fn graph_prints_text_graph() {
    let temp = tempdir().expect("tempdir should be created");

    mkdir(&temp.path().join("apps/web"));

    Command::cargo_bin("monad")
        .expect("monad binary should exist")
        .arg("--root")
        .arg(temp.path())
        .args(["repo", "discover"])
        .assert()
        .success();

    Command::cargo_bin("monad")
        .expect("monad binary should exist")
        .arg("--root")
        .arg(temp.path())
        .arg("graph")
        .assert()
        .success()
        .stdout(predicate::str::contains("Nodes:"))
        .stdout(predicate::str::contains("application:web"));
}

#[test]
fn graph_prints_mermaid_graph() {
    let temp = tempdir().expect("tempdir should be created");

    mkdir(&temp.path().join("apps/web"));

    Command::cargo_bin("monad")
        .expect("monad binary should exist")
        .arg("--root")
        .arg(temp.path())
        .args(["repo", "discover"])
        .assert()
        .success();

    Command::cargo_bin("monad")
        .expect("monad binary should exist")
        .arg("--root")
        .arg(temp.path())
        .args(["graph", "--format", "mermaid"])
        .assert()
        .success()
        .stdout(predicate::str::contains("graph TD"))
        .stdout(predicate::str::contains("application_web"));
}

#[test]
fn resources_list_without_discovery_fails_with_helpful_message() {
    let temp = tempdir().expect("tempdir should be created");

    Command::cargo_bin("monad")
        .expect("monad binary should exist")
        .arg("--root")
        .arg(temp.path())
        .args(["resources", "list"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Run `monad repo discover` first"));
}

#[test]
fn repo_discover_json_outputs_index_json() {
    let temp = tempdir().expect("tempdir should be created");

    touch(&temp.path().join(".github/workflows/ci.yml"));

    Command::cargo_bin("monad")
        .expect("monad binary should exist")
        .arg("--root")
        .arg(temp.path())
        .args(["repo", "discover", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"workflow:ci\""));
}
