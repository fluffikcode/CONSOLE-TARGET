//! Consent boundary for filesystem mutations.

use std::path::Path;

use anyhow::Result;

/// Details of a pending write shown to the approver.
#[derive(Debug)]
pub struct WriteRequest<'a> {
    /// Absolute destination path, already validated by the sandbox.
    pub path: &'a Path,
    /// Path relative to the workspace root, suitable for display.
    pub relative_path: &'a str,
    /// Full content that would be written.
    pub content: &'a str,
}

/// Gate for operations that mutate the filesystem.
///
/// Implementations decide how consent is gathered. The interactive binary uses
/// [`crate::ui::TerminalApprover`]; tests can supply deterministic stand-ins.
pub trait Approver: Send + Sync {
    /// Return `true` when the write may proceed.
    fn approve_write(&self, request: &WriteRequest<'_>) -> Result<bool>;
}
