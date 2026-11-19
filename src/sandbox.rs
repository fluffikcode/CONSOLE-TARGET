//! Path policy for the agent's single working directory.
//!
//! Every tool that touches the filesystem must resolve its path through a
//! [`Workspace`]. The rules are intentionally conservative:
//!
//! - empty, absolute, and home (`~`) paths are rejected;
//! - `..` components are rejected before any filesystem access;
//! - existing paths are canonicalised and must remain under the root, which
//!   also defeats symlink escapes;
//! - new files must live in a parent directory that canonicalises inside the
//!   root.

use std::path::{Component, Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};

/// A canonical working directory and the path rules that guard it.
#[derive(Debug, Clone)]
pub struct Workspace {
    root: PathBuf,
}

impl Workspace {
    /// Open and canonicalise the sandbox root.
    pub fn new(root: impl AsRef<Path>) -> Result<Self> {
        let requested = root.as_ref();
        let root = requested
            .canonicalize()
            .with_context(|| format!("Unable to open working directory: {:?}", requested))?;
        if !root.is_dir() {
            bail!("Working directory is not a directory: {:?}", root);
        }
        Ok(Self { root })
    }

    /// Canonical sandbox root.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Resolve a path that is expected to exist inside the workspace.
    pub fn resolve_existing(&self, requested: &str) -> Result<PathBuf> {
        let requested = validate_relative(requested)?;
        let candidate =
            self.root.join(requested).canonicalize().with_context(|| {
                format!("Path does not exist or cannot be resolved: {requested}")
            })?;
        if !candidate.starts_with(&self.root) {
            bail!("Path is outside the allowed working directory");
        }
        Ok(candidate)
    }

    /// Resolve a destination path whose parent directory exists inside the
    /// workspace. The leaf file itself may not exist yet.
    pub fn resolve_new_file(&self, requested: &str) -> Result<PathBuf> {
        let requested = validate_relative(requested)?;
        let candidate = self.root.join(requested);
        let parent = candidate
            .parent()
            .ok_or_else(|| anyhow!("Invalid destination path"))?;
        let canonical_parent = parent.canonicalize().with_context(|| {
            format!("Parent directory does not exist or cannot be resolved: {parent:?}")
        })?;
        if !canonical_parent.starts_with(&self.root) {
            bail!("Path is outside the allowed working directory");
        }
        let file_name = candidate
            .file_name()
            .ok_or_else(|| anyhow!("Invalid destination file name"))?;
        Ok(canonical_parent.join(file_name))
    }

    /// Render a workspace path relative to the root for display.
    pub fn relative(&self, path: &Path) -> String {
        path.strip_prefix(&self.root)
            .unwrap_or(path)
            .display()
            .to_string()
    }
}

/// Validate a user/model supplied path before it is joined to the root.
fn validate_relative(requested: &str) -> Result<&str> {
    let requested = requested.trim();
    if requested.is_empty() {
        bail!("Empty file path");
    }
    if requested.starts_with('/') || requested.starts_with('~') {
        bail!("Absolute and home-directory paths are not allowed");
    }
    for component in Path::new(requested).components() {
        match component {
            Component::ParentDir => {
                bail!("Parent-directory components ('..') are not allowed");
            }
            Component::RootDir | Component::Prefix(_) => {
                bail!("Absolute paths are not allowed");
            }
            Component::CurDir | Component::Normal(_) => {}
        }
    }
    Ok(requested)
}
