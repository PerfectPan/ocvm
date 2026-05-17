# Changelog

## Unreleased

- Replaced GitHub raw installer guidance with a static installer path that can be mirrored to OSS/CDN.
- Added website documentation routes for installation, commands, configuration, and release sources.
- Added GitHub and documentation navigation to the landing page.
- Updated landing page and crate metadata for `https://ocvm.vercel.app`.
- Clarified the site Install section around the pinned `v0.1.1` release.
- Added a GitHub Actions site build gate for the Vercel app in `site/`.
- Added repository-root Vercel build configuration for GitHub deployments.

## 0.1.0

- Initial Rust CLI project for `ocvm`.
- Added MVP commands for install, uninstall, use, default, current, list, list-remote, pin, unpin, exec, and doctor.
- Added npm and release-manifest source providers.
- Added stable-to-latest npm fallback when a stable dist-tag is not present.
- Added shell init helpers for zsh, bash, and fish.
- Added snapshot and rollback metadata commands.
- Added staged executable SHA256 verification for release manifests.
- Added GitHub Release workflow, install script, and Docker e2e harness.
- Added a TanStack Start landing page with Vercel deployment configuration.
- Added staging install flow with `openclaw --version` verification and backup restore when replacing an existing version.
- Added project pin resolution through `.openclaw-version`.
- Added CI gates for fmt, clippy, and tests.
