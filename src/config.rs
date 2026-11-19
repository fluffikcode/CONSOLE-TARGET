//! Validated runtime configuration.

use std::{env, path::PathBuf};

use anyhow::{bail, Context, Result};

use crate::cli::Args;

/// Validated runtime configuration derived from [`Args`] and the environment.
#[derive(Debug, Clone)]
pub struct Settings {
    /// Base URL of the OpenAI-compatible endpoint, without a trailing slash.
    pub base_url: String,
    /// Model identifier sent with every request.
    pub model: String,
    /// Canonical working directory the agent is confined to.
    pub workdir: PathBuf,
    /// API key read from the configured environment variable, if present.
    pub api_key: Option<String>,
    /// Hard limit on model turns per user request.
    pub max_steps: usize,
}

impl Settings {
    /// Validate CLI arguments and resolve environment-dependent values.
    pub fn from_args(args: &Args) -> Result<Self> {
        let model = args.model.trim().to_string();
        if model.is_empty() {
            bail!("--model must not be empty");
        }

        let base_url = args.base_url.trim().trim_end_matches('/').to_string();
        if base_url.is_empty() {
            bail!("--base-url must not be empty");
        }

        if args.max_steps == 0 {
            bail!("--max-steps must be at least 1");
        }

        let workdir = args
            .workdir
            .canonicalize()
            .with_context(|| format!("Unable to open working directory: {:?}", args.workdir))?;
        if !workdir.is_dir() {
            bail!("Working directory is not a directory: {:?}", workdir);
        }

        let api_key = env::var(&args.api_key_env)
            .ok()
            .map(|key| key.trim().to_string())
            .filter(|key| !key.is_empty());

        Ok(Self {
            base_url,
            model,
            workdir,
            api_key,
            max_steps: args.max_steps,
        })
    }
}
