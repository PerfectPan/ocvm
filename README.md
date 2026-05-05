# ocvm

`ocvm` is the OpenClaw Version Manager: a local, `nvm`-style CLI for installing multiple OpenClaw versions, pinning project versions, switching quickly, executing a specific version, and diagnosing environment problems.

This is a version manager, not an OpenClaw installer replacement. Installation is the mechanism; reproducible local OpenClaw environments are the product.

Website: https://ocvm.vercel.app

## Status

MVP crate scaffold with the full command surface from the PRD:

```bash
ocvm install <version|latest|stable>
ocvm uninstall <version>
ocvm use <version>
ocvm default <version>
ocvm current
ocvm list
ocvm list-remote
ocvm list-remote --channel stable
ocvm list-remote --channel nightly
ocvm pin <version>
ocvm unpin
ocvm init <zsh|bash|fish>
ocvm snapshot [name]
ocvm rollback [name]
ocvm exec [version] -- <command>
ocvm doctor
```

## Tech Stack

`ocvm` is implemented as a Rust CLI because this category wants native distribution, fast startup, predictable packaging, and minimal runtime dependencies.

Core crates:

- `clap` for command parsing
- `serde` / `serde_json` for config and release manifests
- `reqwest` with `rustls` for release manifest fetching
- `anyhow` for application errors
- `assert_cmd`, `predicates`, and `tempfile` for CLI integration tests

## Architecture

```text
src/
  main.rs       binary entrypoint
  cli.rs        clap command parsing and command dispatch
  config.rs     ~/.ocvm/config.json load/save
  paths.rs      OCVM_HOME and layout helpers
  project.rs    .openclaw-version pin lookup
  source.rs     npm/release source providers
  version.rs    install, switch, exec, doctor, resolution
tests/
  cli.rs        black-box CLI behavior
  core.rs       install/source/resolution behavior with fake provider
```

The source layer is a trait, so npm, release manifests, GitHub Releases, mirrors, checksums, and future source policies can evolve without rewriting command behavior.

## Directory Layout

By default, `ocvm` stores state under `~/.ocvm`:

```text
~/.ocvm/
  config.json
  versions/
    2026.3.28/
    2026.4.01/
  shims/
    openclaw
  cache/
  logs/
  snapshots/
```

Use `OCVM_HOME` to redirect all state, especially in tests and CI:

```bash
export OCVM_HOME="$PWD/.ocvm"
```

## Version Resolution

`ocvm` resolves versions in this order:

1. Explicit version in `ocvm exec <version> -- ...`
2. Session version written by `ocvm use <version>`
3. Nearest `.openclaw-version` in the current directory or parent directories
4. Global default in `~/.ocvm/config.json`

Project pins are plain text:

```text
.openclaw-version
```

Commit that file to make team behavior reproducible.

## Source Providers

The default source is npm:

```bash
export OCVM_NPM_PACKAGE="openclaw"
```

Release manifest source is also supported:

```bash
export OCVM_SOURCE=release
export OCVM_RELEASE_MANIFEST_URL="https://releases.example.com/openclaw.json"
```

Manifest format:

```json
{
  "versions": [
    {
      "version": "2026.3.28",
      "tags": ["stable"],
      "url": "https://releases.example.com/openclaw-2026.3.28.tgz"
    }
  ]
}
```

The release provider also accepts a raw array of version objects. Tarball fields can be named `url`, `tarball`, or `npm`.

The npm provider maps `stable` to the `latest` dist-tag when npm does not define a separate `stable` dist-tag. Project config can override this with `channels.stable`.

Release manifest entries can include `sha256` or `executableSha256`. When present, `ocvm` verifies the staged `openclaw` executable before activation.

## Shell Setup

Add the shim directory to `PATH`:

```bash
export PATH="$HOME/.ocvm/shims:$PATH"
```

`ocvm use <version>` cannot mutate a parent shell directly, so it prints a shell command:

```bash
ocvm use 2026.3.28
```

Then run the printed `export PATH=...` command in your shell.

For helper functions:

```bash
ocvm init zsh
ocvm init bash
ocvm init fish
```

For example, zsh users can add this to their shell profile:

```bash
eval "$(ocvm init zsh)"
```

Then use:

```bash
ocvm-use 2026.3.28
```

## Install Safety

`ocvm install` uses a staging directory, verifies:

```bash
openclaw --version
```

and only then activates the installed version under `versions/<version>`. If replacing an existing version fails, the previous version is restored from a backup directory.

## Snapshots

Capture the current default and session version metadata:

```bash
ocvm snapshot before-upgrade
```

Rollback metadata later:

```bash
ocvm rollback before-upgrade
```

Rollback does not delete unrelated installed versions.

## Install

After a GitHub Release exists:

```bash
curl -fsSL https://raw.githubusercontent.com/PerfectPan/ocvm/v0.1.1/install.sh | sh
```

## Docker E2E

Run real npm OpenClaw install validation in Docker instead of on the host:

```bash
./scripts/e2e-docker.sh
```

## Landing Page

The TanStack Start landing page is deployed at https://ocvm.vercel.app. Source lives in `site/` and is configured for Vercel deployments from that directory:

```bash
npm ci --prefix site
npm run build --prefix site
```

For automatic Vercel deployments from GitHub, connect the Vercel project to
`PerfectPan/ocvm` and track `main` as the production branch. The root
`vercel.json` installs and builds the `site/` app, then places Nitro's Vercel
output where Vercel expects it for a repository-root project. Pull requests
receive preview deployments; merges to `main` publish production deployments
for `ocvm.vercel.app`.

## Development

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

The CI workflow runs the same gates on stable Rust.

Release documentation is in [docs/release.md](./docs/release.md).

Contribution workflow is documented in [CONTRIBUTING.md](./CONTRIBUTING.md). AI delivery and repository hygiene rules are documented in [AGENTS.md](./AGENTS.md), with a Claude-specific entrypoint in [CLAUDE.md](./CLAUDE.md). Security reporting guidance lives in [SECURITY.md](./SECURITY.md).
