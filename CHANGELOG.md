# Changelog

All notable changes to this project are documented here. The format follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.2] - 2026-05-03

### Added

- `saya auth set` writes the Bearer access token to the credentials file (`--token` or one line from stdin).

### Changed

- CLI runs on a single Tokio runtime (`#[tokio::main]`); `health` and conversation creation use one async `reqwest::Client` (no `reqwest::blocking`, no nested runtime in `chat`).
- Stream retries use `tokio::time::sleep` instead of `std::thread::sleep`.
- Interactive restricted-tools confirm runs in `spawn_blocking` so the async runtime is not blocked on stdin.

## [0.1.1] - 2026-04-17

### Changed

- Release workflow: `npm_publish` fails if `NPM_TOKEN` is missing (no silent skip).

## [0.1.0] - 2026-04-17

### Added

- Initial published distribution story: Rust binary `saya`, npm wrapper `@sozlabs/saya-cli`, and platform-specific optional packages for macOS (arm64, x64), Linux (arm64, x64), and Windows (x64).
- GitHub Actions `release` workflow: builds per target on tag `v*`, uploads GitHub Release archives, publishes to npm when `NPM_TOKEN` is configured.

### Breaking

- None for this initial release baseline. Future breaking CLI flag or JSON output changes will be listed under **Breaking** with migration notes.
