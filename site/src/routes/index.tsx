import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/')({ component: Home })

const currentRelease = 'v0.1.1'
const installCommand = `curl -fsSL \\
  https://raw.githubusercontent.com/PerfectPan/ocvm/${currentRelease}/install.sh | sh`

const features = [
  {
    title: 'Project pinning',
    body: 'Commit .openclaw-version and every shell in the repo resolves the same OpenClaw version before falling back to user defaults.',
  },
  {
    title: 'Stable and exact installs',
    body: 'Install stable, latest, nightly, or an exact version from npm today, with release manifests ready for GitHub Release assets.',
  },
  {
    title: 'Snapshot and rollback',
    body: 'Capture current default and session metadata before upgrades, then rollback that state without deleting installed versions.',
  },
  {
    title: 'Doctor checks',
    body: 'Inspect PATH, shims, defaults, project pins, and installed executables when a local OpenClaw environment behaves strangely.',
  },
  {
    title: 'Release verification',
    body: 'Staged installs validate openclaw --version, and release manifests can verify executable SHA-256 before activation.',
  },
  {
    title: 'Docker E2E',
    body: 'The npm OpenClaw install path is tested in Docker so verification does not mutate the host environment.',
  },
]

function Home() {
  return (
    <main>
      <section className="hero">
        <nav className="topbar" aria-label="Primary">
          <a className="brand" href="https://github.com/PerfectPan/ocvm">
            ocvm
          </a>
          <div className="navlinks">
            <a href="https://github.com/PerfectPan/ocvm/releases">Releases</a>
            <a href="https://github.com/PerfectPan/ocvm/blob/main/docs/release.md">
              Docs
            </a>
          </div>
        </nav>

        <div className="heroGrid">
          <div className="heroCopy">
            <p className="eyebrow">OpenClaw Version Manager</p>
            <h1>Reproducible OpenClaw versions per project.</h1>
            <p className="lede">
              ocvm is an nvm-style CLI for installing OpenClaw, pinning project
              versions, switching quickly, and rolling back local environment
              metadata when an upgrade goes wrong.
            </p>
            <div className="actions">
              <a className="primaryAction" href="#install">
                Install
              </a>
              <a
                className="secondaryAction"
                href="https://github.com/PerfectPan/ocvm"
              >
                GitHub
              </a>
            </div>
          </div>

          <aside className="terminal" aria-label="ocvm install workflow">
            <div className="terminalBar">
              <span />
              <span />
              <span />
            </div>
            <pre>{`$ ocvm install stable
resolved openclaw@latest
verified: openclaw --version
activated: ~/.ocvm/versions/2026.3.28

$ ocvm pin 2026.3.28
wrote .openclaw-version

$ ocvm exec -- openclaw --version
OpenClaw 2026.3.28`}</pre>
          </aside>
        </div>
      </section>

      <section id="install" className="installBand">
        <div className="installText">
          <p className="sectionLabel">Install</p>
          <h2>Install the current pinned release.</h2>
          <p>
            The installer downloads the {currentRelease} GitHub Release asset
            for your platform, verifies the checksum when one is available, and
            installs <code>ocvm</code> into your local bin directory.
          </p>
          <div className="installActions">
            <a
              href={`https://github.com/PerfectPan/ocvm/releases/tag/${currentRelease}`}
            >
              View release
            </a>
            <a href="https://github.com/PerfectPan/ocvm/blob/main/docs/release.md">
              Release guide
            </a>
          </div>
        </div>
        <pre className="installCommand">
          <code>{installCommand}</code>
        </pre>
      </section>

      <section className="features" aria-label="ocvm features">
        {features.map((feature) => (
          <article className="feature" key={feature.title}>
            <h3>{feature.title}</h3>
            <p>{feature.body}</p>
          </article>
        ))}
      </section>

      <section className="workflow">
        <div>
          <p className="sectionLabel">Daily flow</p>
          <h2>Install once, pin where it matters, verify before switching.</h2>
        </div>
        <ol>
          <li>
            <strong>Install</strong>
            <span>Use npm or release manifests as the source of OpenClaw builds.</span>
          </li>
          <li>
            <strong>Pin</strong>
            <span>Commit `.openclaw-version` for each repo that needs stability.</span>
          </li>
          <li>
            <strong>Recover</strong>
            <span>Snapshot before upgrades and rollback metadata if needed.</span>
          </li>
        </ol>
      </section>
    </main>
  )
}
