//! FillRectilinear.cpp:3048-3085,3160-3257,3344-3384,3466-3484.
//! Authored STL inputs; Native streams/exported projects are runtime-only.
use super::{CaseInputs, OrcaRunner, ParityCase, application, repo_root};
use crate::option_coverage::multiline_union::observation;
use crate::{presets::VendorProfiles, runner};
use serde_json::json;
use std::fs;
#[path = "triangles_multiline_mode1_inputs.rs"]
mod inputs;

#[derive(Clone, Copy)]
struct Case {
    pattern: &'static str,
    count: &'static str,
    angle: &'static str,
    density: &'static str,
    hole: bool,
    combined: bool,
}

macro_rules! control {
    ($name:ident, $label:literal, $pattern:literal, $count:literal, $angle:literal, $density:literal, $hole:literal, $combined:literal) => {
        #[test]
        fn $name() {
            compare(
                $label,
                Case {
                    pattern: $pattern,
                    count: $count,
                    angle: $angle,
                    density: $density,
                    hole: $hole,
                    combined: $combined,
                },
            );
        }
    };
}
control!(
    triangles_mode1_t1,
    "T1",
    "triangles",
    "1",
    "45",
    "30%",
    false,
    false
);
control!(
    triangles_mode1_t2,
    "T2",
    "triangles",
    "2",
    "45",
    "30%",
    false,
    false
);
control!(
    triangles_mode1_t3,
    "T3",
    "triangles",
    "3",
    "45",
    "30%",
    false,
    false
);
control!(
    triangles_mode1_t10,
    "T10",
    "triangles",
    "10",
    "45",
    "30%",
    false,
    false
);
control!(
    triangles_mode1_t2a0,
    "T2A0",
    "triangles",
    "2",
    "0",
    "30%",
    false,
    false
);
control!(
    triangles_mode1_t2d20,
    "T2D20",
    "triangles",
    "2",
    "45",
    "20%",
    false,
    false
);
control!(
    triangles_mode1_th1,
    "TH1",
    "triangles",
    "1",
    "45",
    "30%",
    true,
    false
);
control!(
    triangles_mode1_th2,
    "TH2",
    "triangles",
    "2",
    "45",
    "30%",
    true,
    false
);
control!(
    triangles_mode1_t2c2,
    "T2C2",
    "triangles",
    "2",
    "45",
    "30%",
    false,
    true
);
control!(
    triangles_mode1_g2,
    "G2",
    "grid",
    "2",
    "45",
    "30%",
    false,
    false
);
control!(
    triangles_mode1_s2,
    "S2",
    "tri-hexagon",
    "2",
    "45",
    "30%",
    false,
    false
);
control!(
    triangles_mode1_r1,
    "R1",
    "rectilinear",
    "1",
    "45",
    "30%",
    false,
    false
);

fn compare(label: &str, case: Case) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let eligible = case.pattern == "triangles" && case.count != "1";
    let baseline = eligible.then(|| {
        inputs::reference(
            &runner,
            &format!("{label}/count1"),
            Case { count: "1", ..case },
        )
    });
    if let Some(baseline) = &baseline {
        observation::require_sparse(&observation::observe(&baseline.reference));
        require_equal(&evaluate(baseline));
    }
    let active = inputs::reference(&runner, label, case);
    let native = observation::observe(&active.reference);
    observation::require_sparse(&native);
    if case.hole {
        observation::require_hole_clipping(&native);
    }
    if case.combined {
        assert!(
            has_combined_sparse(&active.reference),
            "Native must extrude actual combined sparse layers"
        );
    }
    if let Some(baseline) = &baseline {
        assert_ne!(
            sparse_commands(&baseline.reference),
            sparse_commands(&active.reference),
            "{label}: actual Native multiline consumption"
        );
    }
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
        .prefix("triangles-mode1-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    fs::write(root.join("input.3mf"), &case.project).unwrap();
    fs::write(root.join("native.gcode"), &case.reference).unwrap();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let result = runtime.block_on(crate::runner::application::slice_project_plate_validated(
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
    fs::write(root.join("comparison.json"), serde_json::to_vec_pretty(&json!({
        "label":case.label, "input_sha256":runner::cache_identity::raw_digest(&case.project),
        "native_sha256":runner::cache_identity::raw_digest(&case.reference),
        "native_lines":line_count(&case.reference),
        "api_admitted":result.is_ok(), "api_error":result.as_ref().err().map(|e|format!("{e:?}")),
        "api_comparison":api_comparison, "api_layers_equal":api_layers_equal,
        "api_lines":result.as_ref().ok().map(|b|line_count(b)),
        "sparse_first_difference":result.as_ref().ok().and_then(|bytes| sparse_first_difference(&case.reference, bytes)),
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

fn sparse_commands(bytes: &[u8]) -> Vec<&str> {
    let mut sparse = false;
    std::str::from_utf8(bytes)
        .unwrap()
        .lines()
        .filter(|line| {
            if let Some(role) = line.strip_prefix(";TYPE:") {
                sparse = role == "Sparse infill";
            }
            sparse
                && line.starts_with("G1 ")
                && line.contains('E')
                && (line.contains('X') || line.contains('Y'))
        })
        .collect()
}

fn sparse_first_difference(native: &[u8], ares: &[u8]) -> Option<serde_json::Value> {
    let native = sparse_records(native);
    let ares = sparse_records(ares);
    (0..native.len().max(ares.len())).find_map(|index| {
        let n = native.get(index);
        let a = ares.get(index);
        (n != a).then(|| json!({"index":index, "native":n, "ares":a}))
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

fn has_combined_sparse(bytes: &[u8]) -> bool {
    let mut height = 0.0_f64;
    let mut sparse = false;
    std::str::from_utf8(bytes).unwrap().lines().any(|line| {
        if let Some(value) = line.strip_prefix(";HEIGHT:") {
            height = value.parse().unwrap();
        }
        if let Some(role) = line.strip_prefix(";TYPE:") {
            sparse = role == "Sparse infill";
        }
        sparse
            && height == 0.4
            && line.starts_with("G1 ")
            && line.contains('E')
            && (line.contains('X') || line.contains('Y'))
    })
}
