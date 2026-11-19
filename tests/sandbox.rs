//! Path-policy tests for the sandbox.

mod common;

use std::fs;

use common::TempDir;
use target::sandbox::Workspace;

#[test]
fn resolves_existing_file_inside_workspace() {
    let temp = TempDir::new("sandbox-existing");
    let file = temp.path().join("notes.txt");
    fs::write(&file, "hello").unwrap();

    let workspace = Workspace::new(temp.path()).unwrap();
    let resolved = workspace.resolve_existing("notes.txt").unwrap();

    assert_eq!(resolved, file.canonicalize().unwrap());
}

#[test]
fn resolves_root_with_dot() {
    let temp = TempDir::new("sandbox-root");
    let workspace = Workspace::new(temp.path()).unwrap();

    assert_eq!(workspace.resolve_existing(".").unwrap(), workspace.root());
}

#[test]
fn rejects_parent_directory_components() {
    let temp = TempDir::new("sandbox-parent");
    let workspace = Workspace::new(temp.path()).unwrap();

    let error = workspace
        .resolve_existing("../secret.txt")
        .unwrap_err()
        .to_string();
    assert!(error.contains("Parent-directory"));
}

#[test]
fn rejects_absolute_paths() {
    let temp = TempDir::new("sandbox-absolute");
    let workspace = Workspace::new(temp.path()).unwrap();

    let error = workspace
        .resolve_existing("/etc/passwd")
        .unwrap_err()
        .to_string();
    assert!(error.contains("Absolute"));
}

#[test]
fn rejects_home_directory_paths() {
    let temp = TempDir::new("sandbox-home");
    let workspace = Workspace::new(temp.path()).unwrap();

    let error = workspace
        .resolve_existing("~/notes.txt")
        .unwrap_err()
        .to_string();
    assert!(error.contains("home-directory"));
}

#[test]
fn rejects_empty_paths() {
    let temp = TempDir::new("sandbox-empty");
    let workspace = Workspace::new(temp.path()).unwrap();

    assert!(workspace.resolve_existing("   ").is_err());
}

#[test]
fn new_file_must_live_under_an_existing_parent() {
    let temp = TempDir::new("sandbox-new-parent");
    fs::create_dir(temp.path().join("sub")).unwrap();
    let workspace = Workspace::new(temp.path()).unwrap();

    let resolved = workspace.resolve_new_file("sub/new.txt").unwrap();
    assert!(resolved.starts_with(workspace.root()));
    assert_eq!(resolved.file_name().unwrap(), "new.txt");

    assert!(workspace.resolve_new_file("missing/new.txt").is_err());
}

#[test]
fn new_file_rejects_traversal() {
    let temp = TempDir::new("sandbox-new-traversal");
    let workspace = Workspace::new(temp.path()).unwrap();

    let error = workspace
        .resolve_new_file("sub/../../escape.txt")
        .unwrap_err()
        .to_string();
    assert!(error.contains("Parent-directory"));
}

#[test]
fn workspace_root_must_be_a_directory() {
    let temp = TempDir::new("sandbox-file-root");
    let file = temp.path().join("plain.txt");
    fs::write(&file, "data").unwrap();

    assert!(Workspace::new(&file).is_err());
}

#[test]
fn relative_paths_are_rendered_from_the_root() {
    let temp = TempDir::new("sandbox-relative");
    let nested = temp.path().join("a").join("b.txt");
    fs::create_dir(temp.path().join("a")).unwrap();
    fs::write(&nested, "x").unwrap();

    let workspace = Workspace::new(temp.path()).unwrap();
    let rendered = workspace.relative(&nested.canonicalize().unwrap());

    assert_eq!(
        rendered,
        std::path::Path::new("a")
            .join("b.txt")
            .display()
            .to_string()
    );
}
