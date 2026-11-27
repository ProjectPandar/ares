use super::organic_support::{matches_reference, native_slice, project, reference};

const SHELF: &[u8] = include_bytes!("../../../../../tests/parity/belt-tree-support/slim-shelf.3mf");

#[tokio::test]
async fn slim_tree_can_land_on_a_model_shelf_without_losing_belt_support() {
    let Some(runner) = reference() else { return };
    let root = runner.work_dir();
    let model = root.join("slim-model-landing");
    let plate = root.join("slim-plate-only");
    let model_project = project(SHELF, &[("support_on_build_plate_only", "0")], false);
    let native_model = native_slice(&runner, &model, &model_project);
    let native_plate = native_slice(&runner, &plate, SHELF);
    let (over_shelf, right_wing) = landing_motion(&native_model);
    let (plate_over_shelf, plate_right_wing) = landing_motion(&native_plate);
    assert!(
        over_shelf,
        "Native did not print a shelf interface: {model:?}"
    );
    assert!(
        !plate_over_shelf,
        "plate-only Native printed a shelf interface: {plate:?}"
    );
    assert!(
        right_wing && plate_right_wing,
        "missing Native belt support: {root:?}"
    );
    matches_reference(&native_model, &model, &model_project).await;
    matches_reference(&native_plate, &plate, SHELF).await;
}

fn landing_motion(gcode: &str) -> (bool, bool) {
    let mut xy = [0.0; 2];
    let mut role = "";
    let mut relative_e = false;
    let mut relative_xyz = false;
    let mut shelf = false;
    let mut right = false;
    for line in gcode
        .lines()
        .take_while(|line| *line != "; CONFIG_BLOCK_START")
    {
        if let Some(next_role) = line.strip_prefix(";TYPE:") {
            role = next_role;
        }
        let mut words = line.split(';').next().unwrap().split_ascii_whitespace();
        let Some(command) = words.next() else {
            continue;
        };
        match command {
            "M82" => relative_e = false,
            "M83" => relative_e = true,
            "G90" => relative_xyz = false,
            "G91" => relative_xyz = true,
            "G0" | "G1" | "G92" => {
                let start = xy;
                let extrudes = apply_move(words, &mut xy, relative_xyz && command != "G92");
                if command == "G92" || !relative_e || !extrudes || start == xy {
                    continue;
                }
                // GCode/MachineFrameTransform.cpp:20–57: Y is twice the model
                // height on this 30° X belt. Reverse X puts the 3 mm shelf on
                // the larger-X side and the wing with no shelf on the other.
                if role == "Support interface" {
                    shelf |= [start, xy]
                        .into_iter()
                        .all(|[x, y]| x > 115.0 && x < 125.0 && (6.0..8.0).contains(&y));
                } else if role == "Support" {
                    right |= [start, xy].into_iter().all(|[x, _]| x > 92.0 && x < 105.0);
                }
            }
            _ => {}
        }
    }
    (shelf, right)
}

fn apply_move<'a>(words: impl Iterator<Item = &'a str>, xy: &mut [f64; 2], relative: bool) -> bool {
    let mut extrudes = false;
    for word in words {
        let (key, value) = word.split_at(1);
        if key == "E" {
            extrudes = value.parse::<f64>().is_ok_and(|e| e > 0.0);
        } else if let Some(axis) = ["X", "Y"].iter().position(|&axis| axis == key) {
            let value = value.parse::<f64>().unwrap();
            xy[axis] = if relative { xy[axis] + value } else { value };
        }
    }
    extrudes
}
