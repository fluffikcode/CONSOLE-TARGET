# Changelog

All notable changes to this project are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Repository documentation, CI, issue templates, and contribution guidelines.
- Integration tests for the sandbox path policy and all built-in tools.

## [0.1.0] - 2026-11-19

### Added

- Interactive terminal agent for OpenAI-compatible chat-completions endpoints.
- Sandboxed workspace with canonical path checks, traversal rejection, and
  symlink-escape protection.
- Tools: `list_files`, `read_file`, `search_text`, and `write_file`.
- Write interception with a preview and explicit `y`/`N` approval.
- Configurable endpoint, model, workdir, API-key environment variable, and step
  limit.
- Monochrome terminal UI with an animated banner, startup dashboard, tool-call
  output, wait spinner, and framed answers.
- Session commands: `/clear`, `/reset`, `/exit`.
- Layered module structure with public library API and unit/integration tests.
