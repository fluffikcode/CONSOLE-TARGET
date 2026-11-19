//! `list_files` — shallow recursive directory listing.

use anyhow::Result;
use serde_json::{json, Value};
use walkdir::WalkDir;

use super::{Tool, ToolContext};

/// Maximum depth walked from the requested directory.
const MAX_DEPTH: usize = 3;

/// Lists files and directories under the workspace.
#[derive(Debug, Default, Clone, Copy)]
pub struct ListFiles;

impl Tool for ListFiles {
    fn name(&self) -> &'static str {
        "list_files"
    }

    fn description(&self) -> &'static str {
        "List files and directories inside the working directory. Use this first when exploring an unfamiliar project."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Relative path inside the working directory. Use '.' for the root."
                }
            }
        })
    }

    fn execute(&self, ctx: &ToolContext<'_>, arguments: &Value) -> Result<String> {
        let requested = arguments.get("path").and_then(Value::as_str).unwrap_or(".");
        let directory = ctx.workspace.resolve_existing(requested)?;

        let mut entries = Vec::new();
        for entry in WalkDir::new(&directory)
            .max_depth(MAX_DEPTH)
            .follow_links(false)
            .into_iter()
            .flatten()
        {
            let path = entry.path();
            if path == ctx.workspace.root() {
                continue;
            }
            let relative = ctx.workspace.relative(path);
            let label = if entry.file_type().is_dir() {
                format!("{relative}/")
            } else {
                relative
            };
            entries.push(label);
        }
        entries.sort();

        if entries.is_empty() {
            Ok("No files or directories found.".to_string())
        } else {
            Ok(entries.join("\n"))
        }
    }
}
