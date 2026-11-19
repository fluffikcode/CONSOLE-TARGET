//! Chat transcript entries in the shape expected by OpenAI-compatible APIs.

use serde_json::Value;

/// A single entry in the conversation history.
///
/// `content` is kept as a [`Value`] because the API accepts both plain strings
/// and structured content parts.
#[derive(Debug, Clone)]
pub struct Message {
    pub role: String,
    pub content: Value,
    pub tool_calls: Option<Value>,
    pub tool_call_id: Option<String>,
}

impl Message {
    /// System prompt that anchors the agent's behaviour.
    pub fn system(content: &str) -> Self {
        Self {
            role: "system".to_string(),
            content: Value::String(content.to_string()),
            tool_calls: None,
            tool_call_id: None,
        }
    }

    /// A task or follow-up supplied by the user.
    pub fn user(content: &str) -> Self {
        Self {
            role: "user".to_string(),
            content: Value::String(content.to_string()),
            tool_calls: None,
            tool_call_id: None,
        }
    }

    /// An assistant turn, optionally carrying tool calls.
    pub fn assistant(content: Value, tool_calls: Option<Value>) -> Self {
        Self {
            role: "assistant".to_string(),
            content,
            tool_calls,
            tool_call_id: None,
        }
    }

    /// The result of one tool invocation, correlated by `call_id`.
    pub fn tool_result(call_id: String, content: String) -> Self {
        Self {
            role: "tool".to_string(),
            content: Value::String(content),
            tool_calls: None,
            tool_call_id: Some(call_id),
        }
    }
}

impl serde::Serialize for Message {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut object = serde_json::Map::new();
        object.insert("role".to_string(), Value::String(self.role.clone()));
        object.insert("content".to_string(), self.content.clone());
        if let Some(tool_calls) = &self.tool_calls {
            object.insert("tool_calls".to_string(), tool_calls.clone());
        }
        if let Some(tool_call_id) = &self.tool_call_id {
            object.insert(
                "tool_call_id".to_string(),
                Value::String(tool_call_id.clone()),
            );
        }
        Value::Object(object).serialize(serializer)
    }
}
