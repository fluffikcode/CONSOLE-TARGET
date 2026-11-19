//! TARGET — a local, sandboxed LLM file-analysis agent harness.
//!
//! The crate is split into narrow layers so that the model provider, the
//! sandbox, the tools, and the terminal UI can evolve independently:
//!
//! - [`cli`] and [`config`] parse and validate startup options.
//! - [`agent`] owns the conversation loop and orchestrates model turns.
//! - [`llm`] speaks the OpenAI-compatible chat-completions protocol.
//! - [`message`] models the chat transcript sent back to the endpoint.
//! - [`tools`] implements the capabilities exposed to the model.
//! - [`sandbox`] enforces that every path stays inside the working directory.
//! - [`approval`] gates destructive operations behind user consent.
//! - [`prompt`] holds the system prompt that defines the agent's contract.
//! - [`ui`] renders the terminal interface.
//!
//! The binary entry point in `main.rs` only wires these pieces together.

#![forbid(unsafe_code)]

pub mod agent;
pub mod approval;
pub mod cli;
pub mod config;
pub mod llm;
pub mod message;
pub mod prompt;
pub mod sandbox;
pub mod tools;
pub mod ui;

pub use agent::Agent;
pub use cli::Args;
pub use config::Settings;
