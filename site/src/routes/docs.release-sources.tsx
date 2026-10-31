import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/docs/release-sources')({
  component: ReleaseSourceDocs,
})

function ReleaseSourceDocs() {
  return (
      <article className="docsArticle">
        <p className="sectionLabel">Release sources</p>
        <h1>Choose npm or a release manifest.</h1>
        <p>
          The source layer is isolated from command behavior, so teams can keep
          using npm today and move to signed release manifests when they need
          stricter distribution control.
        </p>
        <h2>Npm source</h2>
        <p>
          The default source reads npm metadata, installs exact versions, and
          maps <code>stable</code> to the <code>latest</code> dist-tag when npm
          does not publish a separate stable tag.
        </p>
        <h2>Release manifest source</h2>
        <p>
          Set <code>OCVM_SOURCE=release</code> and
          <code> OCVM_RELEASE_MANIFEST_URL </code>
          to fetch a manifest from controlled infrastructure.
        </p>
        <pre>
          <code>{`{
  "versions": [
    {
      "version": "2026.3.28",
      "tags": ["stable"],
      "url": "https://releases.example.com/openclaw-2026.3.28.tgz",
      "executableSha256": "..."
    }
  ]
}`}</code>
        </pre>
        <p>
          When a manifest entry includes <code>sha256</code> or
          <code> executableSha256</code>, ocvm verifies the staged executable
          before activation.
        </p>
      </article>
  )
}
