use serde_json::{Value, json};

pub(super) struct Activity {
    pub(super) xyz: Vec<Value>,
    pub(super) modifier_xyz: usize,
    pub(super) eligible_sha256: String,
}

pub(super) fn observe(bytes: &[u8]) -> Activity {
    let mut role = "";
    let mut layer = 0;
    let mut position = [0.0; 3];
    let mut xyz = Vec::new();
    let mut eligible = Vec::new();
    let mut modifier_xyz = 0;
    for line in std::str::from_utf8(bytes).unwrap().lines() {
        if line == ";LAYER_CHANGE" {
            layer += 1;
        }
        if let Some(next) = line.strip_prefix(";TYPE:") {
            role = next;
        }
        if !line.starts_with("G1 ") {
            continue;
        }
        let mut has_xy = false;
        let mut has_z = false;
        let mut extrusion = 0.0;
        for token in line.split_ascii_whitespace().skip(1) {
            if token.starts_with(';') {
                break;
            }
            let (axis, value) = token.split_at(1);
            match axis {
                "X" | "Y" | "Z" => {
                    let index = match axis {
                        "X" => 0,
                        "Y" => 1,
                        _ => 2,
                    };
                    position[index] = value.parse::<f64>().unwrap();
                    has_xy |= index < 2;
                    has_z |= index == 2;
                }
                "E" => extrusion = value.parse::<f64>().unwrap(),
                _ => {}
            }
        }
        if extrusion <= 0.0
            || !has_xy
            || !matches!(role, "Outer wall" | "Inner wall" | "Top surface")
        {
            continue;
        }
        eligible.push(json!([layer, role, line]));
        if has_z {
            let inside =
                (105.0..=110.0).contains(&position[0]) && (105.0..=115.0).contains(&position[1]);
            modifier_xyz += usize::from(inside);
            xyz.push(
                json!({"layer":layer,"role":role,"line":line,"position":position,
                "inside_modifier":inside}),
            );
        }
    }
    Activity {
        xyz,
        modifier_xyz,
        eligible_sha256: super::super::fixture::hash(&serde_json::to_vec(&eligible).unwrap()),
    }
}
