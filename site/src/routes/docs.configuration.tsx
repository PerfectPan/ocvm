import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/docs/configuration')({
  component: ConfigurationDocs,
})

function ConfigurationDocs() {
  return (
      <article className="docsArticle">
        <p className="sectionLabel">Configuration</p>
        <h1>Configure local state and source providers.</h1>
        <p>
          ocvm keeps defaults small: local state under <code>~/.ocvm</code>, npm
          as the default OpenClaw source, and release manifests available for
          controlled distribution.
        </p>
        <dl className="configList">
          <dt>OCVM_HOME</dt>
          <dd>Redirects all ocvm state, including config, versions, shims, cache, logs, and snapshots.</dd>
          <dt>OCVM_NPM_PACKAGE</dt>
          <dd>Overrides the npm package name used by the npm source provider. The default is <code>openclaw</code>.</dd>
          <dt>OCVM_SOURCE</dt>
          <dd>Set to <code>release</code> to use a release manifest instead of npm metadata.</dd>
          <dt>OCVM_RELEASE_MANIFEST_URL</dt>
          <dd>Points the release source provider at a JSON manifest with version, tag, tarball, and optional checksum metadata.</dd>
        </dl>
      </article>
  )
}
