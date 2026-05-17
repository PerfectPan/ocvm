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
git tag v0.1.1
git push origin v0.1.1
```

The `Release` workflow builds platform binaries, packages archives, generates SHA256 checksum files, and creates a GitHub Release.

## Install Script

Publish the installer to object storage or a CDN after the GitHub Release is
created. Use one stable script path for onboarding and pin the binary release
with `OCVM_VERSION` when automation needs repeatable installs:

```bash
export OCVM_INSTALLER_BASE_URL="https://ocvm.vercel.app/ocvm"

# Stable installer path for the current release.
curl -fsSL "$OCVM_INSTALLER_BASE_URL/install.sh" | sh

# Repeatable install pinned to a specific binary release.
curl -fsSL "$OCVM_INSTALLER_BASE_URL/install.sh" | OCVM_VERSION=v0.1.1 sh
```

Override `OCVM_VERSION` to install a different release:

```bash
curl -fsSL "$OCVM_INSTALLER_BASE_URL/install.sh" | OCVM_VERSION=latest sh
```

GitHub raw content can remain a maintainer fallback while testing release tags,
but it should not be the primary user-facing install command.

For an OSS bucket, mirror the same files and replace the prefix:

```text
ocvm/install.sh
```

## Docker E2E

Use Docker to validate real npm OpenClaw install behavior without touching the host:

```bash
./scripts/e2e-docker.sh
```

The container sets `OCVM_HOME=/tmp/ocvm-home`, installs from npm, and runs `ocvm exec -- openclaw --version`.
