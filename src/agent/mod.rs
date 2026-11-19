//! Conversation loop that binds the model, the tools, and the sandbox.

use std::sync::Arc;

use anyhow::{bail, Result};
use serde_json::Value;

use crate::approval::Approver;
use crate::config::Settings;
use crate::llm::LlmClient;
use crate::message::Message;
use crate::prompt::SYSTEM_PROMPT;
use crate::sandbox::Workspace;
use crate::tools::{ToolContext, ToolRegistry};
use crate::ui;

/// Sampling temperature used for every request.
const TEMPERATURE: f32 = 0.2;

/// Stateful agent session: transcript, sandbox, tools, and model client.
pub struct Agent {
    client: LlmClient,
    workspace: Workspace,
    tools: ToolRegistry,
    approver: Arc<dyn Approver>,
    messages: Vec<Message>,
    max_steps: usize,
}

impl Agent {
    /// Build a session from validated settings and a consent provider.
    pub fn new(settings: &Settings, approver: Arc<dyn Approver>) -> Result<Self> {
        let workspace = Workspace::new(&settings.workdir)?;
        let client = LlmClient::new(
            settings.base_url.clone(),
            settings.model.clone(),
            settings.api_key.clone(),
        )?;

        Ok(Self {
            client,
            workspace,
            tools: ToolRegistry::standard(),
            approver,
            messages: vec![Message::system(SYSTEM_PROMPT)],
            max_steps: settings.max_steps,
        })
    }

    /// Number of messages currently held in context.
    pub fn message_count(&self) -> usize {
        self.messages.len()
    }

    /// Drop the conversation and start over from the system prompt.
    pub fn reset_memory(&mut self) {
        self.messages = vec![Message::system(SYSTEM_PROMPT)];
    }

    /// Run one user request to completion, executing any tool calls the model
    /// requests along the way.
    pub async fn run_task(&mut self, task: &str) -> Result<String> {
        self.messages.push(Message::user(task));
        println!();

        for step in 0..self.max_steps {
            let answer = self.call_model(step).await?;
            if !answer.trim().is_empty() {
                return Ok(answer);
            }
            if step + 1 == self.max_steps {
                bail!(
                    "Reached the maximum number of tool-call steps ({}).",
                    self.max_steps
                );
            }
        }

        bail!("Agent loop ended without a final answer");
    }

    /// Ask the model for the next turn and execute its tool calls.
    ///
    /// Returns the final text when the model stops requesting tools, or an
    /// empty string when the loop should continue.
    async fn call_model(&mut self, step: usize) -> Result<String> {
        let spinner_msg = if step == 0 {
            "PROCESSING NEURAL STREAM".to_string()
        } else {
            format!("SYNTHESIZING STEP {:02}/{:02}", step + 1, self.max_steps)
        };

        let spinner = ui::Spinner::start(&spinner_msg);
        let turn = self
            .client
            .complete(&self.messages, self.tools.specs(), TEMPERATURE)
            .await;
        spinner.stop();
        let turn = turn?;

        let tool_calls = turn.tool_calls.clone();
        self.messages.push(Message::assistant(
            turn.content.clone().unwrap_or(Value::Null),
            tool_calls.clone(),
        ));

        let Some(calls) = tool_calls.and_then(|value| value.as_array().cloned()) else {
            let answer = turn
                .content
                .and_then(|value| value.as_str().map(str::to_string))
                .unwrap_or_default();
            return Ok(answer);
        };

        for call in &calls {
            self.execute_tool_call(step, call);
        }

        Ok(String::new())
    }

    /// Execute a single tool call and append its observation to the transcript.
    fn execute_tool_call(&mut self, step: usize, call: &Value) {
        let call_id = call["id"].as_str().unwrap_or_default().to_string();
        let function = &call["function"];
        let name = function["name"].as_str().unwrap_or_default().to_string();
        let raw_arguments = function["arguments"].as_str().unwrap_or("{}");

        ui::print_tool_call(step + 1, self.max_steps, &name, raw_arguments);

        let arguments: Value = match serde_json::from_str(raw_arguments) {
            Ok(value) => value,
            Err(error) => {
                let result = format!(
                    "Tool arguments were invalid JSON: {}. Raw arguments: {}",
                    error, raw_arguments
                );
                ui::print_tool_result(&result, true);
                self.messages.push(Message::tool_result(call_id, result));
                return;
            }
        };

        let context = ToolContext::new(&self.workspace, self.approver.as_ref());
        let (result, is_err) = match self.tools.execute(&name, &context, &arguments) {
            Ok(result) => (result, false),
            Err(error) => (format!("Tool execution failed: {:#}", error), true),
        };

        ui::print_tool_result(&result, is_err);
        self.messages.push(Message::tool_result(call_id, result));
    }
}
