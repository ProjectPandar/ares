//! FillCrossHatch.cpp:225 -> FillBase.hpp:52 / FillBase.cpp:1820-1828: dont_connect.
//! Authored STL inputs; Native streams/exported projects are runtime-only.
use crate::option_coverage::multiline_union::observation;
use crate::runner::{CaseInputs, OrcaRunner, ParityCase, application, repo_root};
use crate::{presets::VendorProfiles, runner};
use serde_json::json;
use std::fs;
#[path = "crosshatch_dont_connect_inputs.rs"]
mod inputs;

#[derive(Clone, Copy)]
struct Case {
    anchor_max: &'static str,
    multiline: &'static str,
}

macro_rules! witness {
    ($name:ident, $label:literal, $anchor_max:literal, $multiline:literal) => {
        #[test]
        fn $name() {
            compare(
                $label,
                Case {
                    anchor_max: $anchor_max,
                    multiline: $multiline,
                },
            );
        }
    };
}
witness!(crosshatch_no_anchor_multiline_1, "C0M1", "0", "1");
witness!(crosshatch_no_anchor_multiline_2, "C0M2", "0", "2");
witness!(crosshatch_short_anchor_multiline_1, "C049M1", "0.049", "1");
witness!(crosshatch_short_anchor_multiline_2, "C049M2", "0.049", "2");
witness!(
    crosshatch_threshold_anchor_multiline_1,
    "C05M1",
    "0.05",
    "1"
);
witness!(
    crosshatch_threshold_anchor_multiline_2,
    "C05M2",
    "0.05",
    "2"
);
witness!(
    crosshatch_unlimited_anchor_multiline_1,
    "C1000M1",
    "1000",
    "1"
);
witness!(
    crosshatch_unlimited_anchor_multiline_2,
    "C1000M2",
    "1000",
    "2"
);

fn compare(label: &str, case: Case) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let active = inputs::reference(&runner, label, case);
    let native = observation::observe(&active.reference);
    observation::require_sparse(&native);
    let root = evaluate(&active);
    fs::write(
        root.join("native.sparse.json"),
        serde_json::to_vec_pretty(&native).unwrap(),
    )
    .unwrap();
    require_equal(&root);
}

fn evaluate(case: &ParityCase) -> std::path::PathBuf {
    let root = tempfile::Builder::new()
        .prefix("crosshatch-dont-connect-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    fs::write(root.join("input.3mf"), &case.project).unwrap();
    fs::write(root.join("native.gcode"), &case.reference).unwrap();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let observed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        runtime.block_on(crate::runner::application::slice_project_plate_validated(
            &case.project,
            ares_core::GenerationMetadata::default(),
            None,
            &ares_core::ConfigOverrides::default(),
            ares_core::SlicerContext::new(0),
        ))
    }));
    let api_panicked = observed.is_err();
    let result = observed
        .map_err(|payload| {
            payload
                .downcast_ref::<String>()
                .cloned()
                .or_else(|| payload.downcast_ref::<&str>().map(|s| (*s).to_owned()))
                .unwrap_or_else(|| "non-string panic".to_owned())
        })
        .and_then(|result| result.map_err(|error| format!("{error:?}")));
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
    let cli = crate::runner::application::cli_command()
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
    let cli_layers_equal = cli_bytes
        .as_ref()
        .is_ok_and(|bytes| layer_lines(bytes) == layer_lines(&case.reference));
    fs::write(root.join("comparison.json"), serde_json::to_vec_pretty(&json!({
        "label":case.label, "input_sha256":runner::cache_identity::raw_digest(&case.project),
        "native_sha256":runner::cache_identity::raw_digest(&case.reference),
        "native_lines":line_count(&case.reference),
        "debug_assertions":cfg!(debug_assertions), "api_panicked":api_panicked,
        "api_admitted":result.is_ok(), "api_error":result.as_ref().err().map(|e|format!("{e:?}")),
        "api_comparison":api_comparison, "api_layers_equal":api_layers_equal,
        "api_lines":result.as_ref().ok().map(|b|line_count(b)),
        "sparse_first_difference":result.as_ref().ok().and_then(|bytes| sparse_first_difference(&case.reference, bytes)),
        "cli_sparse_first_difference":cli_bytes.as_ref().ok().and_then(|bytes| sparse_first_difference(&case.reference, bytes)),
        "api_cli_raw_equal":result.as_ref().ok().zip(cli_bytes.as_ref().ok()).is_some_and(|(a,c)|a==c),
        "cli_code":cli.status.code(), "cli_comparison":cli_comparison,
        "cli_layers_equal":cli_layers_equal,
        "cli_lines":cli_bytes.as_ref().ok().map(|b|line_count(b)),
    })).unwrap()).unwrap();
    root
}

fn require_equal(root: &std::path::Path) {
    let result: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("comparison.json")).unwrap()).unwrap();
    assert_eq!(result["api_panicked"], false, "{root:?}: {result}");
    assert_eq!(result["api_cli_raw_equal"], true, "{root:?}: {result}");
    assert_eq!(result["api_admitted"], true, "{root:?}: {result}");
    assert_eq!(result["cli_code"], 0, "{root:?}: {result}");
    assert_eq!(result["api_layers_equal"], true, "{root:?}: {result}");
    assert_eq!(result["cli_layers_equal"], true, "{root:?}: {result}");
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

fn sparse_first_difference(native: &[u8], ares: &[u8]) -> Option<serde_json::Value> {
    let native = sparse_records(native);
    let ares = sparse_records(ares);
    (0..native.len().max(ares.len())).find_map(|index| {
        let n = native.get(index);
        let a = ares.get(index);
        (n.map(|x| (x.0, x.2)) != a.map(|x| (x.0, x.2)))
            .then(|| json!({"index":index, "native":n, "ares":a}))
    })
}

fn sparse_records(bytes: &[u8]) -> Vec<(usize, usize, &str)> {
    let mut layer = 0;
    let mut sparse = false;
    std::str::from_utf8(bytes)
        .unwrap()
        .lines()
        .enumerate()
        .filter_map(|(index, line)| {
            if line == ";LAYER_CHANGE" {
                layer += 1;
            }
            if let Some(role) = line.strip_prefix(";TYPE:") {
                sparse = role == "Sparse infill";
            }
            (sparse
                && line.starts_with("G1 ")
                && line.contains('E')
                && (line.contains('X') || line.contains('Y')))
            .then_some((layer, index + 1, line))
        })
        .collect()
}
