# ocvm site

TanStack Start landing page for the OpenClaw Version Manager.

Production URL: https://ocvm.vercel.app

## Development

Use Node.js 26.x, version 26.13.1 or later. The site manifest defines this range for npm and CI.

```bash
npm ci
npm run dev
```

## Build

```bash
npm run build
```

The production build uses Nitro's Vercel preset and emits `.vercel/output`.
Vercel selects Node.js 26.x for builds and functions and manages patch updates.

## Docker Verification

```bash
docker run --pull=always --rm -u "$(id -u):$(id -g)" \
  -e npm_config_cache=/tmp/npm-cache \
  -v "$PWD/..":/work \
  -w /work/site \
  node:26-bookworm sh -lc "npm ci && npm run build"
```

## Deploy

Deploy the `site/` directory with Vercel. The site-specific `vercel.json` uses
`npm ci` and `npm run build`.

For GitHub-backed automatic deployments:

- Import or connect `PerfectPan/ocvm` in Vercel.
- Use `main` as the production branch.
- Assign `ocvm.vercel.app` to the project production domain.
- Keep the repository-root `vercel.json` when the Vercel project is connected
  at the repository root. It runs the site install/build commands and copies
  `site/.vercel/output` to the root `.vercel/output` expected by Vercel.
- If the Vercel project Root Directory is changed to `site`, the site-local
  `vercel.json` is enough and the root build wrapper is not used.
- Keep the GitHub `CI` workflow's `site` job required so proposed changes
  exercise the same site build command before merge.
