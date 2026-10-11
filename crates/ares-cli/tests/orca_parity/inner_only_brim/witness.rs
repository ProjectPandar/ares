//! Diagnostic geometric witnesses supplement, never replace, full byte equality.
use serde_json::{Value, json};
use std::path::Path;

fn in_hole(point: [f64; 2], holes: &Value) -> bool {
    holes.as_array().unwrap().iter().any(|hole| {
        let polygon = hole.as_array().unwrap();
        let mut hit = false;
        for (a, b) in polygon.iter().zip(polygon.iter().cycle().skip(1)) {
            let [ax, ay] = [
                a[0].as_f64().unwrap() + 110.0,
                a[1].as_f64().unwrap() + 110.0,
            ];
            let [bx, by] = [
                b[0].as_f64().unwrap() + 110.0,
                b[1].as_f64().unwrap() + 110.0,
            ];
            if (ay > point[1]) != (by > point[1])
                && point[0] < ax + (point[1] - ay) * (bx - ax) / (by - ay)
            {
                hit = !hit;
            }
        }
        hit
    })
}

pub(super) fn capture(bytes: &[u8], case: &Value) -> Value {
    let mut position = [0.0; 3];
    let mut extrusion = 0.0;
    let mut relative_e = false;
    let mut role = "";
    let mut motions = Vec::new();
    for line in std::str::from_utf8(bytes).unwrap().lines() {
        if let Some(value) = line.strip_prefix(";TYPE:") {
            role = value;
        }
        let mut words = line.split(';').next().unwrap().split_whitespace();
        let Some(command) = words.next() else {
            continue;
        };
        if command == "M83" {
            relative_e = true;
        }
        if command == "M82" {
            relative_e = false;
        }
        if !matches!(command, "G0" | "G1" | "G92") {
            continue;
        }
        let start = position;
        let mut e_value = None;
        for word in words {
            let (axis, number) = word.split_at(1);
            let Ok(value) = number.parse::<f64>() else {
                continue;
            };
            match axis {
                "X" => position[0] = value,
                "Y" => position[1] = value,
                "Z" => position[2] = value,
                "E" => e_value = Some(value),
                _ => {}
            }
        }
        let delta_e = match e_value {
            Some(value) if relative_e => value,
            Some(value) => value - extrusion,
            None => 0.0,
        };
        if let Some(value) = e_value {
            extrusion = value;
        }
        if command == "G1"
            && role == "Brim"
            && delta_e > 0.0
            && (start[0] != position[0] || start[1] != position[1])
        {
            let a = [start[0], start[1]];
            let b = [position[0], position[1]];
            let midpoint = [(a[0] + b[0]) / 2.0, (a[1] + b[1]) / 2.0];
            motions.push(json!({"line":line,"start":a,"end":b,"z":position[2],
                "positive_e":delta_e,"first_layer":position[2] == 0.2,
                "inside_input_holes": ([a, midpoint, b].into_iter().all(|p| in_hole(p, &case["holes_local_mm"])))}));
        }
    }
    json!({"role":"Brim","scope":"positive XY/E, first layer, endpoints and midpoint in input holes",
        "motions":motions})
}

pub(super) fn verify(witness: &Value, case: &Value, root: &Path) {
    let motions = witness["motions"].as_array().unwrap();
    assert_eq!(
        !motions.is_empty(),
        case["expect_brim"].as_bool().unwrap(),
        "Native/public brim activation: {root:?}"
    );
    for motion in motions {
        assert_eq!(
            motion["first_layer"], true,
            "brim layer: {root:?}: {motion}"
        );
        assert_eq!(
            motion["inside_input_holes"], true,
            "brim in hole: {root:?}: {motion}"
        );
    }
}
