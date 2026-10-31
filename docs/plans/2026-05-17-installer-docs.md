# Installer Distribution and Documentation Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Resolve GitHub issues #19 and #20 by replacing GitHub raw installer guidance with an OSS/CDN-ready distribution convention and adding a coherent documentation experience to the existing site.

**Architecture:** Keep the Rust CLI at the repository root and the web experience in `site/`. Treat installer distribution as release documentation plus a reusable URL contract, and treat docs as a first-class route group in the existing TanStack Start app so landing and docs share copy, navigation, and styles.

**Tech Stack:** Rust CLI, POSIX shell installer, TanStack Start, React, TypeScript, Vite, CSS, Fumadocs-compatible documentation structure.

---

### Task 1: Document the Installer Distribution Contract

**Files:**
- Modify: `README.md`
- Modify: `docs/release.md`
- Modify: `CHANGELOG.md`

**Steps:**

1. Add an OSS/CDN installer URL convention with one stable script path:
   - `https://<oss-or-cdn-domain>/ocvm/install.sh`
2. Document repeatable automation as `curl .../install.sh | OCVM_VERSION=v0.1.1 sh`.
3. Keep GitHub raw as a maintainer fallback, not the primary user-facing install path.
4. Document `OCVM_INSTALLER_BASE_URL` as the release-time prefix maintainers publish to object storage.
4. Update changelog with user-facing installer guidance changes.
5. Verify with `rg -n "raw.githubusercontent.com|OCVM_INSTALLER_BASE_URL|oss-or-cdn" README.md docs site install.sh CHANGELOG.md`.

### Task 2: Add Shared Site Content for Install and Docs Navigation

**Files:**
- Create: `site/src/content.ts`
- Modify: `site/src/routes/index.tsx`

**Steps:**

1. Extract current release, installer base URL, install command, repo URL, release URL, docs URL, and star label into `site/src/content.ts`.
2. Use the shared install command on the landing page.
3. Add a GitHub star link in the top navigation with a static accessible label.
4. Point Docs navigation to `/docs` instead of a GitHub blob URL.
5. Verify TypeScript imports by running `npm run build --prefix site`.

### Task 3: Add Documentation Routes

**Files:**
- Create: `site/src/routes/docs.tsx`
- Create: `site/src/routes/docs.installation.tsx`
- Create: `site/src/routes/docs.commands.tsx`
- Create: `site/src/routes/docs.configuration.tsx`
- Create: `site/src/routes/docs.release-sources.tsx`

**Steps:**

1. Build `/docs` as an overview page with links to the four documentation pages.
2. Add install docs covering the OSS/CDN convention and local bin setup.
3. Add command docs matching the README command surface.
4. Add configuration docs for `OCVM_HOME`, `OCVM_NPM_PACKAGE`, `OCVM_SOURCE`, and release manifest URL.
5. Add release-source docs for npm and manifest source behavior.
6. Keep docs content concise and current.
7. Verify route generation/build with `npm run build --prefix site`.

### Task 4: Unify Landing and Docs Styling

**Files:**
- Modify: `site/src/styles.css`

**Steps:**

1. Add shared top navigation styles that work for both landing and docs pages.
2. Add docs layout styles with readable width, sidebar-like page index, and code blocks.
3. Keep card radius at 8px or less and avoid nested cards.
4. Check responsive rules for mobile and desktop.
5. Verify visually with a local site server and browser screenshots when feasible.

### Task 5: Verify Repository Gates

**Commands:**

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
npm run build --prefix site
cargo build --release
cargo package --allow-dirty
rg --hidden --no-ignore -n "private-token|secret|internal-domain.example|HOME_PATH_PLACEHOLDER" . --glob '!target/**' --glob '!.git/**' --glob '!.omx/**' --glob '!AGENTS.md' --glob '!CONTRIBUTING.md' --glob '!SECURITY.md'
```

If network or package-cache constraints block the site build, record the exact command and failure output, then run all checks that do not require new network access.

### Task 6: Publish the Branch

**Steps:**

1. Inspect `git status --short --branch`.
2. Review changed files for generated output, secrets, and machine paths.
3. Commit with a concise conventional commit message ending with:

```text
Generated with Codex

Co-Authored-By: Codex <noreply@openai.com>
```

4. Push the branch.
5. Create or reuse a GitHub Pull Request.
6. Include validation commands and remaining risks in the PR summary.
