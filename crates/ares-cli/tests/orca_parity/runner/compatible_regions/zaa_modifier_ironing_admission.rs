//! Modifier-part PrintRegionConfig admission for object ironing.
//! Upstream PrintObject.cpp:3664-3709 -> Fill.cpp:1571-1614.
use super::fixture;
use super::*;
use crate::runner::OrcaRunner;
use std::{fs, process::Command};

#[derive(Clone, Copy)]
enum Case {
    Mixed,
    MixedDisabled,
    NoReplacement,
}
impl Case {
    fn name(self) -> &'static str {
        match self {
            Self::Mixed => "mixed",
            Self::MixedDisabled => "mixed-disabled",
            Self::NoReplacement => "no-replacement",
        }
    }
    fn expected_sha(self) -> &'static str {
        match self {
            Self::Mixed => "414914364064c0e6087377bfb22ffb121223f0fc124d380e24099934967e0b59",
            Self::MixedDisabled => {
                "45e86945db992f1d53287759b86c478a6eeb57d82dd767837f70b0ebeb160878"
            }
            Self::NoReplacement => {
                "70028897af49949402b5c0511d6a9e8de92d9d32fb4c6488e27ab4796b04c75c"
            }
        }
    }
}

#[test]
fn modifier_mixed_pattern_strict_output_diverges() {
    verify(Case::Mixed);
}
#[test]
fn modifier_mixed_disabled_strict_output_diverges() {
    verify(Case::MixedDisabled);
}
#[test]
fn modifier_global_concentric_is_primary_admission_red() {
    verify(Case::NoReplacement);
}

fn verify(case: Case) {
    let Some(mut runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let project = frozen(case);
    runner.cache = tempfile::Builder::new()
        .prefix("zaa-modifier-ironing-cache-")
        .tempdir_in(&runner.work)
        .unwrap()
        .keep();
    let root = tempfile::Builder::new()
        .prefix("zaa-modifier-ironing-input-")
        .tempdir_in(&runner.work)
        .unwrap()
        .keep();
    let project_path = root.join("input.3mf");
    fs::write(&project_path, &project).unwrap();
    let output_dir = root.join("native");
    fs::create_dir(&output_dir).unwrap();
    let sliced = runner
        .rerun_reference(&crate::runner::ExportedCase {
            label: case.name().to_string(),
            project: project.clone(),
            project_path,
            output_dir,
        })
        .unwrap();
    let outcome = parity::compare_case(&sliced);
    let artifacts = outcome.artifacts.as_ref().unwrap();
    let cli = Command::new(env!("CARGO_BIN_EXE_ares"))
        .args(["slice", "-o"])
        .arg(artifacts.join("cli.gcode"))
        .arg(artifacts.join("input.3mf"))
        .output()
        .unwrap();
    fs::write(artifacts.join("cli.stdout"), &cli.stdout).unwrap();
    fs::write(artifacts.join("cli.stderr"), &cli.stderr).unwrap();
    let api = fs::read(artifacts.join("ares.gcode")).unwrap();
    let cli_bytes = fs::read(artifacts.join("cli.gcode")).unwrap();
    assert!(cli.status.success());
    assert_eq!(api, cli_bytes, "API/CLI raw output must remain equal");
    let strict = crate::golden::compare_ordered_bytes_generator_only(&sliced.reference, &api);
    assert!(
        strict.is_ok(),
        "modifier ironing must match Native strict output: {}",
        case.name()
    );
    let native_text = std::str::from_utf8(&sliced.reference).unwrap();
    let api_text = std::str::from_utf8(&api).unwrap();
    assert_eq!(
        native_text.matches(";TYPE:Ironing").count(),
        api_text.matches(";TYPE:Ironing").count(),
        "Ironing role count must match Native: {}",
        case.name()
    );
    let native = std::str::from_utf8(&sliced.reference).unwrap();
    assert!(
        native.contains(";TYPE:Ironing"),
        "Native fixture must emit Ironing: {}",
        case.name()
    );
}

fn frozen(case: Case) -> Vec<u8> {
    let path = match case {
        Case::Mixed => "mixed.3mf",
        Case::MixedDisabled => "mixed-disabled.3mf",
        Case::NoReplacement => "no-replacement.3mf",
    };
    let bytes = fs::read(
        crate::runner::repo_root()
            .join("tests/parity/zaa-modifier-ironing-admission")
            .join(path),
    )
    .unwrap();
    assert_eq!(fixture::hash(&bytes), case.expected_sha());
    bytes
}
