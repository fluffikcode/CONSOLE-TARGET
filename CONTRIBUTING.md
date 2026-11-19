# Contributing to TARGET

Thanks for taking the time to contribute. This project favours small, focused
changes with tests and a clear rationale.

## Before you start

- Open an issue for non-trivial features or refactors so the design can be
  discussed first.
- For security issues, follow [`SECURITY.md`](SECURITY.md) instead of opening a
  public issue.
- Be respectful; see [`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md).

## Development setup

```bash
git clone <your-fork-url>
cd target
rustup toolchain install stable --component rustfmt,clippy
cargo build
```

Rust 1.75 or newer is required (see `rust-version` in `Cargo.toml`).

## Checks

Run all checks before pushing:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets
cargo doc --no-deps
```

CI runs formatting and clippy on Linux and tests on Linux, macOS, and Windows.
A change that passes locally on only one platform may still fail CI.

## Code guidelines

- Keep the layering described in [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md).
  Filesystem access belongs in `tools` and must go through `Workspace`.
- Never bypass `Workspace` or `Approver` with direct path handling.
- Keep tools synchronous; only the model call is async.
- Prefer explicit, testable functions over large `main` routines.
- Document public items with `///` doc comments.
- Add integration tests for new tools, including denial and traversal cases
  where relevant.
- Do not add `unsafe` code. The crate forbids it.

## Adding a tool

1. Implement the `Tool` trait in a new `src/tools/<name>.rs`.
2. Register it in `src/tools/mod.rs` and in `ToolRegistry::standard()`.
3. Add behavioral tests to `tests/tools.rs`.
4. Update the tools table in `README.md` and the limits in `docs/ARCHITECTURE.md`.

## Commits

Use short, imperative subject lines:

```text
add search_text result limit test
fix symlink escape in resolve_new_file
docs: explain the approval pipeline
```

Keep unrelated changes in separate commits. Reference issues in the body when
applicable.

## Pull requests

A pull request should:

- explain the problem and the chosen approach;
- note user-visible changes;
- include or update tests;
- pass all checks above;
- update documentation when behavior or configuration changes.

The pull request template mirrors this checklist. Small PRs are reviewed faster.
