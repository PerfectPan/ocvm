import { Outlet, createFileRoute } from '@tanstack/react-router'

import { repositoryUrl } from '../content'

export const Route = createFileRoute('/docs')({ component: DocsLayout })

function DocsLayout() {
  return (
    <main className="docsPage">
      <DocsTopbar />
      <Outlet />
    </main>
  )
}

export function DocsTopbar() {
  return (
    <nav className="topbar docsTopbar" aria-label="Documentation">
      <a className="brand" href="/">
        ocvm
      </a>
      <div className="navlinks">
        <a href="/docs">Manual</a>
        <a href="/docs/installation">Install</a>
        <a href={repositoryUrl}>GitHub</a>
      </div>
    </nav>
  )
}
