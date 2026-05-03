# Changelog

## 0.1.0

- Initial Rust CLI project for `ocvm`.
- Added MVP commands for install, uninstall, use, default, current, list, list-remote, pin, unpin, exec, and doctor.
- Added npm and release-manifest source providers.
- Added staging install flow with `openclaw --version` verification and backup restore when replacing an existing version.
- Added project pin resolution through `.openclaw-version`.
- Added CI gates for fmt, clippy, and tests.
