//! Shared fixtures for integration tests.

#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;

use target::approval::{Approver, WriteRequest};

/// Self-cleaning unique temporary directory.
pub struct TempDir {
    path: PathBuf,
}

impl TempDir {
    pub fn new(label: &str) -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);

        let nonce = COUNTER.fetch_add(1, Ordering::Relaxed);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "target-{label}-{}-{nanos}-{nonce}",
            std::process::id()
        ));

        std::fs::create_dir_all(&path).expect("create temporary directory");
        Self { path }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

/// Approver that accepts every write.
#[derive(Debug, Default, Clone, Copy)]
pub struct ApproveAll;

impl Approver for ApproveAll {
    fn approve_write(&self, _request: &WriteRequest<'_>) -> Result<bool> {
        Ok(true)
    }
}

/// Approver that rejects every write.
#[derive(Debug, Default, Clone, Copy)]
pub struct DenyAll;

impl Approver for DenyAll {
    fn approve_write(&self, _request: &WriteRequest<'_>) -> Result<bool> {
        Ok(false)
    }
}
