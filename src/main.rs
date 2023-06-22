use std::path::Path;
use std::process::Command;

use miette::IntoDiagnostic;
use tracing::{debug, info};

fn main() -> miette::Result<()> {
    miette::set_hook(Box::new(|_| {
        Box::new(
            miette::MietteHandlerOpts::new()
                .terminal_links(false)
                .unicode(false)
                .context_lines(2)
                .tab_width(4)
                .build(),
        )
    }))
    .unwrap();
    setup_logging();

    let support_lib_directory = Path::new("rmake-support");
    let support_lib_entry_point = support_lib_directory.join("lib.rs");

    let test_directory = Path::new("run-make-v2/CURRENT_RUSTC_VERSION");
    let test_recipe = test_directory.join("__rmake.rs");
    info!(
        "building and running test at `{}`",
        test_directory.display()
    );

    let temp_dir = std::env::temp_dir();

    let temp_support_lib_build_dir = temp_dir.join("rmake_support");

    let temp_bin = temp_dir.join("__rmake");

    let support_output = Command::new("rustc")
        .arg("--crate-name=rmake_support")
        .arg("--edition=2021")
        .arg(&support_lib_entry_point)
        .arg("--crate-type=lib")
        .arg("--emit=link")
        .arg("-Cembed-bitcode=no")
        .arg("-Cdebuginfo=2")
        .arg(format!(
            "--out-dir={}",
            temp_support_lib_build_dir.display()
        ))
        .arg(format!(
            "-Ldependency={}",
            temp_support_lib_build_dir.display()
        ))
        .output()
        .into_diagnostic()?;

    debug!("{}", String::from_utf8_lossy(&support_output.stderr));

    info!(
        "built support library at `{}`",
        temp_support_lib_build_dir.display()
    );

    let paths = std::fs::read_dir(&temp_support_lib_build_dir).unwrap();
    for path in paths {
        info!("Name: {}", path.unwrap().path().display())
    }

    debug!(
        "{}",
        temp_support_lib_build_dir
            .join("librmake_support.rlib")
            .display()
    );

    let recipe_build_output = Command::new("rustc")
        .arg("-o")
        .arg(format!("{}", temp_bin.display()))
        .arg(&test_recipe)
        .arg(format!(
            "-Ldependency={}",
            temp_support_lib_build_dir.display()
        ))
        .arg("--extern")
        .arg(format!(
            "rmake_support={}",
            temp_support_lib_build_dir
                .join("librmake_support.rlib")
                .display()
        ))
        .output()
        .into_diagnostic()?;

    debug!("{}", String::from_utf8_lossy(&recipe_build_output.stderr));

    let recipe_output = Command::new(temp_bin).output().into_diagnostic()?;

    debug!("RECIPE STDERR\n\n{}", String::from_utf8_lossy(&recipe_output.stderr));

    debug!("RECIPE STDOUT\n\n{}", String::from_utf8_lossy(&recipe_output.stdout));

    Ok(())
}

fn setup_logging() {
    use tracing_subscriber::prelude::*;
    use tracing_subscriber::{fmt, EnvFilter};

    let fmt_layer = fmt::layer()
        .compact()
        .with_level(true)
        .with_target(true)
        .without_time();
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();
    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .init();
}
