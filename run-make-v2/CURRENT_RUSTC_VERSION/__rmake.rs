extern crate rmake_support;

use rmake_support::{CrateType, EmitKind, PathFragment, RmakeSupportContext};

fn main() -> std::io::Result<()> {
    let mut scx = RmakeSupportContext::init();
    let output = scx
        .rustc()
        .current_dir(&vec![PathFragment::ExpandEnvVar(vec![PathFragment::Str(
            "TMPDIR".to_string(),
        )])])
        .emit(EmitKind::Metadata)
        .crate_type(CrateType::Lib)
        .path(&vec![
            PathFragment::Str(format!(
                "{}",
                std::env::current_dir()?
                    .join("run-make-v2/CURRENT_RUSTC_VERSION")
                    .display()
            )),
            PathFragment::Str("/stable.rs".to_string()),
        ])
        .compile()?;

    assert!(output.status.success());

    let output = scx
        .rustc()
        .current_dir(&vec![PathFragment::Str(format!(
            "{}",
            std::env::current_dir()?
                .join("run-make-v2/CURRENT_RUSTC_VERSION")
                .display()
        ))])
        .r#extern(
            "stable".to_string(),
            &vec![
                PathFragment::ExpandEnvVar(vec![PathFragment::Str("TMPDIR".to_string())]),
                PathFragment::Str("/libstable.rmeta".to_string()),
            ],
        )
        .path(&vec![PathFragment::Str("main.rs".to_string())])
        .compile()?;


    let stderr = String::from_utf8_lossy(&output.stderr);

    // too lazy to include src/version, imagine it exists

    assert!(stderr.contains("since 1.72.0-nightly"));

    Ok(())
}
