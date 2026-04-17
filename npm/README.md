# npm distribution

- **Wrapper:** `@sozlabs/saya-cli` — installs a Node shim `saya` that runs the native binary from one of the `optionalDependencies` packages matching your OS/arch.
- **Platform packages:** `@sozlabs/saya-cli-darwin-arm64`, `-darwin-x64`, `-linux-arm64`, `-linux-x64`, `-win32-x64` — contain only the compiled `saya` / `saya.exe` binary.

Release automation copies binaries from `cargo build --release` into `npm/packages/<platform>/bin/` and runs `npm publish` for each package, then the wrapper.

To bump versions locally before tagging, align `Cargo.toml` and run:

```bash
node npm/scripts/set-version.js 0.2.0
```

Then commit, tag `v0.2.0`, and push.
