STATUS: WORKING - phase 2 (candidate A) setup: branch created, dependencies installed, installing test tools
UPDATED: 2026-10-13T07:33:45Z
BRANCH: contrib/port-bundled-deps-tarball-tests
MODE: Auto

## Status

- Branch `contrib/port-bundled-deps-tarball-tests` created from `origin/main` (8cca2f6). Nothing committed yet.
- `pnpm install` finished (exit 0).
- `just` and `cargo-binstall` installed from crates.io. Next: `cargo-nextest`, `typos-cli`, `taplo-cli` via `cargo binstall`, as `just init` does.
- Then: port `bundledDependencies.ts:29` and `:46` into `pnpm/crates/cli/tests/suite/bundled_dependencies.rs`, do the break-and-restore check, and run the checks.

If a port fails on v12, I will stop here and report the lockfile difference before fixing anything.
