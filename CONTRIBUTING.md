# Contributing

## Development Setup

Install stable Rust with rustfmt and clippy, then run:

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Run the local CLI:

```bash
cargo run -- --help
```

## Contribution Flow

1. Open an issue or discussion for ambiguous work.
2. Write an RFC for substantial changes.
3. Create a focused branch.
4. Add or update tests for behavior changes.
5. Update `CHANGELOG.md` for user-facing changes.
6. Run fmt, clippy, and tests.
7. Open a pull request with motivation, implementation notes, validation, and follow-up risks.

Small typo corrections, narrow documentation fixes, and repository metadata updates do not need an RFC.

## When to Write an RFC

Use `rfcs/` when a change affects:

- public CLI behavior
- install or rollback safety
- source-provider trust boundaries
- configuration shape
- release process
- repository structure
- long-term integration strategy

RFCs should describe the problem, goals, non-goals, proposed design, alternatives, rollout plan, and risks.

## Pull Request Expectations

Every PR should answer:

- What changed?
- Why is this change needed?
- How was this tested?
- Are there follow-up tasks or risks?

For release-facing changes, include the output summary from:

```bash
cargo package --allow-dirty
```

## Repository Hygiene

Do not commit private tokens, local config, generated workspaces, internal hostnames, or personal filesystem paths.

Keep package contents intentional. If a file should ship in the crate, verify it appears in `cargo package --list --allow-dirty`.

## Security Reports

Use `SECURITY.md` for vulnerability reporting guidance. Do not include secrets, exploit details, or private infrastructure in public issues or pull requests.
