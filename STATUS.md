STATUS: DONE
UPDATED: 2026-10-18
BRANCH: contrib/interactive-update-ports (pushed to ahmed0magdy/pnpm, head bc949954, base origin/main 07ab9fa8)
MODE: Idle

## What was done

The six `global interactive update` cases from `pnpm11/installing/commands/test/update/interactive.ts` are ported in one commit, `bc949954`:

| Upstream | Rust test |
|---|---|
| `:80` reads current versions from the global virtual store | `global_interactive_update_reads_current_versions_from_the_global_virtual_store` |
| `:115` offers a matching group even when the requested package is current | `global_interactive_update_offers_a_matching_group_even_when_the_requested_package_is_current` |
| `:140` reports when no group has the requested package | `global_interactive_update_reports_when_no_group_has_the_requested_package` |
| `:158` reads current versions when the lockfile setting is off | `global_interactive_update_reads_current_versions_when_the_lockfile_setting_is_off` |
| `:181` does not match a group on an inherited Object key | `global_interactive_update_does_not_match_a_group_on_an_inherited_object_key` |
| `:196` leaves without an error when the prompt is canceled | `global_interactive_update_leaves_without_an_error_when_the_prompt_is_canceled` |

Files:

- `pnpm/crates/cli/src/cli_args/update_interactive/tests/global.rs` (new): the `GlobalFixture` and the six tests.
- `pnpm/crates/cli/src/cli_args/update_interactive/tests.rs`: one line, `mod global;`.
- `pnpm/crates/cli/src/cli_args/global/test_support.rs` (new): a test-only helper, `add_global_group`, declared in `global.rs` as `#[cfg(test)] pub(crate) mod test_support;`. It is not in the shipped binary.
- `pnpm/plans/TEST_PORTING.md`: the six entries are ticked. The remaining entry now names only `:376` and `:516`.

Commit: author and committer `ahmed0magdy <110637215+ahmed0magdy@users.noreply.github.com>`, `Co-Authored-By: Claude` trailer, no Claude-Session trailer. Normal push, no force. No PR, issue or comment was opened. No wakeups were scheduled.

## A push that failed first

The first push attempt (commit `24101734`) was rejected by the pre-push hook. It never reached GitHub. Dylint failed twice:

- `perfectionist::excessive-inline-tests`: the helper was written inline in `global.rs` behind `#[cfg(test)]`. I moved it to `global/test_support.rs`.
- A `pub mod` came after other items. I moved the declaration up next to `pub mod rust;`.

I amended the unpushed commit to `bc949954`, then re-ran everything: dylint, rustfmt, the 103 module tests, `check:rust`, `lint:rust`, rustdoc and typos. All passed. The move does not change any behavior, so the break results below still hold.

## Why this batch

One fixture unlocks all six global cases. Each needs the same thing: a global group installed for real, then `update -g --interactive` run against it. So they go together, as the scope rule asked. `:376` and `:516` need a different fixture (a recursive workspace install) and are left for the next PR.

## How the fixture works

- `global::test_support::add_global_group` (test-only) calls `GlobalInstallTarget::add_group`, which is the code `pnpm add -g` runs for each group. It installs the group, links its bins into a temp bin dir, and writes the hash link. It skips only `handle_global_add`'s check that the global bin dir is on `PATH`. An in-process test cannot pass that check without changing the process environment, which the repo rules avoid.
- The update runs through `UpdateArgs::run_global`, the same entry the CLI dispatches `update -g` to. The prompt is answered through the existing `UpdatePrompt::Scripted` seam. Answering with nothing selected returns before `handle_global_update`, the same as upstream's mock returning `[]`.
- Each fixture has its own mock registry storage, so moving a `latest` tag in one test cannot leak into another.

## Assertions, compared with the TypeScript tests

- Rows: the TypeScript tests check `choices: [{ name: '... 1.0.0 → 2.1.0', value: expect.any(String) }]`. The Rust ports check the whole row list exactly, and that the value is the hash of the one installed group, not just any string.
- `:140` and `:181` (no match): the TypeScript tests check the returned message and that no prompt was shown. pacquet prints the message to stdout, which an in-process test cannot read. So the ports first move `latest` to 2.1.0 to give the group an update. Without the param the command would prompt, so "no prompt" can only mean the param matched no group. A prompt would panic with "the test scripted no answer".
- `:196` (cancel): the TypeScript test checks `process.exit(0)`, which means leaving without an error. The Rust port checks four things: the run returns `Ok`; `Update canceled` is reported; the group's install dir is unchanged; and the installed version is still 1.0.0.
- `:181`: a Rust map has no inherited keys like `constructor`. The port pins the same outcome for a param that names no dependency, and the plan entry says so.

## Break checks

Each break was undone with `git restore` plus `touch`. Logs are in `/tmp/claude-0/-home-user-pnpm/d18f8526-da91-5ef2-ad4a-e1058a0502ae/scratchpad/breaks/`.

