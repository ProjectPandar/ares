/// Positive layer2 Sparse infill extrusion: start XY, end XY, relative E.
/// Fill.cpp:1235-1242 -> FillRectilinear.cpp:2762 / FillBase.cpp:311-313.
pub(super) fn sparse(bytes: &[u8]) -> Vec<[f64; 5]> {
    let mut layer = 0;
    let mut role = "";
    let mut position = [0.0; 2];
    let mut result = Vec::new();
    for line in std::str::from_utf8(bytes).unwrap().lines() {
        if line == ";LAYER_CHANGE" {
            layer += 1;
        }
        if let Some(next) = line.strip_prefix(";TYPE:") {
            role = next;
        }
        if !line.starts_with("G1 ") && !line.starts_with("G0 ") {
            continue;
        }
        let start = position;
        let mut has_xy = false;
        let mut extrusion = 0.0;
        for token in line.split(';').next().unwrap().split_whitespace().skip(1) {
            if let Some(value) = token.strip_prefix('X') {
                position[0] = value.parse().unwrap();
                has_xy = true;
            } else if let Some(value) = token.strip_prefix('Y') {
                position[1] = value.parse().unwrap();
                has_xy = true;
            } else if let Some(value) = token.strip_prefix('E') {
                extrusion = value.parse().unwrap();
            }
        }
        if layer == 2 && role == "Sparse infill" && has_xy && extrusion > 0.0 {
            result.push([start[0], start[1], position[0], position[1], extrusion]);
        }
    }
    result
}
