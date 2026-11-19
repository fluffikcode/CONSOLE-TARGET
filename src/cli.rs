//! Command-line surface of the binary.

use std::path::PathBuf;

use clap::Parser;

/// Command-line options for the TARGET agent runtime.
#[derive(Debug, Clone, Parser)]
#[command(
    name = "target",
    version,
    about = "TARGET: a local, sandboxed LLM file-analysis agent",
    long_about = "TARGET is an interactive, local-first agent harness. It talks to any \
                  OpenAI-compatible endpoint and exposes a small, audited set of file \
                  tools confined to a single working directory."
)]
pub struct Args {
    /// OpenAI-compatible API base URL (for example http://localhost:11434/v1).
    #[arg(long, value_name = "URL")]
    pub base_url: String,

    /// Model name or local model path understood by the endpoint.
    #[arg(long, value_name = "MODEL")]
    pub model: String,

    /// Working directory the agent is allowed to inspect.
    #[arg(long, value_name = "DIR", default_value = ".")]
    pub workdir: PathBuf,

    /// Environment variable that holds the API key (optional for local endpoints).
    #[arg(long, value_name = "VAR", default_value = "OPENAI_API_KEY")]
    pub api_key_env: String,

    /// Maximum number of model turns (tool-call steps) per user request.
    #[arg(long, value_name = "N", default_value_t = 12)]
    pub max_steps: usize,
}
