//! Behavioural tests for the built-in tool registry.

mod common;

use std::fs;

use common::{ApproveAll, DenyAll, TempDir};
use serde_json::json;
use target::sandbox::Workspace;
use target::tools::{ToolContext, ToolRegistry};

fn read_tool_result(
    registry: &ToolRegistry,
    workspace: &Workspace,
    tool: &str,
    arguments: serde_json::Value,
) -> anyhow::Result<String> {
    let approver = DenyAll;
    let context = ToolContext::new(workspace, &approver);
    registry.execute(tool, &context, &arguments)
}

#[test]
fn advertises_all_builtin_tools() {
    let registry = ToolRegistry::standard();
    let specs = registry.specs();
    let names: Vec<&str> = specs
        .as_array()
        .unwrap()
        .iter()
        .map(|spec| spec["function"]["name"].as_str().unwrap())
        .collect();

    assert_eq!(
        names,
        ["list_files", "read_file", "search_text", "write_file"]
    );
    assert_eq!(registry.names().count(), 4);
}

#[test]
fn lists_files_and_directories() {
    let temp = TempDir::new("tools-list");
    fs::create_dir_all(temp.path().join("src")).unwrap();
    fs::write(temp.path().join("README.md"), "# demo").unwrap();
    fs::write(temp.path().join("src").join("main.rs"), "fn main() {}").unwrap();

    let workspace = Workspace::new(temp.path()).unwrap();
    let registry = ToolRegistry::standard();
    let result = read_tool_result(&registry, &workspace, "list_files", json!({"path": "."}))
        .expect("list_files should succeed");

    assert!(result.contains("README.md"));
    assert!(result.contains("src/"));
    assert!(result.contains(
        &std::path::Path::new("src")
            .join("main.rs")
            .display()
            .to_string()
    ));
}

#[test]
fn reads_a_text_file() {
    let temp = TempDir::new("tools-read");
    fs::write(temp.path().join("hello.txt"), "hello world\n").unwrap();

    let workspace = Workspace::new(temp.path()).unwrap();
    let registry = ToolRegistry::standard();
    let result = read_tool_result(
        &registry,
        &workspace,
        "read_file",
        json!({"path": "hello.txt"}),
    )
    .expect("read_file should succeed");

    assert_eq!(result, "hello world\n");
}

#[test]
fn refuses_directories_and_oversized_files() {
    let temp = TempDir::new("tools-read-limits");
    fs::write(
        temp.path().join("big.txt"),
        "a".repeat(1_000_001).into_bytes(),
    )
    .unwrap();

    let workspace = Workspace::new(temp.path()).unwrap();
    let registry = ToolRegistry::standard();

    let directory_error =
        read_tool_result(&registry, &workspace, "read_file", json!({"path": "."}))
            .unwrap_err()
            .to_string();
    assert!(directory_error.contains("not a regular file"));

    let size_error = read_tool_result(
        &registry,
        &workspace,
        "read_file",
        json!({"path": "big.txt"}),
    )
    .unwrap_err()
    .to_string();
    assert!(size_error.contains("too large"));
}

#[test]
fn searches_text_case_insensitively() {
    let temp = TempDir::new("tools-search");
    fs::write(temp.path().join("app.log"), "alpha\nERROR: boom\nomega\n").unwrap();

    let workspace = Workspace::new(temp.path()).unwrap();
    let registry = ToolRegistry::standard();
    let result = read_tool_result(
        &registry,
        &workspace,
        "search_text",
        json!({"query": "error"}),
    )
    .expect("search_text should succeed");

    assert!(result.contains("app.log:2: ERROR: boom"));
}

#[test]
fn rejects_traversal_through_tools() {
    let temp = TempDir::new("tools-traversal");
    let workspace = Workspace::new(temp.path()).unwrap();
    let registry = ToolRegistry::standard();

    let error = read_tool_result(
        &registry,
        &workspace,
        "read_file",
        json!({"path": "../outside.txt"}),
    )
    .unwrap_err()
    .to_string();

    assert!(error.contains("Parent-directory"));
}

#[test]
fn writes_a_file_after_approval() {
    let temp = TempDir::new("tools-write");
    fs::create_dir(temp.path().join("notes")).unwrap();

    let workspace = Workspace::new(temp.path()).unwrap();
    let registry = ToolRegistry::standard();
    let approver = ApproveAll;
    let context = ToolContext::new(&workspace, &approver);

    let result = registry
        .execute(
            "write_file",
            &context,
            &json!({"path": "notes/new.txt", "content": "hello\n"}),
        )
        .expect("write_file should succeed");

    assert!(result.contains("Wrote 6 bytes"));
    assert_eq!(
        fs::read_to_string(temp.path().join("notes").join("new.txt")).unwrap(),
        "hello\n"
    );
}

#[test]
fn denied_writes_leave_no_file_behind() {
    let temp = TempDir::new("tools-deny");
    let workspace = Workspace::new(temp.path()).unwrap();
    let registry = ToolRegistry::standard();
    let approver = DenyAll;
    let context = ToolContext::new(&workspace, &approver);

    let error = registry
        .execute(
            "write_file",
            &context,
            &json!({"path": "denied.txt", "content": "nope"}),
        )
        .unwrap_err()
        .to_string();

    assert!(error.contains("denied"));
    assert!(!temp.path().join("denied.txt").exists());
}

#[test]
fn unknown_tools_are_reported() {
    let temp = TempDir::new("tools-unknown");
    let workspace = Workspace::new(temp.path()).unwrap();
    let registry = ToolRegistry::standard();

    let error = read_tool_result(&registry, &workspace, "delete_everything", json!({}))
        .unwrap_err()
        .to_string();

    assert!(error.contains("Unknown tool"));
}
