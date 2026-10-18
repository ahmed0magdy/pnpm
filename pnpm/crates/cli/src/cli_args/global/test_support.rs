use super::{
    Config, Context, GlobalInstallTarget, IntoDiagnostic, RangeSpecStyle, Reporter, fs, global_dirs,
};

/// [`super::handle_global_add`] for one group, without its check that the global
/// bin directory is on `PATH`: a test running in-process cannot meet it
/// without changing the process environment.
pub(crate) async fn add_global_group<Reporter: self::Reporter + 'static>(
    base_config: &'static Config,
    group: &[String],
) -> miette::Result<()> {
    let (global_pkg_dir, global_bin_dir) = global_dirs(base_config)?;
    for dir in [&global_pkg_dir, &global_bin_dir] {
        fs::create_dir_all(dir).into_diagnostic().wrap_err("create the global directories")?;
    }
    GlobalInstallTarget {
        base_config,
        global_pkg_dir: &global_pkg_dir,
        global_bin_dir: &global_bin_dir,
    }
    .add_group::<Reporter>(group, RangeSpecStyle::default(), None, &[])
    .await
}
