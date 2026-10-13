STATUS: WORKING - tests ported, first build and focused test run in progress
UPDATED: 2026-10-13T13:53:32Z
BRANCH: contrib/port-bundled-deps-tarball-tests
MODE: Auto

## Status

- Tools installed: just, cargo-binstall (built from crates.io), then cargo-nextest, typos-cli and taplo-cli through `cargo binstall`, as `just init` does.
- Git author for this repo is now set to `ahmed0magdy <110637215+ahmed0magdy@users.noreply.github.com>`. No contrib commits exist yet, so nothing needed re-authoring.
- Two tests added to `pnpm/crates/cli/tests/suite/bundled_dependencies.rs`: `local_tarball_with_bundled_dependencies` and `local_tarball_with_bundle_dependencies_true`. The `TEST_PORTING.md` entry is ticked. Not committed yet.
- Running now: `pnpm test:rust -p pnpm-cli -E 'test(/^bundled_dependencies::/)'`.

Environment note: the repo's pre-push hook compiles the TypeScript workspace, which downloads Node 26.11.1. pnpm also fetches musl checksums from unofficial-builds.nodejs.org, which this VM's proxy blocks. I work around it with `PNPM_CONFIG_NODE_DOWNLOAD_MIRRORS='{"release":"https://nodejs.org/dist/"}'` set only in the push command's environment. The hook still runs; it is not bypassed, and the repo is not changed.
