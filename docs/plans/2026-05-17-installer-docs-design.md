# Installer Distribution and Documentation Design

## Goal

Resolve GitHub issues #19 and #20 by making the installer distribution path resilient outside GitHub raw content and by giving new users a coherent landing and documentation experience.

## Installer Distribution

Common installer hosting should use one public object-storage path:

```text
https://<oss-or-cdn-domain>/ocvm/install.sh
```

The stable `install.sh` path is the recommended copy-paste command for most users. Repeatable automation should pin the binary release with `OCVM_VERSION=v0.1.1` while reusing the same script. This avoids uploading a new installer script for every release unless the script logic or default pinned release changes. The repository should not hard-code an unverified vendor bucket domain. Instead, docs and site copy should expose `OCVM_INSTALLER_BASE_URL` as the release-time value that maintainers set to their OSS or CDN prefix.

The install script itself continues to resolve binaries from GitHub Releases unless `OCVM_REPO`, `GITHUB_API_URL`, or future release-source variables override that behavior. Issue #19 is specifically about avoiding GitHub raw content for downloading the script, not about changing release asset storage in this pass.

## Documentation Site

The existing `site/` app remains the single web project. It should add a `/docs` area that shares navigation, visual language, and install commands with the landing page. The docs area should be content-first and lightweight: installation, command reference, configuration, and release source pages are enough for the current CLI surface.

Fumadocs is the target documentation framework. Because the current app is TanStack Start on Vite, integration should follow Fumadocs' Vite/TanStack path where practical. If package constraints make full Fumadocs UI integration risky in this pass, the repository should still adopt a docs route/content boundary that can be migrated cleanly to Fumadocs components later.

## GitHub Stars

The landing page should show GitHub star visibility without making rendering depend on a live GitHub API request. The first implementation can use a static repository stat value with a direct GitHub link and accessible label. A future enhancement can hydrate the value from GitHub with caching once a server data-loading convention is chosen.

## Validation

Run the Rust checks because this remains a publishable CLI repository. Run the site build because the web app changes user-facing behavior. Run a publish-safety scan before pushing. If dependency installation or networked package fetches fail, record the exact blocker and verify all offline-capable checks.
