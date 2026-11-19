//! `search_text` — case-insensitive literal search across text files.

use std::fs;

use anyhow::{bail, Result};
use serde_json::{json, Value};
use walkdir::WalkDir;

use super::{required_str, Tool, ToolContext};

/// Maximum directory depth walked while searching.
const MAX_DEPTH: usize = 8;
/// Stop after this many matches to keep responses small.
const MAX_RESULTS: usize = 100;

/// Searches text files under the workspace for a literal needle.
#[derive(Debug, Default, Clone, Copy)]
pub struct SearchText;

impl Tool for SearchText {
    fn name(&self) -> &'static str {
        "search_text"
    }

    fn description(&self) -> &'static str {
        "Search case-insensitively for a text string in text files inside the working directory."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Text to search for."
                }
            },
            "required": ["query"]
        })
    }

    fn execute(&self, ctx: &ToolContext<'_>, arguments: &Value) -> Result<String> {
        let query = required_str(arguments, "query")?;
        if query.trim().is_empty() {
            bail!("Search query must not be empty");
        }

        let needle = query.to_lowercase();
        let mut results = Vec::new();

        for entry in WalkDir::new(ctx.workspace.root())
            .max_depth(MAX_DEPTH)
            .follow_links(false)
            .into_iter()
            .flatten()
        {
            if !entry.file_type().is_file() {
                continue;
            }
            let path = entry.path();
            let Ok(content) = fs::read_to_string(path) else {
                // Binary or non-UTF-8 files are skipped silently.
                continue;
            };
            for (index, line) in content.lines().enumerate() {
                if line.to_lowercase().contains(&needle) {
                    results.push(format!(
                        "{}:{}: {}",
                        ctx.workspace.relative(path),
                        index + 1,
                        line.trim()
                    ));
                }
                if results.len() >= MAX_RESULTS {
                    results.push(format!(
                        "[Search result limit reached: {MAX_RESULTS} matches]"
                    ));
                    return Ok(results.join("\n"));
                }
            }
        }

        if results.is_empty() {
            Ok(format!("No matches found for: {query}"))
        } else {
            Ok(results.join("\n"))
        }
    }
}
