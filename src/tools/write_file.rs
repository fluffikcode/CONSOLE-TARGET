//! `write_file` — consent-gated file creation and overwrite.

use std::fs;

use anyhow::{bail, Context, Result};
use serde_json::{json, Value};

use super::{required_str, Tool, ToolContext};
use crate::approval::WriteRequest;

/// Writes a UTF-8 text file after explicit approval.
#[derive(Debug, Default, Clone, Copy)]
pub struct WriteFile;

impl Tool for WriteFile {
    fn name(&self) -> &'static str {
        "write_file"
    }

    fn description(&self) -> &'static str {
        "Create or overwrite a UTF-8 text file inside the working directory. Requires explicit user approval before writing."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Relative path to the destination file."
                },
                "content": {
                    "type": "string",
                    "description": "Complete file content to write."
                }
            },
            "required": ["path", "content"]
        })
    }

    fn execute(&self, ctx: &ToolContext<'_>, arguments: &Value) -> Result<String> {
        let requested = required_str(arguments, "path")?;
        let content = required_str(arguments, "content")?;
        let path = ctx.workspace.resolve_new_file(requested)?;
        let relative = ctx.workspace.relative(&path);

        let approved = ctx.approver.approve_write(&WriteRequest {
            path: &path,
            relative_path: &relative,
            content,
        })?;
        if !approved {
            bail!("User denied the file write operation");
        }

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Unable to create parent directory for {requested}"))?;
        }
        fs::write(&path, content).with_context(|| format!("Unable to write file: {requested}"))?;

        Ok(format!("Wrote {} bytes to {}", content.len(), requested))
    }
}
