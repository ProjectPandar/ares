//! GCodeProcessor.cpp:5357-5398: Repetier cached feedrate, runtime Native.
//! Full public API/CLI comparisons retain generator-only normalization.
use super::{CaseInputs, OrcaRunner, ParityCase, application, repo_root};
use crate::{presets::VendorProfiles, runner};
use serde_json::json;
use std::{fs, process::Command};
#[path = "repetier_cached_feedrate_inputs.rs"]
mod inputs;

#[derive(Clone, Copy)]
enum Case {
    R0,
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
    R6Off,
    R3Off,
    R2Off,
}

impl Case {
    fn flavor(self) -> &'static str {
        match self {
            Self::R6 | Self::R6Off => "marlin",
            _ => "repetier",
        }
    }
    fn sequence(self) -> String {
        let tail = match self {
            Self::R0 | Self::R6Off => "G1 X170 Y70 F6000\nG1 X170 Y170\n",
            Self::R1 | Self::R6 => "M401\nG1 X170 Y70 F6000\nM402\nG1 X170 Y170\n",
            Self::R2 => "M401\nG1 X120 Y70 F600\nM401\nG1 X170 Y70 F6000\nM402\nG1 X170 Y170\n",
            Self::R3 => {
                "M401\nG1 X170 Y70 F6000\nM402\nG1 X170 Y120\nG1 X170 Y140 F3000\nM402\nG1 X170 Y170\n"
            }
            Self::R4 => "M401\nG1 X170 Y70 F6000\nM402 F120\nG1 X170 Y170\n",
            Self::R5 => "G1 X170 Y70 F6000\nM402\nG1 X170 Y170\n",
            Self::R7 => "M401\nM402\nG1 X170 Y70 F6000\nG1 X170 Y170\n",
            Self::R2Off => "G1 X120 Y70 F600\nG1 X170 Y70 F6000\nG1 X170 Y170\n",
            Self::R3Off => "G1 X170 Y70 F6000\nG1 X170 Y120\nG1 X170 Y140 F3000\nG1 X170 Y170\n",
        };
        format!("G90\nM83\nG1 X70 Y70 F60\n{tail}")
    }
}

macro_rules! witness {
    ($name:ident, $case:ident) => {
        #[test]
        fn $name() {
            compare(stringify!($case), Case::$case);
        }
    };
}
witness!(repetier_cache_r0, R0);
witness!(repetier_cache_r1, R1);
witness!(repetier_cache_r2, R2);
witness!(repetier_cache_r3, R3);
witness!(repetier_cache_r4, R4);
witness!(repetier_cache_r5, R5);
witness!(repetier_cache_r6, R6);
witness!(repetier_cache_r7, R7);

fn compare(label: &str, case: Case) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let active = inputs::reference(&runner, label, case);
    require_sequence(&active.reference, &case.sequence());
    let baseline_case = match case {
        Case::R0 => None,
        Case::R2 => Some(Case::R2Off),
        Case::R3 => Some(Case::R3Off),
        Case::R6 => Some(Case::R6Off),
        _ => Some(Case::R0),
    };
    if let Some(baseline_case) = baseline_case {
        let baseline = inputs::reference(&runner, &format!("{label}/baseline"), baseline_case);
        require_equal(&evaluate(&baseline));
        match case {
            Case::R1 | Case::R2 | Case::R3 | Case::R4 => assert_ne!(
                footer(&active.reference),
                footer(&baseline.reference),
                "{label}: actual Native cache consumption"
            ),
            _ => assert_eq!(
                footer(&active.reference),
                footer(&baseline.reference),
                "{label}: Native inert cache/flavor control"
            ),
        }
    }
    if matches!(case, Case::R4) {
        let bare = inputs::reference(&runner, "R4/bare", Case::R1);
        // GCodeReader.hpp:35/cpp:292-315: has_value(4,p) is not an F-word lookup.
        assert_eq!(footer(&active.reference), footer(&bare.reference));
    }
    require_equal(&evaluate(&active));
}

fn footer(bytes: &[u8]) -> &str {
    std::str::from_utf8(bytes)
        .unwrap()
        .lines()
        .find(|line| line.starts_with("; estimated printing time (normal mode) = "))
        .unwrap()
}