| Break | Where | Result |
|---|---|---|
| B1: turn the global virtual store on inside the group | `global/install.rs`, `cfg.enable_global_virtual_store = false` removed | All 6 pass (see note) |
| B2: don't read the group's lockfile | `update_interactive/global.rs`, pass `None` | `:80`, `:115`, `:158`, `:196` fail: no prompt shown (0 vs 1) |
| B3: reading follows the `lockfile` setting | `update_interactive/global.rs`, `config.lockfile = true` removed | only `:158` fails |
| B4: group install follows the `lockfile` setting | `global/install.rs`, `cfg.lockfile = true` removed | only `:158` fails |
| B5: offer a group only if the requested package is outdated | `update_interactive/global.rs`, rows filtered by param | only `:115` fails |
| B6: ignore the params | `update_interactive/global.rs`, `has_alias` replaced by `true` | `:140` and `:181` fail: "the test scripted no answer for the prompt" |
| B7: don't report the cancel | `update_interactive/global.rs`, `report_cancelled` removed | `:196` fails: "no `Update canceled` was reported: []" |
| B8: return the cancel as an error | `update_interactive/global.rs`, `Err(miette!("canceled"))` | `:196` fails: "run pacquet update --global: × canceled" |
| B9: swap current and target in the row label | `update_interactive/global.rs` | `:80`, `:115`, `:158` fail on the exact row: `2.1.0 → 1.0.0` vs `1.0.0 → 2.1.0` |

Note on B1: pacquet always keeps a global group's virtual store inside the group, and reads the group's own `pnpm-lock.yaml` wherever its packages are linked. No pacquet code path depends on the global virtual store here, so nothing GVS-specific can break. The `:80` port is still a real check through B2 and B9, and it guards the outcome if that ever changes. The plan entry says this.

## Checks run (all passing)

- `pnpm test:rust -p pnpm-cli -E 'test(/update_interactive::tests::global::/)'`: 6 passed.
- `pnpm test:rust -p pnpm-cli -E 'test(/cli_args::update_interactive::/) | test(/cli_args::global::/)'`: 103 passed. This covers the existing interactive ports and the global module the helper lives in.
- `pnpm check:rust`, `pnpm lint:rust` (clippy `-D warnings`), `typos pnpm pnpr`, rustfmt `--check`.
- The pre-push hook: rustfmt, clippy, rustdoc (`-D warnings`), dylint, typos, taplo, and the TypeScript compile and lint.

## Left for a later PR, and why

- `:376` `interactive update of dev dependencies only` and `:516` `interactive recursive workspace update allows updating external dependencies`. Both run `install` then `update` recursively over a two-project workspace. They need a recursive workspace-install fixture at the in-process level, which the existing `UpdateFixture` (one project) does not provide. Building it is a separate, reviewable step.
- `:516` was not in the plan at all. `interactively_update_with_workspace_flag_allows_external_dependencies` already covers `--workspace` with an external dependency, but for one project, not the recursive run. So I did not tick it. The remaining plan entry names it.

## Explain it to the owner

**What the tests are about.** `pnpm update -g --interactive` lists your global package groups that have newer versions and lets you pick which to update. These six tests check that list.

- **Virtual store (`:80`).** With the global virtual store setting on, the list still shows the installed version ("1.0.0 → 2.1.0").
- **Requested package already current (`:115`).** You ask about package B, which is already at its latest version, but it shares a group with package A, which is outdated. The group must still be offered, because groups update as a whole.
- **No group has the package (`:140`).** You ask about a package no group contains. Nothing is offered and no prompt appears.
- **Lockfile setting off (`:158`).** With `lockfile=false` in your config, the installed versions are still found, because a global group always writes its own lockfile.
- **The name "constructor" (`:181`).** Asking for `constructor` matches nothing. In JavaScript that name was a real trap (it exists on every object). Rust has no such trap, so the test simply pins "matches nothing".
- **Ctrl-C (`:196`).** Pressing Ctrl-C at the prompt says "Update canceled", changes nothing, and exits without an error.

**How the tests run.** They install real global groups from a local test registry, using the same code `pnpm add -g` uses. The one step skipped is a check that your bin folder is on `PATH`. Then they run `update -g --interactive` and answer the prompt through the test hook pacquet already has.

**What the breaks proved.** I broke the real code in nine ways, one at a time, and put it back each time:

- Stop reading the group's lockfile: four tests fail.
- Show the versions backwards: the three row tests fail.
- Let the lockfile setting win, on the read or on the install: only the lockfile test fails.
- Only offer a group if the requested package itself is outdated: only that test fails.
- Ignore the package names you typed: both "no match" tests fail.
- Don't report the cancel, or treat it as an error: the Ctrl-C test fails.

One break failed nothing: turning the global virtual store on inside a group. That's expected, because pacquet never uses the shared store for global groups. I wrote this down instead of hiding it.

**Likely reviewer questions:**

