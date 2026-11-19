//! Tool implementations exposed to the model.
//!
//! Every tool receives a [`ToolContext`] containing a sandboxed
//! [`Workspace`] and an [`Approver`]. Tools never touch the filesystem outside
//! those abstractions, which keeps the safety policy in one place.

mod list_files;
mod read_file;
mod search_text;
mod write_file;

use anyhow::{anyhow, Result};
use serde_json::{json, Value};

use crate::approval::Approver;
use crate::sandbox::Workspace;

pub use list_files::ListFiles;
pub use read_file::ReadFile;
pub use search_text::SearchText;
pub use write_file::WriteFile;

/// Execution context handed to every tool invocation.
pub struct ToolContext<'a> {
    pub workspace: &'a Workspace,
    pub approver: &'a dyn Approver,
}

impl<'a> ToolContext<'a> {
    /// Pair a workspace with the approver that guards mutations.
    pub fn new(workspace: &'a Workspace, approver: &'a dyn Approver) -> Self {
        Self {
            workspace,
            approver,
        }
    }
}

/// A capability the model can request.
pub trait Tool: Send + Sync {
    /// Stable identifier used in the API schema and in tool calls.
    fn name(&self) -> &'static str;

    /// Human-readable description shown to the model.
    fn description(&self) -> &'static str;

    /// JSON Schema describing the tool arguments.
    fn parameters(&self) -> Value;

    /// Run the tool and return the observation sent back to the model.
    fn execute(&self, ctx: &ToolContext<'_>, arguments: &Value) -> Result<String>;

    /// Full OpenAI tool definition assembled from the pieces above.
    fn spec(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": self.name(),
                "description": self.description(),
                "parameters": self.parameters(),
            }
        })
    }
}

/// Ordered collection of tools advertised to the model.
#[derive(Default)]
pub struct ToolRegistry {
    tools: Vec<Box<dyn Tool>>,
}

impl ToolRegistry {
    /// Registry populated with the built-in, sandboxed file tools.
    pub fn standard() -> Self {
        Self {
            tools: vec![
                Box::new(ListFiles),
                Box::new(ReadFile),
                Box::new(SearchText),
                Box::new(WriteFile),
            ],
        }
    }

    /// Add a custom tool. The newest tool is advertised last.
    pub fn register(&mut self, tool: Box<dyn Tool>) {
        self.tools.push(tool);
    }

    /// Names of the registered tools, in advertisement order.
    pub fn names(&self) -> impl Iterator<Item = &'static str> + '_ {
        self.tools.iter().map(|tool| tool.name())
    }

    /// Tool definitions in the format expected by the chat-completions API.
    pub fn specs(&self) -> Value {
        Value::Array(self.tools.iter().map(|tool| tool.spec()).collect())
    }

    /// Look up a tool by name.
    pub fn get(&self, name: &str) -> Option<&dyn Tool> {
        self.tools
            .iter()
            .find(|tool| tool.name() == name)
            .map(|tool| tool.as_ref())
    }

    /// Execute a tool call, surfacing unknown names as regular tool errors.
    pub fn execute(&self, name: &str, ctx: &ToolContext<'_>, arguments: &Value) -> Result<String> {
        let tool = self
            .get(name)
            .ok_or_else(|| anyhow!("Unknown tool: {name}"))?;
        tool.execute(ctx, arguments)
    }
}

/// Extract a required string argument from a tool call.
pub(crate) fn required_str<'a>(arguments: &'a Value, key: &str) -> Result<&'a str> {
    arguments
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("Missing required argument: {key}"))
}
