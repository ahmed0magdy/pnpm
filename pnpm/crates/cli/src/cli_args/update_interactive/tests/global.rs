//! Ports of the `global interactive update` cases. Each group is installed
//! through the code `pnpm add -g` runs, and the update is driven through
//! [`UpdateArgs::run_global`], where the CLI dispatches `update -g`.

use super::{MULTI_A, MULTI_B, MULTI_C, UpdatePrompt, scripted_prompts};
use crate::cli_args::{global::test_support::add_global_group, update::UpdateArgs};
use clap::Parser;
use pnpm_config::Config;
use pnpm_reporter::{GlobalLog, LogEvent, LogLevel, Reporter, SilentReporter};
use pnpm_testing_utils::registry::TestRegistry;
use std::{fs, path::PathBuf, sync::Mutex};
use tempfile::TempDir;

/// A pnpm home with its own global packages directory, against a mocked
/// registry of its own.
struct GlobalFixture {
    _dir: TempDir,
    global_pkg_dir: PathBuf,
    cache_dir: PathBuf,
    config: &'static Config,
    registry: TestRegistry,
}

impl GlobalFixture {
    fn new() -> Self {
        Self::with_config(|_| {})
    }

    fn with_config(customize: impl FnOnce(&mut Config)) -> Self {
        let dir = tempfile::tempdir().expect("create temporary pnpm home");
        let registry = TestRegistry::start_with_own_storage(dir.path());
        let global_pkg_dir = dir.path().join("global");
        let cache_dir = dir.path().join("cache");
        let mut config = Config::new();
        config.registry = registry.url().to_string();
        config.store_dir = dir.path().join("pnpm-store").into();
        config.cache_dir = cache_dir.clone();
        config.global_pkg_dir = Some(global_pkg_dir.clone());
        config.global_bin = Some(dir.path().join("bin"));
        config.enable_global_virtual_store = false;
        customize(&mut config);
        let config = Config::leak(config);
        Self { _dir: dir, global_pkg_dir, cache_dir, config, registry }
    }

    fn set_dist_tag(&self, package: &str, version: &str, tag: &str) {
        self.registry.set_dist_tag(package, version, tag);
        if self.cache_dir.exists() {
            fs::remove_dir_all(&self.cache_dir).expect("drop the cached registry metadata");
        }
    }

    /// `pnpm add -g` of one group holding every selector.
    async fn add(&self, selectors: &[&str]) {
        let group = selectors
            .iter()
            .map(|selector| (*selector).to_string())
            .collect::<Vec<_>>();
        add_global_group::<SilentReporter>(self.config, &group).await
            .expect("install the global group");
    }

    /// `pnpm update -g --interactive --latest [packages]`.
    async fn update_interactive<Reporter: self::Reporter + 'static>(&self, packages: &[&str]) {
        #[derive(Parser)]
        struct Harness {
            #[clap(flatten)]
            args: UpdateArgs,
        }
        let args = ["pacquet-test", "--interactive", "--latest"]
            .into_iter()
            .chain(packages.iter().copied());
        let mut parsed = Harness::try_parse_from(args).expect("parse update arguments").args;
        parsed.prompt = UpdatePrompt::Scripted;
        parsed.run_global::<Reporter>(self.config).await.expect("run pacquet update --global");
    }

    /// The one global group installed.
    fn only_group(&self) -> pnpm_global::GlobalPackageInfo {
        let groups = pnpm_global::scan_global_packages(&self.global_pkg_dir)
            .expect("scan the global packages");
        assert_eq!(groups.len(), 1, "expected one global group: {groups:?}");
        groups
            .into_iter()
            .next()
            .expect("one global group")
    }
}

/// Ports `global interactive update reads current versions from the
/// global virtual store`.
#[tokio::test]
async fn global_interactive_update_reads_current_versions_from_the_global_virtual_store() {
    let fixture = GlobalFixture::with_config(|config| config.enable_global_virtual_store = true);
    fixture.add(&[&format!("{MULTI_A}@1.0.0")]).await;
    fixture.set_dist_tag(MULTI_A, "2.1.0", "latest");

    let scripted = scripted_prompts();
    scripted.answer_next(&[]);
    fixture.update_interactive::<SilentReporter>(&[]).await;

    let prompts = scripted.seen();
    assert_eq!(prompts.len(), 1);
    assert_eq!(
        prompts[0].rows,
        [(format!("{MULTI_A} 1.0.0 → 2.1.0"), Some(fixture.only_group().hash))],
    );
}