- Why is there a new `global/test_support.rs`? To install a group the real way without the `PATH` check. It's behind `#[cfg(test)]`, so it isn't in the shipped binary. It sits in its own file because the repo's dylint rules forbid inline test code.
- Why not assert the "No matching global packages found" message? pacquet prints it to stdout, which an in-process test can't read. The test instead makes "no prompt" mean "no match".
- Why not `:376`/`:516` too? They need a different fixture (a recursive workspace). Keeping this PR to one fixture keeps it small.

## Ready-to-paste PR

Title:

```text
test(cli): port the global interactive update tests
```

Body:

````markdown
## Summary

Ports the six `global interactive update` cases from `pnpm11/installing/commands/test/update/interactive.ts` into `pnpm/crates/cli/src/cli_args/update_interactive/tests/global.rs`. They cover the "five remaining global cases" entry in `pnpm/plans/TEST_PORTING.md` plus the global-prompt half of the cancel case:

- `:80` reads current versions from the global virtual store, ported as `global_interactive_update_reads_current_versions_from_the_global_virtual_store`
- `:115` offers a matching group even when the requested package is current, ported as `global_interactive_update_offers_a_matching_group_even_when_the_requested_package_is_current`
- `:140` reports when no group has the requested package, ported as `global_interactive_update_reports_when_no_group_has_the_requested_package`
- `:158` reads current versions when the lockfile setting is off, ported as `global_interactive_update_reads_current_versions_when_the_lockfile_setting_is_off`
- `:181` does not match a group on an inherited Object key, ported as `global_interactive_update_does_not_match_a_group_on_an_inherited_object_key`
- `:196` leaves without an error when the prompt is canceled, ported as `global_interactive_update_leaves_without_an_error_when_the_prompt_is_canceled`

Each group is installed through `GlobalInstallTarget::add_group`, the code `pnpm add -g` runs. A test-only helper in `global/test_support.rs` skips only the check that the global bin directory is on `PATH`, which an in-process test cannot meet without changing the process environment. The update runs through `UpdateArgs::run_global`, the entry the CLI dispatches `update -g` to, with the prompt answered through `UpdatePrompt::Scripted`.

### What each test asserts

- `:80`, `:115`, `:158`: exactly one prompt, whose rows are exactly `[("@pnpm.e2e/multi-version-a 1.0.0 → 2.1.0", <hash of the installed group>)]`.
- `:140`, `:181`: no prompt. The `latest` tag is moved first, so the group has an update and only the param can keep the prompt away. pacquet prints the message to stdout, which an in-process test cannot read.
- `:196`: the run returns `Ok`, `Update canceled` is reported, and the group's install dir and installed version are unchanged.

### Break checks

Each break was reverted with `git restore`:

- Not reading the group's lockfile fails `:80`, `:115`, `:158`, `:196`.
- Swapping current and target in the row label fails `:80`, `:115`, `:158`.
- Letting the `lockfile` setting win, on the read or the group install, fails only `:158`.
- Offering a group only when the requested package is outdated fails only `:115`.
- Ignoring the params fails `:140` and `:181`.
- Not reporting the cancel, or returning it as an error, fails `:196`.

Turning the global virtual store on inside the group fails nothing. A group reads its own lockfile wherever its packages are linked, so `:80` guards the outcome rather than a GVS-specific path. The plan entry notes this.

### Checks run

- `pnpm test:rust -p pnpm-cli -E 'test(/cli_args::update_interactive::/) | test(/cli_args::global::/)'`: 103 passed.
- `pnpm check:rust` and `pnpm lint:rust`.
- `cargo dylint --all`.
- The pre-push hook: rustfmt, clippy, rustdoc, dylint, typos, taplo, and the TypeScript compile and lint.

`interactive update of dev dependencies only` (`:376`) and `interactive recursive workspace update allows updating external dependencies` (`:516`) still need a recursive workspace-install fixture. They stay in the plan for a follow-up.

## Squash Commit Body

```text
Port the six `global interactive update` cases from
installing/commands/test/update/interactive.ts. Each group is
installed through the code `pnpm add -g` runs, and the update is
driven through `UpdateArgs::run_global`, where the CLI dispatches
`update -g`. The prompt is answered through `UpdatePrompt::Scripted`.

`global::test_support::add_global_group` is a test-only entry to
`GlobalInstallTarget::add_group`. It skips the check that the global
bin directory is on `PATH`, which a test running in-process cannot
meet without changing the process environment.

Each port was checked against a temporary break: not reading the
group's lockfile, swapping the versions in the row label, following
the `lockfile` setting, offering a group only when the requested
package is outdated, ignoring the params, and not reporting or
erroring on a cancel each fail the ports that cover them. Turning
the global virtual store on inside the group fails nothing, since a
group reads its own lockfile wherever its packages are linked.
```

## Checklist

- [x] Added or updated tests.

This PR only adds tests, so it needs no changeset.

---
Written by an agent (Claude Code, <model>).
````
