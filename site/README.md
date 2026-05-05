# ocvm site

TanStack Start landing page for the OpenClaw Version Manager.

Production URL: https://ocvm.vercel.app

## Development

```bash
npm ci
npm run dev
```

## Build

```bash
npm run build
```

The production build uses Nitro's Vercel preset and emits `.vercel/output`.

## Docker Verification

```bash
docker run --rm -u "$(id -u):$(id -g)" \
  -e npm_config_cache=/tmp/npm-cache \
  -v "$PWD/..":/work \
  -w /work/site \
  node:22-bookworm sh -lc "npm ci && npm run build"
```

## Deploy

Deploy the `site/` directory with Vercel. The site-specific `vercel.json` uses
`npm ci` and `npm run build`.

For GitHub-backed automatic deployments:

- Import or connect `PerfectPan/ocvm` in Vercel.
- Set the Vercel project Root Directory to `site`.
- Use `main` as the production branch.
- Assign `ocvm.vercel.app` to the project production domain.
- Keep the GitHub `CI` workflow's `site` job required so proposed changes
  exercise the same site build command before merge.
