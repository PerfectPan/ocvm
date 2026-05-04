# Release

`ocvm` publishes native binaries through GitHub Releases.

## Create a Release

1. Ensure the working tree is clean.
2. Run the validation gates:

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo package --allow-dirty
```

3. Create and push a version tag:

```bash
git tag v0.1.0
git push origin v0.1.0
```

The `Release` workflow builds platform binaries, packages archives, generates SHA256 checksum files, and creates a GitHub Release.

## Install Script

Users can install the latest release with:

```bash
curl -fsSL https://raw.githubusercontent.com/PerfectPan/ocvm/main/install.sh | sh
```

For private repository installs, set a token with `repo` access:

```bash
GITHUB_TOKEN=... curl -fsSL https://raw.githubusercontent.com/PerfectPan/ocvm/main/install.sh | sh
```

## Docker E2E

Use Docker to validate real npm OpenClaw install behavior without touching the host:

```bash
./scripts/e2e-docker.sh
```

The container sets `OCVM_HOME=/tmp/ocvm-home`, installs from npm, and runs `ocvm exec -- openclaw --version`.

