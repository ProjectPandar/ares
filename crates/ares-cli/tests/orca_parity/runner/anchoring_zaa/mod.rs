//! Shared public sparse-fill checks; strict acceptance includes all M73/feedrates.

use crate::runner::OrcaRunner;
use serde_json::json;
use std::fs;

mod case_runner;
#[path = "../combine_infill_region_view.rs"]
mod combine_infill_region_view;
#[path = "../combine_solid_clearance.rs"]
mod combine_solid_clearance;
mod complete;
use complete::{Recorded, assert_complete};
#[path = "../fill_group_flow_ratio.rs"]
mod fill_group_flow_ratio;
#[path = "../seam_gap_fill_group.rs"]
mod seam_gap_fill_group;

#[path = "../consistent_grid_object_bounds.rs"]
mod consistent_grid_object_bounds;
#[path = "../honeycomb_anchoring_zaa.rs"]
mod honeycomb_anchoring_zaa;
#[path = "../line_anchoring_zaa.rs"]
mod line_anchoring_zaa;
mod object_bounds;
mod observe;
#[path = "../plane_path_object_bounds.rs"]
mod plane_path_object_bounds;
#[path = "../tpmsd_anchoring_contraction.rs"]
mod tpmsd_anchoring_contraction;

#[derive(Clone, Copy)]
struct Case {
    name: &'static str,
    sha: &'static str,
    top: usize,
    enabled: bool,
    flag: bool,
}

struct Expectations {
    input_dir: &'static str,
    label: &'static str,
    pattern: ares_core::ProcessInfillPattern,
    multiline: i32,
    footer: [&'static str; 2],
}

fn frozen(expected: &Expectations, case: Case) -> Vec<u8> {
    let project = fs::read(
        crate::runner::repo_root()
            .join(expected.input_dir)
            .join(format!("{}.3mf", case.name)),
    )
    .unwrap();
    assert_eq!(
        observe::hash(&project),
        case.sha,
        "frozen input {}",
        case.name
    );
    let loaded = ares_core::load_project(&project).unwrap();
    assert_eq!(loaded.objects().len(), 1);
    assert_eq!(loaded.objects()[0].instances().len(), 1);
    let region = &loaded.settings().process.region;
    assert_eq!(region.zaa_enabled.0, case.enabled);
    assert_eq!(region.zaa_dont_alternate_fill_direction.0, case.flag);
    assert_eq!(region.sparse_infill_pattern, expected.pattern);
    assert_eq!(region.fill_multiline.0, expected.multiline);
    assert_eq!(
        usize::try_from(region.top_shell_layers.0).unwrap(),
        case.top
    );
    project
}

fn verify(expected: &Expectations, case: Case) {
    let Some(runs) = repeated(expected, case) else {
        return;
    };
    for run in &runs {
        let (native_sparse, api_sparse, root) = (&run.native_sparse, &run.api_sparse, &run.root);
        assert_eq!(
            native_sparse, api_sparse,
            "ordinary Sparse unchanged: {root:?}"
        );
        assert_complete(run);
    }
}

fn repeated(expected: &Expectations, case: Case) -> Option<Vec<Recorded>> {
    let mut runner = OrcaRunner::from_env().unwrap()?;
    let project = frozen(expected, case);
    let runs: Vec<_> = (1..=2)
        .map(|repeat| run_once(&mut runner, expected, case, repeat, &project))
        .collect();
    complete::assert_repeated(&runs);
    Some(runs)
}
fn run_once(
    runner: &mut OrcaRunner,
    expected: &Expectations,
    case: Case,
    repeat: usize,
    project: &[u8],
) -> Recorded {
    let run = complete::run_once(runner, expected.label, case.name, repeat, project);
    let root = &run.root;
    let native = &run.native;
    let api_bytes = &run.api_bytes;
    let cli_bytes = &run.cli_bytes;
    let native_sha = &run.native_sha;
    let api_sha = &run.api_sha;
    let cli_difference = &run.cli_difference;
    let native_sparse = &run.native_sparse;
    let api_sparse = &run.api_sparse;
    let native_bridge = observe::moves(native, "Internal Bridge");
    let api_bridge = observe::moves(api_bytes, "Internal Bridge");
    let native_layers = observe::positive_layers(&native_bridge);
    let api_layers = observe::positive_layers(&api_bridge);
    fs::write(
        root.join("anchoring-zaa-witness.json"),
        serde_json::to_vec_pretty(&json!({
            "case": case.name, "repeat": repeat, "input_sha256": observe::hash(project),
            "enabled": case.enabled, "flag": case.flag, "top_shell_layers": case.top,
            "native_normalized_sha256": native_sha, "api_sha256": api_sha,
            "cli_sha256": observe::hash(cli_bytes), "cli_exit": run.cli_exit,
            "native_bridge_layers": native_layers, "api_bridge_layers": api_layers,
            "native_bridge": native_bridge, "api_bridge": api_bridge,
            "native_sparse_rows": native_sparse.len(), "api_sparse_rows": api_sparse.len(),
            "sparse_equal": native_sparse == api_sparse, "api_cli_equal": api_bytes == cli_bytes,
            "fresh_native_cache": runner.cache, "status": run.status,
            "detail": run.detail, "cli_difference": cli_difference
        }))
        .unwrap(),
    )
    .unwrap();
    let reference = std::str::from_utf8(native).unwrap();
    for line in [
        "; CONFIG_BLOCK_END".to_owned(),
        expected.footer[0].to_owned(),
        "; sparse_infill_density = 20%".to_owned(),
        "; infill_direction = 45".to_owned(),
        expected.footer[1].to_owned(),
        format!("; top_shell_layers = {}", case.top),
        format!("; zaa_enabled = {}", u8::from(case.enabled)),
        format!(
            "; zaa_dont_alternate_fill_direction = {}",
            u8::from(case.flag)
        ),
    ] {
        assert!(
            reference.lines().any(|found| found == line),
            "Native effective footer: {line}; {root:?}"
        );
    }
    // 10mm / 0.2mm = 50 object layers; top shells begin at physical 51 - count.
    let bridge_layers = std::collections::BTreeSet::from([51 - case.top]);
    assert_eq!(
        native_layers, bridge_layers,
        "Native bridge active: {root:?}"
    );
    assert_eq!(api_layers, bridge_layers, "API bridge active: {root:?}");
    assert!(!native_sparse.is_empty(), "Native Sparse active: {root:?}");
    run
}
