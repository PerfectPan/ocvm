# ocvm Landing Page Design

## Goal

Build a small TanStack Start landing page for ocvm that explains what the tool does, how to install it, and where to find release and project documentation.

## Approach

Use a `site/` app inside the repository so the CLI crate remains the root project and the web app has its own package boundary. Deploy the `site/` directory as the Vercel project root, and keep the page static so it can ship as a low-risk preview before a production domain exists.

## Page Content

The first viewport should make the product clear: ocvm is a version manager for OpenClaw. It should include an install command, a short description of project-local version pinning, and links to GitHub Releases and repository documentation.

Below the first viewport, include compact sections for:

- project pinning with `.openclaw-version`
- channel and exact-version installs
- snapshot and rollback
- doctor and release safety checks
- Docker-backed verification for real npm installs

## Architecture

The site will use TanStack Start with React and Vite. It will have a single route, local CSS, and no server data dependencies. The site-local Vercel config will use the TanStack Start Nitro Vercel preset.

## Testing

Verification should avoid installing or executing packages globally. Run dependency install and build inside Docker where possible, then run a local preview only if needed for visual QA. Capture visual QA with a browser screenshot against the built page.
