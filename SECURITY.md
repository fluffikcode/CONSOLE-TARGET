# Security Policy

## Supported versions

TARGET is pre-1.0. Security fixes are applied to the latest `0.1.x` release and
to the `main` branch. Older commits are not maintained.

## Reporting a vulnerability

Please do not open a public issue for security problems.

Use the repository's **Security → Advisories → Report a vulnerability** flow on
GitHub so the report stays private. Include:

- the affected version or commit;
- the platform and Rust version;
- a minimal reproduction, if possible;
- the impact you believe is possible.

You can expect an initial acknowledgement within a few days. Please allow time
for a fix and a coordinated disclosure before publishing details.

## Threat model

TARGET is a local agent harness that gives a language model controlled access to
a directory. Its main security goal is to prevent accidental or model-driven
access **outside that directory**, and to keep every mutation behind explicit
human consent.

### Defenses in place

- **Directory confinement.** The workdir is canonicalised at startup. Existing
  paths are canonicalised after joining and must remain under the root, which
  blocks `..` traversal and symlink escapes.
- **Input path rejection.** Empty paths, absolute paths, `~` expansion, parent
  components, and platform path prefixes are rejected before filesystem access.
- **Bounded reads.** File reads are capped at 1,000,000 bytes; traversal depth
  and search results are capped to keep resource use finite.
- **Write consent.** `write_file` displays the destination, size, and a preview,
  and proceeds only on an explicit `y` from the terminal.
- **No secret persistence.** The API key is read from an environment variable and
  used only as a bearer token. It is not added to the transcript or printed.
- **No `unsafe` code.** The crate forbids `unsafe` at the crate root.

### Non-goals and limitations

- **Not an OS sandbox.** TARGET does not use containers, namespaces, seccomp,
  AppArmor, or a VM. A bug in path handling or the process itself is not
  contained by the operating system.
- **No network isolation.** The model endpoint is contacted over the network;
  whatever data you send to it leaves the machine.
- **Prompt injection is not solved.** Content read from files can influence the
  model. The tool policy limits damage, but a model can still propose misleading
  or unwanted (approved-by-you) writes. Review previews before typing `y`.
- **Approval fatigue.** The consent gate is only as strong as the human reading
  the preview. Do not auto-approve unattended runs against important data.
- **Binary and non-UTF-8 files** are skipped by search and rejected by
  `read_file`; this is a reliability boundary, not a confidentiality one.

### Operational guidance

- Run TARGET against a dedicated project directory, not your home directory.
- Use a scoped API key with a spending limit where the provider supports it.
- Keep a clean git state in the target directory so accidental writes are easy
  to inspect and revert.
- Treat model output and file content as untrusted data.
