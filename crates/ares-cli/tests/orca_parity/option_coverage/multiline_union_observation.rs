use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Serialize)]
pub(crate) struct Observation {
    sparse_layers: BTreeMap<usize, usize>,
    wall_min: [f64; 2],
    wall_max: [f64; 2],
    sparse_endpoints: Vec<[f64; 2]>,
}

pub(crate) fn observe(bytes: &[u8]) -> Observation {
    let mut result = Observation {
        sparse_layers: BTreeMap::new(),
        wall_min: [f64::INFINITY; 2],
        wall_max: [f64::NEG_INFINITY; 2],
        sparse_endpoints: Vec::new(),
    };
    let mut role = "";
    let mut layer = 0;
    let mut relative = false;
    let mut position = [0.0_f64; 2];
    let mut absolute_e = 0.0;
    for line in std::str::from_utf8(bytes).unwrap().lines() {
        if let Some(value) = line.strip_prefix(";TYPE:") {
            role = value;
        }
        if line == ";LAYER_CHANGE" {
            layer += 1;
        }
        let mut words = line.split(';').next().unwrap().split_ascii_whitespace();
        let command = words.next();
        match command {
            Some("M83") => relative = true,
            Some("M82") => relative = false,
            _ => {}
        }
        if !matches!(command, Some("G0" | "G1" | "G92")) {
            continue;
        }
        let mut next = position;
        let mut xy = false;
        let mut extrusion = None;
        for word in words {
            if let Some(value) = word.strip_prefix('X') {
                next[0] = value.parse().unwrap();
                xy = true;
            }
            if let Some(value) = word.strip_prefix('Y') {
                next[1] = value.parse().unwrap();
                xy = true;
            }
            if let Some(value) = word.strip_prefix('E') {
                extrusion = Some(value.parse::<f64>().unwrap());
            }
        }
        let e = extrusion.map(|e| {
            let delta = if relative { e } else { e - absolute_e };
            absolute_e = e;
            delta
        });
        if command != Some("G92") && xy && e.is_some_and(|e| e > 0.0) {
            record(&mut result, role, layer, position, next);
        }
        position = next;
    }
    result
}

fn record(result: &mut Observation, role: &str, layer: usize, start: [f64; 2], end: [f64; 2]) {
    if role == "Sparse infill" {
        *result.sparse_layers.entry(layer).or_default() += 1;
        result.sparse_endpoints.extend([start, end]);
    }
    if role == "Outer wall" {
        for point in [start, end] {
            for (axis, value) in point.into_iter().enumerate() {
                result.wall_min[axis] = result.wall_min[axis].min(value);
                result.wall_max[axis] = result.wall_max[axis].max(value);
            }
        }
    }
}

pub(crate) fn require_sparse(result: &Observation) {
    let phases = result
        .sparse_layers
        .keys()
        .map(|id| id % 3)
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(phases.len(), 3, "missing actual sparse infill layer phases");
    assert!(result.sparse_layers.values().all(|count| *count > 0));
}

pub(crate) fn require_hole_clipping(result: &Observation) {
    let center = [
        (result.wall_min[0] + result.wall_max[0]) / 2.0,
        (result.wall_min[1] + result.wall_max[1]) / 2.0,
    ];
    // Consumer witnesses, not output-comparison tolerances: positive-E endpoints
    // reach both the 8mm hole neighborhood and the exterior of the 64x48mm mesh.
    let hole = result
        .sparse_endpoints
        .iter()
        .any(|p| (p[0] - center[0]).abs() < 6.0 && (p[1] - center[1]).abs() < 6.0);
    let exterior = result
        .sparse_endpoints
        .iter()
        .any(|p| (p[0] - center[0]).abs() > 29.0 || (p[1] - center[1]).abs() > 21.0);
    assert!(
        hole && exterior,
        "missing hole/exterior sparse clipping witnesses"
    );
}
