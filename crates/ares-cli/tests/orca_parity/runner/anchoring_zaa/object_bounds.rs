//! Shared public object-frame admission/activity; full acceptance remains strict.

use std::fs;

use super::{Case, Expectations, Recorded, assert_complete, observe, repeated};

pub(super) fn check(expected: &Expectations, case: Case, compensation: f64) {
    let Some(runs) = checked_runs(expected, case, compensation) else {
        return;
    };
    for run in &runs {
        assert_complete(run);
    }
}

pub(super) fn checked_runs(
    expected: &Expectations,
    case: Case,
    compensation: f64,
) -> Option<Vec<Recorded>> {
    let runs = repeated(expected, case)?;
    for run in &runs {
        let project = fs::read(run.root.join("input.3mf")).unwrap();
        let loaded = ares_core::load_project(&project).unwrap();
        assert_eq!(
            loaded.settings().process.object.xy_contour_compensation.0,
            compensation
        );
        let native = fs::read(run.root.join("orca.gcode")).unwrap();
        let api = fs::read(run.root.join("ares.gcode")).unwrap();
        let footer = format!("; xy_contour_compensation = {compensation}");
        assert!(
            std::str::from_utf8(&native)
                .unwrap()
                .lines()
                .any(|line| line == footer),
            "Native compensation footer: {:?}",
            run.root
        );
        assert!(
            !observe::positive_layers(&run.native_sparse).is_empty(),
            "Native positive Sparse activity: {:?}",
            run.root
        );
        assert!(
            !observe::positive_layers(&run.api_sparse).is_empty(),
            "API positive Sparse activity: {:?}",
            run.root
        );
        let first_sparse_layer = run.native_sparse[0].0;
        assert_eq!(
            initial_motion(&native, first_sparse_layer),
            initial_motion(&api, first_sparse_layer),
            "initial solid-only motion layers unchanged: {:?}",
            run.root
        );
    }
    Some(runs)
}

pub(super) fn xy(rows: &[(usize, String)]) -> Vec<(usize, String)> {
    rows.iter()
        .filter_map(|(layer, line)| {
            let coordinates: Vec<_> = line
                .split_ascii_whitespace()
                .filter(|token| token.starts_with('X') || token.starts_with('Y'))
                .collect();
            (!coordinates.is_empty()).then(|| (*layer, coordinates.join(" ")))
        })
        .collect()
}

fn initial_motion(bytes: &[u8], first_sparse_layer: usize) -> Vec<&str> {
    let mut layer = 0;
    std::str::from_utf8(bytes)
        .unwrap()
        .lines()
        .take_while(|line| {
            if *line == ";LAYER_CHANGE" {
                layer += 1;
            }
            layer < first_sparse_layer
        })
        .filter(|line| line.starts_with("G0 ") || line.starts_with("G1 "))
        .collect()
}