fn require_sequence(bytes: &[u8], sequence: &str) {
    let text = std::str::from_utf8(bytes).unwrap();
    let block = text
        .split("; W6_REPETIER_CACHE_BEGIN\n")
        .nth(1)
        .unwrap()
        .split("; W6_REPETIER_CACHE_END")
        .next()
        .unwrap();
    let commands = block
        .lines()
        .filter(|line| !line.starts_with("M73 "))
        .collect::<Vec<_>>();
    assert_eq!(commands, sequence.lines().collect::<Vec<_>>());
    assert!(text.contains(";TYPE:Sparse infill"));
}

fn evaluate(case: &ParityCase) -> std::path::PathBuf {
    let root = tempfile::Builder::new()
        .prefix("repetier-cache-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    fs::write(root.join("input.3mf"), &case.project).unwrap();
    fs::write(root.join("native.gcode"), &case.reference).unwrap();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let result = runtime.block_on(ares_core::slice_project_plate_validated(
        &case.project,
        ares_core::GenerationMetadata::default(),
        None,
        &ares_core::ConfigOverrides::default(),
        ares_core::SlicerContext::new(0),
    ));
    if let Ok(bytes) = &result {
        fs::write(root.join("ares.gcode"), bytes).unwrap();
    }
    let api_comparison = result
        .as_ref()
        .map_err(|error| format!("{error:?}"))
        .and_then(|bytes| {
            crate::golden::compare_ordered_bytes_generator_only(&case.reference, bytes)
        });
    let output = root.join("ares.cli.gcode");
    let cli = Command::new(env!("CARGO_BIN_EXE_ares"))
        .args(["slice", "-o"])
        .arg(&output)
        .arg(root.join("input.3mf"))
        .output()
        .unwrap();
    fs::write(root.join("cli.stdout"), &cli.stdout).unwrap();
    fs::write(root.join("cli.stderr"), &cli.stderr).unwrap();
    let cli_bytes = fs::read(&output);
    let cli_comparison = cli_bytes
        .as_ref()
        .map_err(|error| error.to_string())
        .and_then(|bytes| {
            crate::golden::compare_ordered_bytes_generator_only(&case.reference, bytes)
        });
    let api_layers_equal = result
        .as_ref()
        .is_ok_and(|bytes| layer_lines(bytes) == layer_lines(&case.reference));
    fs::write(root.join("comparison.json"), serde_json::to_vec_pretty(&json!({
        "label":case.label, "input_sha256":runner::cache_identity::raw_digest(&case.project),
        "native_sha256":runner::cache_identity::raw_digest(&case.reference),
        "native_lines":line_count(&case.reference),
        "api_admitted":result.is_ok(), "api_error":result.as_ref().err().map(|e|format!("{e:?}")),
        "api_comparison":api_comparison, "api_layers_equal":api_layers_equal,
        "api_lines":result.as_ref().ok().map(|b|line_count(b)),
        "cli_code":cli.status.code(), "cli_comparison":cli_comparison,
        "cli_lines":cli_bytes.as_ref().ok().map(|b|line_count(b)),
    })).unwrap()).unwrap();
    root
}

fn require_equal(root: &std::path::Path) {
    let result: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("comparison.json")).unwrap()).unwrap();
    assert_eq!(result["api_admitted"], true, "{root:?}: {result}");
    assert_eq!(result["cli_code"], 0, "{root:?}: {result}");
    assert_eq!(result["api_layers_equal"], true, "{root:?}: {result}");
    assert_eq!(
        result["api_lines"], result["native_lines"],
        "{root:?}: {result}"
    );
    assert_eq!(
        result["cli_lines"], result["native_lines"],
        "{root:?}: {result}"
    );
    assert_eq!(
        result["api_comparison"],
        json!({"Ok":null}),
        "{root:?}: {result}"
    );
    assert_eq!(
        result["cli_comparison"],
        json!({"Ok":null}),
        "{root:?}: {result}"
    );
}

fn line_count(bytes: &[u8]) -> usize {
    std::str::from_utf8(bytes).unwrap().lines().count()
}

fn layer_lines(bytes: &[u8]) -> Vec<&str> {
    std::str::from_utf8(bytes)
        .unwrap()
        .lines()
        .filter(|line| line.starts_with(";Z:") || line.starts_with(";HEIGHT:"))
        .collect()
}
