import { createFileRoute } from '@tanstack/react-router'

import { commandGroups } from '../content'

export const Route = createFileRoute('/docs/commands')({
  component: CommandDocs,
})

function CommandDocs() {
  return (
      <article className="docsArticle">
        <p className="sectionLabel">Commands</p>
        <h1>Current CLI command surface.</h1>
        <p>
          ocvm manages OpenClaw installs, project pins, defaults, snapshots, and
          diagnostic checks from one local CLI.
        </p>
        {commandGroups.map((group) => (
          <section className="commandGroup" key={group.title}>
            <h2>{group.title}</h2>
            <pre>
              <code>{group.commands.join('\n')}</code>
            </pre>
          </section>
        ))}
      </article>
  )
}
