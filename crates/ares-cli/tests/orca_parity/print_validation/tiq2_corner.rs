//! Fixture-only corner anchors; WipeTower.cpp:1733-1750 brim arithmetic.
use std::{
    fs,
    io::{Cursor, Read, Write},
    path::Path,
};

use ares_core::{PlaterSession, ProjectVolumeType};
use serde_json::{Value, json};

#[derive(Clone, Copy)]
pub(crate) enum Scenario {
    RetainedFan,
    WholeFile,
    Slots,
    Unused,
    Assignment,
}

impl Scenario {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::RetainedFan => "retained-role-fan",
            Self::WholeFile => "whole-file-footer",
            Self::Slots => "slot-index",
            Self::Unused => "unused-slot",
            Self::Assignment => "assignment",
        }
    }
}

pub(crate) fn corner_input(original: &[u8], scenario: Scenario, directory: &Path) -> Vec<u8> {
    let mut archive = zip::ZipArchive::new(Cursor::new(original)).unwrap();
    let mut entries = Vec::new();
    for index in 0..archive.len() {
        let mut file = archive.by_index(index).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        entries.push((file.name().to_owned(), file.compression(), bytes));
    }
    let settings_index = entries
        .iter()
        .position(|entry| entry.0 == "Metadata/project_settings.config")
        .unwrap();
    let mut settings: Value = serde_json::from_slice(&entries[settings_index].2).unwrap();
    match scenario {
        Scenario::RetainedFan | Scenario::WholeFile => {}
        Scenario::Slots | Scenario::Unused | Scenario::Assignment => {
            settings["single_extruder_multi_material"] = "0".into();
            settings["nozzle_temperature"] = json!(["211", "233"]);
            settings["filament_end_gcode"] = json!([
                "; FOOTER_SLOT0 current={current_extruder} slot={filament_extruder_id} temp=[nozzle_temperature]\n",
                "; FOOTER_SLOT1 current={current_extruder} slot={filament_extruder_id} temp=[nozzle_temperature]\n"
            ]);
            settings["machine_end_gcode"] = "; FOOTER_MACHINE current={current_extruder}\n".into();
            match scenario {
                Scenario::Unused => {
                    settings["support_filament"] = "1".into();
                    settings["support_interface_filament"] = "1".into();
                }
                Scenario::Assignment => {
                    settings["filament_end_gcode"] = json!([
                        "{position[0]=123}\n; FOOTER_SLOT0 x={position[0]}\n",
                        "; FOOTER_SLOT1 x={position[0]}\n{position[0]=124}\n"
                    ]);
                    settings["machine_end_gcode"] = "; FOOTER_MACHINE x={position[0]}\n".into();
                }
                Scenario::Slots => {}
                Scenario::RetainedFan | Scenario::WholeFile => unreachable!(),
            }
        }
    }
    let before = settings.clone();
    assert!(
        settings["extruder_printable_area"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    let points: Vec<[f64; 2]> = settings["printable_area"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| {
            let (x, y) = value.as_str().unwrap().split_once('x').unwrap();
            [x.parse().unwrap(), y.parse().unwrap()]
        })
        .collect();
    assert_eq!(points.len(), 4);
    let bounds = [0, 1].map(|axis| {
        [
            points.iter().map(|p| p[axis]).fold(f64::INFINITY, f64::min),
            points
                .iter()
                .map(|p| p[axis])
                .fold(f64::NEG_INFINITY, f64::max),
        ]
    });
    assert!(points.iter().all(|p| {
        [0, 1]
            .into_iter()
            .all(|axis| bounds[axis].contains(&p[axis]))
    }));
    let session = PlaterSession::load(original.to_vec()).unwrap();
    let plates = session.plates();
    assert_eq!(plates.len(), 1);
    let plate = &plates[0];
    let mut bbox = [[f64::INFINITY; 3], [f64::NEG_INFINITY; 3]];
    for &(object_index, instance_index) in &plate.instances {
        let object = &session.project().objects()[object_index];
        let instance = &object.instances()[instance_index];
        for volume in object.volumes() {
            if volume.volume_type() != ProjectVolumeType::ModelPart {
                continue;
            }
            for point in volume.mesh().vertices() {
                let p = instance
                    .transform()
                    .transform_point(volume.transform().transform_point(*point));
                let p = [p.x - plate.origin[0], p.y - plate.origin[1], p.z];
                bbox[0] = std::array::from_fn(|axis| bbox[0][axis].min(p[axis]));
                bbox[1] = std::array::from_fn(|axis| bbox[1][axis].max(p[axis]));
            }
        }
    }
    let number = |key: &str| settings[key].as_str().unwrap().parse::<f64>().unwrap();
    let rotation = number("wipe_tower_rotation_angle").rem_euclid(360.0);
    let quadrant = match rotation as i32 {
        0 => [1, 1],
        90 => [-1, 1],
        180 => [-1, -1],
        270 => [1, -1],
        _ => panic!("fixture rotation is not an axis-aligned quadrant"),
    };
    assert_eq!(rotation, f64::from(rotation as i32));
    assert_eq!(settings["wipe_tower_type"], "type2");
    let configured_brim = number("prime_tower_brim_width") as f32;
    let nozzle = settings["nozzle_diameter"][0]
        .as_str()
        .unwrap()
        .parse::<f32>()
        .unwrap();
    let initial = number("initial_layer_print_height");
    let first_layer = if initial > 1e-4 {
        initial
    } else {
        number("layer_height")
    } as f32;
    let brim = resolve_brim(
        configured_brim,
        nozzle,
        first_layer,
        (bbox[1][2] - bbox[0][2]) as f32,
    );
    // Print.cpp offsets by float(scale_(brim)); Clipper rounds the offset to its integer grid.
    let scaled_brim = (f64::from(brim) / 1e-6) as f32;
    let inset = f64::from(scaled_brim).round() * 1e-6;
    let anchor = [0, 1].map(|axis| {
        let lower = quadrant[axis] > 0;
        let exact = if lower {
            bounds[axis][0] + inset
        } else {
            bounds[axis][1] - inset
        };
        let nearest = exact as f32;
        let inward = if lower && f64::from(nearest) < exact {
            nearest.next_up()
        } else if !lower && f64::from(nearest) > exact {
            nearest.next_down()
        } else {
            nearest
        };
        f64::from(inward)
    });
    let corner = [0, 1].map(|axis| bounds[axis][usize::from(quadrant[axis] < 0)]);
    let bbox_distance_squared: f64 = [0, 1]
        .map(|axis| {
            (bbox[0][axis] - corner[axis])
                .max(corner[axis] - bbox[1][axis])
                .max(0.0)
                .powi(2)
        })
        .into_iter()
        .sum();
    for (axis, key) in ["wipe_tower_x", "wipe_tower_y"].into_iter().enumerate() {
        assert_eq!(settings[key].as_array().unwrap().len(), 1);
        settings[key][0] = anchor[axis].to_string().into();
    }
    let mut unchanged = settings.clone();
    unchanged["wipe_tower_x"] = before["wipe_tower_x"].clone();
    unchanged["wipe_tower_y"] = before["wipe_tower_y"].clone();
    assert_eq!(unchanged, before);
    fs::write(
        directory.join("input-binding.json"),
        serde_json::to_vec_pretty(&json!({
            "original_sha256": super::sha256(original),
            "scenario": scenario.label(), "bounds": bounds, "object_bbox": bbox,
            "rotation": rotation, "quadrant": quadrant, "eligible_corner_count": 1,
            "corner": corner, "bbox_distance_squared": bbox_distance_squared,
            "configured_brim": configured_brim, "brim": brim, "brim_bits": brim.to_bits(),
            "scaled_brim": scaled_brim, "inset": inset, "anchor": anchor,
            "before_settings": before, "after_settings": settings,
            "changed_settings": ["wipe_tower_x", "wipe_tower_y"],
        }))
        .unwrap(),
    )
    .unwrap();
    entries[settings_index].2 = serde_json::to_vec(&settings).unwrap();
    let mut rewritten = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for (name, compression, bytes) in &entries {
        rewritten
            .start_file(
                name,
                zip::write::SimpleFileOptions::default().compression_method(*compression),
            )
            .unwrap();
        rewritten.write_all(bytes).unwrap();
    }
    let bytes = rewritten.finish().unwrap().into_inner();
    let mut check = zip::ZipArchive::new(Cursor::new(&bytes)).unwrap();
    for (index, (name, _, expected)) in entries.iter().enumerate() {
        let mut entry = check.by_index(index).unwrap();
        assert_eq!(entry.name(), name);
        let mut actual = Vec::new();
        entry.read_to_end(&mut actual).unwrap();
        assert_eq!(&actual, expected);
    }
    bytes
}

fn resolve_brim(mut brim: f32, nozzle: f32, height: f32, object_height: f32) -> f32 {
    // WipeTower::get_auto_brim_by_height / estimate_brim_real_width, Type2.
    if brim < 0.0 {
        brim = if object_height < 100.0 {
            object_height / 100.0 * 8.0
        } else {
            8.0
        };
    }
    if brim <= 0.0 {
        return brim;
    }
    let spacing = nozzle * 1.25 - height * (1.0 - std::f64::consts::FRAC_PI_4) as f32;
    if f64::from(spacing) <= 1e-4 {
        return brim;
    }
    let loops = ((brim + spacing / 2.0) / spacing) as i32;
    loops as f32 * spacing
}
