use std::collections::BTreeSet;

pub(super) fn motion_lines(native: &str) -> impl Iterator<Item = &str> {
    let mut config = false;
    native.lines().filter(move |line| match *line {
        "; CONFIG_BLOCK_START" => {
            config = true;
            false
        }
        "; CONFIG_BLOCK_END" => {
            config = false;
            false
        }
        _ => !config,
    })
}

// Observe only genuine wall/sparse XY extrusion, excluding startup/flush moves
// and CONFIG_BLOCK template literals. Full comparison above is never filtered.
pub(super) fn observe(native: &str) -> BTreeSet<(usize, usize, usize, String)> {
    let mut layer = 0;
    let mut active = None;
    let mut role = "";
    let mut observations = BTreeSet::new();
    for line in motion_lines(native) {
        if line == "; CHANGE_LAYER" {
            layer += 1;
        }
        if let Some(marker) = line.strip_prefix("; G4_ACTIVE F") {
            let (filament, nozzle) = marker.split_once(" N").unwrap();
            active = Some((
                filament.parse::<usize>().unwrap(),
                nozzle.parse::<usize>().unwrap(),
            ));
        }
        if let Some(value) = line.strip_prefix("; FEATURE: ") {
            role = value;
        }
        if line.starts_with("G1 ") && matches!(role, "Outer wall" | "Inner wall" | "Sparse infill")
        {
            let fields = line.split_ascii_whitespace().collect::<Vec<_>>();
            let xy = fields
                .iter()
                .any(|field| field.starts_with('X') || field.starts_with('Y'));
            let extrusion = fields
                .iter()
                .filter_map(|field| field.strip_prefix('E'))
                .any(|value| value.parse::<f64>().unwrap() > 0.0);
            if xy && extrusion {
                let (filament, nozzle) = active.unwrap();
                observations.insert((layer, filament, nozzle, role.to_owned()));
            }
        }
    }
    observations
}
