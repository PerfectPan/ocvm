import { createFileRoute } from '@tanstack/react-router'

import {
  currentRelease,
  installCommand,
  pinnedInstallCommand,
  stableInstallerUrl,
} from '../content'

export const Route = createFileRoute('/docs/installation')({
  component: InstallationDocs,
})

function InstallationDocs() {
  return (
      <article className="docsArticle">
        <p className="sectionLabel">Installation</p>
        <h1>Install ocvm from the published installer.</h1>
        <p>
          For users, the preferred install command reads the installer from a
          public OSS or CDN path instead of GitHub raw content.
        </p>
        <pre>
          <code>{installCommand}</code>
        </pre>
        <h2>Release layout</h2>
        <p>Publish one stable installer script:</p>
        <pre>
          <code>{stableInstallerUrl}</code>
        </pre>
        <p>
          Use the same script for normal onboarding and repeatable automation.
          Pin the binary release with <code>OCVM_VERSION</code> instead of
          uploading a new script for every version.
        </p>
        <h2>Install a different release</h2>
        <p>
          The installer defaults to its pinned release, but automation can
          override the binary release version.
        </p>
        <pre>
          <code>{pinnedInstallCommand}</code>
        </pre>
        <pre>
          <code>{`${installCommand.replace(' | sh', ' | OCVM_VERSION=latest sh')}`}</code>
        </pre>
        <h2>Shell setup</h2>
        <p>Add the binary and shim directories to your shell profile:</p>
        <pre>
          <code>{`export PATH="$HOME/.local/bin:$PATH"\nexport PATH="$HOME/.ocvm/shims:$PATH"`}</code>
        </pre>
      </article>
  )
}
