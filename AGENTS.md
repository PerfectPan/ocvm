# Agent Guidelines

This repository is intended to become a publishable Rust CLI. Treat every change as if it may be reviewed, packaged, indexed, and installed by users.

## Working Rules

- Keep changes scoped to the user request and nearby code.
- Prefer existing module boundaries over new abstractions.
- Do not commit local config, credentials, generated logs, temporary workspaces, or machine-specific paths.
- Do not add private tokens, internal hostnames, private repository names, or personal filesystem paths.
- Use `rg` for searches when available.
- For Rust changes, run the relevant checks before claiming completion:

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

- For release readiness, also run:

```bash
cargo build --release
cargo package --allow-dirty
```

Use `--offline` only when network access is unavailable and the crate cache is sufficient.

## Documentation

- Keep `README.md` focused on orientation, quick start, and current user-facing behavior.
- Use `CONTRIBUTING.md` for contribution workflow.
- Use `docs/` for operational guides and reference material.
- Use `rfcs/` for substantial design proposals.
- Update `CHANGELOG.md` for user-facing changes unless the change is docs-only or repository-only.

## AI Delivery Workflow

When an AI agent completes implementation work:

1. Inspect `git status --short --branch`.
2. Verify generated files, secrets, machine paths, and build artifacts are not staged.
3. Run the required verification gates and record the exact commands.
4. Commit pending changes with a concise conventional commit message.
5. Push the branch and verify the remote head.
6. Create or reuse a GitHub Pull Request when the task is not landing directly on `main`.
7. Include a delivery summary with motivation, implementation notes, validation, and follow-up risks.

## Git

- Branch names should be short and descriptive, such as `feat/release-source`.
- Commit messages should be concise and use conventional prefixes when they fit.
- Signed commits are preferred when local git signing is configured.
- Do not rewrite or discard user changes unless explicitly requested.

## Publish Safety Check

Before pushing public-facing or package-facing changes, scan for accidental private references:

```bash
rg --hidden --no-ignore -n "private-token|secret|internal-domain.example|HOME_PATH_PLACEHOLDER" . \
  --glob '!target/**' \
  --glob '!.git/**' \
  --glob '!.omx/**' \
  --glob '!AGENTS.md'
```
