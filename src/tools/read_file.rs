//! `read_file` — bounded UTF-8 file reads.

use std::fs;

use anyhow::{bail, Context, Result};
use serde_json::{json, Value};

use super::{required_str, Tool, ToolContext};

/// Largest file the agent is allowed to pull into context.
const MAX_FILE_BYTES: u64 = 1_000_000;

/// Reads a UTF-8 text file from the workspace.
#[derive(Debug, Default, Clone, Copy)]
pub struct ReadFile;

impl Tool for ReadFile {
    fn name(&self) -> &'static str {
        "read_file"
    }

    fn description(&self) -> &'static str {
        "Read a UTF-8 text file inside the working directory. Do not use this tool for binary files."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Relative path to the file."
                }
            },
            "required": ["path"]
        })
    }

    fn execute(&self, ctx: &ToolContext<'_>, arguments: &Value) -> Result<String> {
        let requested = required_str(arguments, "path")?;
        let path = ctx.workspace.resolve_existing(requested)?;
        let metadata = fs::metadata(&path)?;

        if !metadata.is_file() {
            bail!("Path is not a regular file: {requested}");
        }
        if metadata.len() > MAX_FILE_BYTES {
            bail!(
                "File is too large to read: {} bytes. Maximum supported size is {} bytes.",
                metadata.len(),
                MAX_FILE_BYTES
            );
        }

        fs::read_to_string(&path)
            .with_context(|| format!("Unable to read file as UTF-8 text: {requested}"))
    }
}