/// Ports `global interactive update offers a matching group even when the
/// requested package is current`.
#[tokio::test]
async fn global_interactive_update_offers_a_matching_group_even_when_the_requested_package_is_current()
 {
    let fixture = GlobalFixture::new();
    fixture.set_dist_tag(MULTI_B, "3.1.0", "latest");
    fixture.add(&[&format!("{MULTI_A}@1.0.0"), &format!("{MULTI_B}@3.1.0")]).await;
    fixture.set_dist_tag(MULTI_A, "2.1.0", "latest");

    let scripted = scripted_prompts();
    scripted.answer_next(&[]);
    fixture.update_interactive::<SilentReporter>(&[MULTI_B]).await;

    let prompts = scripted.seen();
    assert_eq!(prompts.len(), 1);
    assert_eq!(
        prompts[0].rows,
        [(format!("{MULTI_A} 1.0.0 → 2.1.0"), Some(fixture.only_group().hash))],
    );
}

/// Ports `global interactive update reports when no group has the
/// requested package`. The `latest` tag is moved so that the group has an
/// update to offer: without the param, the command would prompt.
#[tokio::test]
async fn global_interactive_update_reports_when_no_group_has_the_requested_package() {
    let fixture = GlobalFixture::new();
    fixture.add(&[&format!("{MULTI_A}@1.0.0")]).await;
    fixture.set_dist_tag(MULTI_A, "2.1.0", "latest");

    let scripted = scripted_prompts();
    fixture.update_interactive::<SilentReporter>(&[MULTI_C]).await;

    assert!(scripted.seen().is_empty(), "a param no group has must not prompt");
}

/// Ports `global interactive update reads current versions when the
/// lockfile setting is off`.
#[tokio::test]
async fn global_interactive_update_reads_current_versions_when_the_lockfile_setting_is_off() {
    let fixture = GlobalFixture::with_config(|config| config.lockfile = false);
    fixture.add(&[&format!("{MULTI_A}@1.0.0")]).await;
    fixture.set_dist_tag(MULTI_A, "2.1.0", "latest");

    let scripted = scripted_prompts();
    scripted.answer_next(&[]);
    fixture.update_interactive::<SilentReporter>(&[]).await;

    let prompts = scripted.seen();
    assert_eq!(prompts.len(), 1);
    assert_eq!(
        prompts[0].rows,
        [(format!("{MULTI_A} 1.0.0 → 2.1.0"), Some(fixture.only_group().hash))],
    );
}

/// Ports `global interactive update does not match a group on an inherited
/// Object key`. A Rust map has no inherited keys, so this pins the same
/// outcome for a param that names no dependency.
#[tokio::test]
async fn global_interactive_update_does_not_match_a_group_on_an_inherited_object_key() {
    let fixture = GlobalFixture::new();
    fixture.add(&[&format!("{MULTI_A}@1.0.0")]).await;
    fixture.set_dist_tag(MULTI_A, "2.1.0", "latest");

    let scripted = scripted_prompts();
    fixture.update_interactive::<SilentReporter>(&["constructor"]).await;

    assert!(scripted.seen().is_empty(), "`constructor` must not match a group");
}

/// Ports `global interactive update leaves without an error when the
/// prompt is canceled`: the command reports the cancellation, updates
/// nothing, and returns without an error.
#[tokio::test]
async fn global_interactive_update_leaves_without_an_error_when_the_prompt_is_canceled() {
    static EVENTS: Mutex<Vec<LogEvent>> = Mutex::new(Vec::new());
    struct RecordingReporter;
    impl Reporter for RecordingReporter {
        fn emit(event: &LogEvent) {
            EVENTS
                .lock()
                .unwrap()
                .push(event.clone());
        }
    }

    let fixture = GlobalFixture::new();
    fixture.add(&[&format!("{MULTI_A}@1.0.0")]).await;
    fixture.set_dist_tag(MULTI_A, "2.1.0", "latest");
    let install_dir = fixture.only_group().install_dir;

    let scripted = scripted_prompts();
    scripted.cancel_next();
    fixture.update_interactive::<RecordingReporter>(&[]).await;

    assert_eq!(scripted.seen().len(), 1);
    assert_eq!(fixture.only_group().install_dir, install_dir, "the group must not be reinstalled");
    assert_eq!(
        pnpm_global::installed_versions(&install_dir).get(MULTI_A).map(String::as_str),
        Some("1.0.0"),
    );
    let events = EVENTS.lock().unwrap();
    let canceled = events.iter().any(|event| {
        matches!(
            event,
            LogEvent::Global(GlobalLog { level: LogLevel::Info, message }) if message == "Update canceled",
        )
    });
    assert!(canceled, "no `Update canceled` was reported: {events:?}");
}
