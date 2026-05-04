# ocvm Landing Page Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add and deploy a TanStack Start landing page for ocvm.

**Architecture:** Keep the web app in `site/` so the Rust CLI stays the repository root. Use a static single-page TanStack Start app with Vercel configuration inside `site/`.

**Tech Stack:** TanStack Start, React, Vite, TypeScript, Vercel, Docker for build verification.

---

### Task 1: Scaffold the TanStack Start Site

**Files:**
- Create: `site/package.json`
- Create: `site/tsconfig.json`
- Create: `site/vite.config.ts`
- Create: `site/src/router.tsx`
- Create: `site/src/routes/__root.tsx`
- Create: `site/src/routes/index.tsx`
- Create: `site/src/styles.css`

**Steps:**

1. Create a minimal TanStack Start app under `site/`.
2. Keep dependencies local to `site/package.json`.
3. Add scripts for `dev`, `build`, and `start`.
4. Implement one route that renders the landing page and imports local CSS.

### Task 2: Add Product Page Content

**Files:**
- Modify: `site/src/routes/index.tsx`
- Modify: `site/src/styles.css`

**Steps:**

1. Add hero content with install command and links to GitHub Releases and docs.
2. Add feature sections for pinning, installs, rollback, doctor, release verification, and Docker E2E.
3. Keep copy concise and specific to ocvm.
4. Use responsive CSS with stable spacing and no decorative card nesting.

### Task 3: Configure Vercel and Documentation

**Files:**
- Create: `site/vercel.json`
- Modify: `README.md`
- Modify: `CHANGELOG.md`

**Steps:**

1. Configure Vercel to deploy `site/` as the project root.
2. Document the landing page location and local commands.
3. Add a changelog entry for the landing page.

### Task 4: Verify and Publish

**Commands:**
- `docker run --rm -v "$PWD":/work -w /work/site node:22-bookworm sh -lc "npm ci && npm run build"`
- `cargo fmt --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test`
- `cargo build --release`
- `cargo package --allow-dirty`
- `rg --hidden --no-ignore -n "private-token|secret|internal-domain.example|HOME_PATH_PLACEHOLDER" . --glob '!target/**' --glob '!.git/**' --glob '!.omx/**' --glob '!AGENTS.md' --glob '!CONTRIBUTING.md' --glob '!SECURITY.md' --glob '!docs/plans/**' --glob '!site/package-lock.json' --glob '!site/node_modules/**' --glob '!site/.vercel/**'`

**Steps:**

1. Run the site build inside Docker.
2. Run the Rust repository gates because the repository remains publishable.
3. Run the publish-safety scan.
4. Run visual QA against the built or previewed site.
5. Commit, push, open a PR, and deploy a Vercel preview.
