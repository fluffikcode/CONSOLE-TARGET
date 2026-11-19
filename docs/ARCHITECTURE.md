# Architecture

TARGET is deliberately small, but it separates policy from mechanism so the
pieces can be tested without a model or a terminal.

## Layers

| Layer | Module | Responsibility |
| --- | --- | --- |
| Entry point | `main.rs` | Parse arguments, build the agent, run the REPL |
| CLI | `cli.rs` | Declarative `clap` surface |
| Configuration | `config.rs` | Validate arguments, canonicalise the workdir, read the API key |
| Orchestration | `agent` | Transcript, model turns, tool-call dispatch, step limits |
| Provider | `llm.rs` | HTTP client for `/chat/completions` |
| Transcript | `message.rs` | System/user/assistant/tool messages and serialization |
| Prompt | `prompt.rs` | The agent's behavioural contract |
| Path policy | `sandbox.rs` | Confine every path to one canonical root |
| Consent | `approval.rs` | `Approver` trait and `WriteRequest` |
| Capabilities | `tools` | Tool schemas and implementations |
| Presentation | `ui` | Theme, banner, dashboard, output, spinner, terminal approver |

Dependencies point inward:

```text
main ──> agent ──> llm
            ├────> tools ──> sandbox
            │        └────> approval
            ├────> message
            ├────> prompt
            └────> ui
```

Tools do not know about the model client. The UI does not know about tools. The
agent is the only component that sees both sides.

## Request lifecycle

1. `main` parses `Args`, builds `Settings`, and constructs an `Agent` with a
   `TerminalApprover`.
2. `Agent::run_task` appends the user message and enters the turn loop.
3. `Agent::call_model` asks `LlmClient::complete` for the next assistant turn.
   The transcript and the registry's JSON schemas are sent with the request.
4. If the reply contains no `tool_calls`, its text is returned to the REPL and
   rendered by the UI.
5. If it contains tool calls, each one is parsed, announced in the UI, executed
   through `ToolRegistry`, and appended as a `tool` message correlated by
   `tool_call_id`. The loop repeats.
6. The loop stops on the first non-empty final answer or when `max_steps` is
   exhausted.

Malformed tool arguments and tool execution errors are not fatal: they are
returned to the model as observations so it can recover.

## Path-safety pipeline

All filesystem access goes through `Workspace`:

```text
model-provided string
  ├─ empty? ──────────────────────────────> reject
  ├─ starts with "/" or "~"? ─────────────> reject
  ├─ contains ".." / RootDir / Prefix? ───> reject
  ├─ join(root, path)
  ├─ canonicalize (existing) or parent (new file)
  ├─ starts_with(root)? ──────────────────> reject if not
  └─ hand the canonical path to the tool
```

Canonicalising after checking `..` is important: the textual check gives a clear
error, while canonicalisation catches symlinks and platform-specific path
aliases. New-file resolution canonicalises the parent and keeps only the leaf
file name, so a non-existent destination cannot escape the root.

## Consent pipeline

`write_file` never calls `fs::write` on its own:

```text
tool call
  └─> Workspace::resolve_new_file
        └─> Approver::approve_write(WriteRequest { path, relative_path, content })
              ├─ TerminalApprover: preview + y/N on stdin
              ├─ test ApproveAll / DenyAll
              └─ false ──> tool error, no filesystem mutation
```

Because approval is a trait, the agent can be driven headlessly in tests or
embedded behind a different UI without changing the tool.

## Adding a tool

1. Create `src/tools/<name>.rs` with a unit struct implementing `Tool`.
2. Return a JSON Schema from `parameters()` and implement `execute()` using
   `ctx.workspace` for every path. Never call `canonicalize` or `fs` directly on
   an unvalidated path.
3. Register the module and type in `src/tools/mod.rs`, then add it to
   `ToolRegistry::standard()`.
4. Add a behavioral test in `tests/tools.rs`, including a traversal attempt if
   the tool accepts paths.
5. Document the tool in the README table.

## Swapping the UI or provider

- **UI:** anything implementing `Approver` can replace `TerminalApprover`. The
  print helpers in `ui` are free functions; an embedding application can call
  `Agent::run_task` directly instead of the REPL.
- **Provider:** `LlmClient` is the only module that knows the HTTP shape of the
  API. Keep `AssistantTurn` raw (`serde_json::Value` for `tool_calls`) if you
  add another provider, so provider-specific fields can be replayed verbatim.

## Testing strategy

- `tests/sandbox.rs` exercises path policy: relative resolution, traversal,
  absolute and home paths, root validation, and new-file parents.
- `tests/tools.rs` drives the public `ToolRegistry` API with deterministic
  approvers, covering listing, reading, size limits, search, approved writes,
  denied writes, traversal, and unknown tools.
- Unit-level behavior that depends on the network or a live terminal is kept
  out of the test suite; the boundaries are injectable instead.
