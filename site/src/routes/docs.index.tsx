import { createFileRoute } from '@tanstack/react-router'

import {
  currentRelease,
  docLinks,
  installCommand,
  pinnedInstallCommand,
} from '../content'

export const Route = createFileRoute('/docs/')({ component: DocsHome })

function DocsHome() {
  return (
    <>
      <section className="docsHero">
        <p className="sectionLabel">User manual</p>
        <h1>Use ocvm with repeatable OpenClaw versions.</h1>
        <p>
          Start with the installer, then pin versions per project and use
          snapshots before upgrades that may affect local development.
        </p>
      </section>

      <section className="docsGrid" aria-label="Documentation sections">
        {docLinks.map((link) => (
          <a className="docTile" href={link.href} key={link.href}>
            <h2>{link.title}</h2>
            <p>{link.body}</p>
          </a>
        ))}
      </section>

      <section className="docsSection">
        <h2>Installer paths</h2>
        <p>
          The recommended release layout publishes one stable OSS/CDN installer
          script. Pin the binary release with <code>OCVM_VERSION</code> when
          automation needs repeatable installs.
        </p>
        <pre>
          <code>{installCommand}</code>
        </pre>
        <pre>
          <code>{pinnedInstallCommand}</code>
        </pre>
        <p>
          The current pinned installer version is <code>{currentRelease}</code>.
          Maintainers publish these files from release automation by setting the
          OSS or CDN prefix for the release.
        </p>
      </section>
    </>
  )
}
