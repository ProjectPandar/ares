//! Print.cpp:932-979 and GCode.cpp:5215-5236,5302-5305: project wrapping.
use super::*;
use serde_json::json;

#[path = "wrapping_exclude_coverage/input.rs"]
pub(super) mod input;
#[path = "wrapping_exclude_coverage/rejections.rs"]
mod rejections;

#[test]
fn orca_parity_wrapping_exclude_area_bounded_project_output() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let plan = domains::load(&runner::repo_root())
        .into_iter()
        .find(|plan| plan.key == "wrapping_exclude_area")
        .unwrap();
    let requests = [
        json!([]),
        json!(["0x0"]),
        json!(["0x0", "10x0"]),
        json!(["0x0", "10x0", "0x10"]),
        json!(["0x0", "10x0", "10x10", "0x10"]),
    ];
    let root = parity::artifacts::root_from_env().unwrap();
    let evidence = root.join("wrapping-exclude-area-bounded.json");
    let mut proofs = Vec::new();
    for enabled in [false, true] {
        for (index, points) in requests.iter().enumerate() {
            let label = format!("bounded-wrapping/{enabled}/{index}");
            let exported = input::export(&runner, points, enabled, &label).unwrap();
            let bounds = footprint(&exported.project);
            assert!(bounds[0] > 10.0 && bounds[1] > 10.0, "{bounds:?}");
            let sliced = runner.slice_case(&exported).unwrap();
            let native = std::str::from_utf8(&sliced.reference).unwrap();
            assert!(
                !native
                    .lines()
                    .any(|line| line == ";WIPE_TOWER_START" || line == ";TYPE:Wipe tower"),
                "{label}: unexpected tower"
            );
            let markers = native
                .lines()
                .filter(|line| line.starts_with(";W6_WRAP "))
                .collect::<Vec<_>>();
            let layers = native
                .lines()
                .filter(|line| *line == ";LAYER_CHANGE")
                .count();
            assert!(layers > 0, "{label}: no layers");
            assert_eq!(markers.len(), if enabled { layers } else { 0 }, "{label}");
            assert!(
                native.lines().any(positive_extrusion),
                "{label}: no extrusion"
            );
            let outcome = parity::compare_case(&sliced);
            proofs.push(json!({"label":label,"requested":points,"enabled":enabled,
                "footprint":bounds,"layers":layers,"marker_lines":markers,
                "full_output_status":outcome.status,"detail":outcome.detail,
                "artifacts":outcome.artifacts}));
            parity::artifacts::overwrite(&evidence, &serde_json::to_vec_pretty(&proofs).unwrap())
                .unwrap();
            assert_eq!(
                outcome.status, "PASS",
                "{label}: {}; {root:?}",
                outcome.detail
            );
        }
    }
    assert_eq!(
        plan.cases
            .iter()
            .map(|case| case.value.clone())
            .collect::<Vec<_>>(),
        requests.into_iter().map(Some).collect::<Vec<_>>(),
        "declared requests must match runtime witnesses"
    );
    let outcome = OptionOutcome::executed(&plan, plan.cases.len(), Vec::new(), 0, None);
    parity::artifacts::overwrite(
        &evidence,
        &serde_json::to_vec_pretty(&json!({
            "cases":proofs,"classification":outcome.status,"detail":outcome.detail
        }))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(outcome.status, "PASS", "{}; {root:?}", outcome.detail);
}

fn positive_extrusion(line: &str) -> bool {
    line.starts_with("G1 ")
        && line
            .split_whitespace()
            .any(|word| word.starts_with(['X', 'Y']))
        && line
            .split_whitespace()
            .filter_map(|word| word.strip_prefix('E'))
            .any(|amount| amount.parse::<f64>().unwrap() > 0.0)
}

fn footprint(project: &[u8]) -> [f64; 4] {
    let loaded = ares_core::load_project(project).unwrap();
    assert_eq!(loaded.objects().len(), 1);
    let object = &loaded.objects()[0];
    assert_eq!(object.instances().len(), 1);
    assert_eq!(object.volumes().len(), 1);
    let volume = &object.volumes()[0];
    let transform = object.instances()[0].transform().then(volume.transform());
    let mut bounds = [
        f64::INFINITY,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::NEG_INFINITY,
    ];
    for vertex in volume.mesh().vertices() {
        let point = transform.transform_point(*vertex);
        bounds[0] = bounds[0].min(point.x);
        bounds[1] = bounds[1].min(point.y);
        bounds[2] = bounds[2].max(point.x);
        bounds[3] = bounds[3].max(point.y);
    }
    bounds
}
