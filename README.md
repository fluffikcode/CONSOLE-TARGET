# TARGET

**A local, sandboxed LLM file-analysis agent.**

TARGET is an interactive command-line harness that connects any OpenAI-compatible
chat-completions endpoint to a small, audited set of file tools. The model can
inspect one working directory, search it, read UTF-8 text, and propose writes —
but every write is intercepted, previewed, and requires explicit human approval.

![license: MIT](https://img.shields.io/badge/license-MIT-blue.svg)
![rust: 1.75+](https://img.shields.io/badge/rust-1.75%2B-orange.svg)

---

## Highlights

- **Provider-agnostic.** Works with OpenAI, Ollama, LM Studio, vLLM, and any other
  endpoint that implements the OpenAI chat-completions protocol.
- **Directory-confined.** Every path is resolved through a sandbox that rejects
  absolute paths, `~`, `..` traversal, and symlink escapes.
- **Read-only until approved.** `write_file` is consent-gated: the pending file,
  size, and preview are shown before anything touches disk.
- **Small tool surface.** Four tools with explicit JSON schemas, size caps, depth
  caps, and result limits.
- **Layered architecture.** The model client, sandbox, tools, consent gate, and
  terminal UI are separate modules that can be tested and replaced independently.

## Architecture

```text
src/
├── main.rs              # entry point: wire settings → agent → REPL
├── lib.rs               # crate documentation and module exports
├── cli.rs               # clap argument definitions
├── config.rs            # validated runtime settings
├── agent/
│   └── mod.rs           # conversation loop and tool-call orchestration
├── llm.rs               # OpenAI-compatible chat-completions client
├── message.rs           # transcript entries and JSON serialization
├── prompt.rs            # system prompt / agent contract
├── sandbox.rs           # workspace path policy
├── approval.rs          # Approver trait and write-request model
├── tools/
│   ├── mod.rs           # Tool trait, ToolContext, ToolRegistry
│   ├── list_files.rs    # directory listing (depth 3)
│   ├── read_file.rs     # UTF-8 reads (max 1 MB)
│   ├── search_text.rs   # case-insensitive search (100 results)
│   └── write_file.rs    # consent-gated writes
└── ui/
    ├── mod.rs           # UI exports
    ├── theme.rs         # monochrome ANSI palette
    ├── banner.rs        # animated wordmark
    ├── dashboard.rs     # startup telemetry panel
    ├── output.rs        # prompts, tool results, answers, errors
    ├── spinner.rs       # wait spinner
    └── approval.rs      # interactive write confirmation
```

One request flows like this:

```text
user input
  └─> Agent::run_task
        └─> LlmClient::complete ──> model reply
              ├─ no tool calls ──> final answer ──> UI
              └─ tool calls
                    └─> ToolRegistry::execute
                          └─> Workspace path checks ──> filesystem
                                └─> observation ──> transcript ──> next turn
```

See [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) for module contracts, the
path-safety pipeline, and extension points.

## Requirements

- Rust **1.75** or newer (stable toolchain recommended)
- An OpenAI-compatible endpoint that supports tool calling

## Build

From the repository root:

```bash
cargo build --release
```

The binary is written to `target/release/target`. To install it on `PATH`:

```bash
cargo install --path .
```

## Usage

```bash
# Hosted OpenAI-compatible endpoint
export OPENAI_API_KEY="sk-..."
target \
  --base-url https://api.openai.com/v1 \
  --model gpt-4o-mini \
  --workdir ./project
```

```bash
# Local endpoint that does not require a key
target \
  --base-url http://localhost:11434/v1 \
  --model llama3.1 \
  --workdir ./project
```

On PowerShell, set the key with `$env:OPENAI_API_KEY = "sk-..."`.

### Options

| Flag | Default | Description |
| --- | --- | --- |
| `--base-url <URL>` | *(required)* | OpenAI-compatible API base URL |
| `--model <MODEL>` | *(required)* | Model identifier understood by the endpoint |
| `--workdir <DIR>` | `.` | Directory the agent is confined to |
| `--api-key-env <VAR>` | `OPENAI_API_KEY` | Environment variable holding the API key |
| `--max-steps <N>` | `12` | Maximum model turns per user request |

### Session commands

| Command | Effect |
| --- | --- |
| `/clear` | Clear the screen and redraw the banner |
| `/reset` | Drop the conversation history |
| `/exit` | End the session |

`exit`, `clear`, and `reset` (without the slash) are accepted as aliases.

### Tools

| Tool | Access | Limits |
| --- | --- | --- |
| `list_files` | Read | Depth 3 from the requested directory |
| `read_file` | Read | UTF-8 only, 1,000,000 bytes maximum |
| `search_text` | Read | Case-insensitive literal search, 100 matches |
| `write_file` | Write | Explicit `y` confirmation for every write |

## Safety model

- **Sandbox root.** The working directory is canonicalised at startup. All
  existing paths are canonicalised again and checked against that root, so
  symlinks cannot escape it.
- **Path validation.** Absolute paths, home-directory paths, and `..`
  components are rejected before any filesystem access.
- **Read limits.** File size and traversal depth are capped to keep context and
  runtime bounded.
- **Write interception.** Writes are previewed and require a `y` response on
  stdin. Everything else denies.
- **Secrets.** The API key is read from the configured environment variable and
  sent only as a bearer token. It is never written to the transcript or UI.

TARGET is a harness, not an OS sandbox. It does not isolate the process, restrict
network access, or defend against a fully adversarial model that is allowed to
write files. See [`SECURITY.md`](SECURITY.md) for the full threat model.

## Development

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets
cargo doc --no-deps
```

The test suite covers path policy and every built-in tool, including approval
denial and traversal attempts. CI runs the same checks on Linux, macOS, and
Windows.

## Contributing

Contributions are welcome. Please read [`CONTRIBUTING.md`](CONTRIBUTING.md) and
[`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md) before opening a pull request.

## License

Released under the [MIT License](LICENSE).
