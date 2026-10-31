export const currentRelease = 'v0.1.1'

export const repositoryUrl = 'https://github.com/PerfectPan/ocvm'
export const releasesUrl = `${repositoryUrl}/releases`
export const currentReleaseUrl = `${releasesUrl}/tag/${currentRelease}`
export const docsUrl = '/docs'

export const installerBaseUrl =
  'https://ocvm.vercel.app/ocvm'

export const stableInstallerUrl = `${installerBaseUrl}/install.sh`

export const installCommand = `curl -fsSL \\
  ${stableInstallerUrl} | sh`

export const pinnedInstallCommand = `curl -fsSL \\
  ${stableInstallerUrl} | OCVM_VERSION=${currentRelease} sh`

export const githubStarsLabel = 'GitHub stars'
export const githubStarsText = '0 stars'

export const commandGroups = [
  {
    title: 'Install and remove',
    commands: [
      'ocvm install <version|latest|stable>',
      'ocvm uninstall <version>',
      'ocvm list',
      'ocvm list-remote',
      'ocvm list-remote --channel stable',
      'ocvm list-remote --channel nightly',
    ],
  },
  {
    title: 'Select versions',
    commands: [
      'ocvm use <version>',
      'ocvm default <version>',
      'ocvm current',
      'ocvm pin <version>',
      'ocvm unpin',
    ],
  },
  {
    title: 'Operate safely',
    commands: [
      'ocvm snapshot [name]',
      'ocvm rollback [name]',
      'ocvm exec [version] -- <command>',
      'ocvm doctor',
      'ocvm init <zsh|bash|fish>',
    ],
  },
]

export const docLinks = [
  {
    href: '/docs/installation',
    title: 'Installation',
    body: 'Install ocvm from the OSS/CDN installer path and set up PATH entries.',
  },
  {
    href: '/docs/commands',
    title: 'Commands',
    body: 'Review the current CLI surface for installs, pinning, snapshots, and diagnostics.',
  },
  {
    href: '/docs/configuration',
    title: 'Configuration',
    body: 'Configure ocvm home, package source, and release manifest behavior.',
  },
  {
    href: '/docs/release-sources',
    title: 'Release sources',
    body: 'Understand npm and manifest source providers and checksum verification.',
  },
]
