/// Positive layer47 Internal Bridge extrusion: start XY, end XY, relative E.
/// Fill.cpp:1440-1447 -> PrintObject.cpp:2735-2754/3244-3250.
pub(super) fn bridge(bytes: &[u8]) -> Vec<[f64; 5]> {
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
        if layer == 47 && role == "Internal Bridge" && has_xy && extrusion > 0.0 {
            result.push([start[0], start[1], position[0], position[1], extrusion]);
        }
    }
    result
}
