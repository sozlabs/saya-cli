# Changelog

All notable changes to this project are documented here. The format follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-04-17

### Added

- Initial published distribution story: Rust binary `saya`, npm wrapper `@sozlabs/saya-cli`, and platform-specific optional packages for macOS (arm64, x64), Linux (arm64, x64), and Windows (x64).
- GitHub Actions `release` workflow: builds per target on tag `v*`, uploads GitHub Release archives, publishes to npm when `NPM_TOKEN` is configured.

### Breaking

- None for this initial release baseline. Future breaking CLI flag or JSON output changes will be listed under **Breaking** with migration notes.
