use serde::Serialize;

use crate::SlicedArchive;

pub(in crate::project::export) fn plate_json(
    archive: &SlicedArchive,
    first_print_object_id: u32,
) -> Vec<u8> {
    #[derive(Serialize)]
    struct Object<'a> {
        area: f64,
        bbox: [f64; 4],
        id: u32,
        layer_height: f64,
        name: &'a str,
    }
    #[derive(Serialize)]
    struct Plate<'a> {
        bbox_all: [f64; 4],
        bbox_objects: Vec<Object<'a>>,
        bed_type: &'static str,
        filament_colors: [String; 1],
        filament_ids: [u32; 1],
        first_extruder: u32,
        first_layer_time: f32,
        is_seq_print: bool,
        nozzle_diameter: f64,
        version: u32,
    }
    let bbox_all = archive
        .objects
        .iter()
        .map(|object| object.bounds)
        .reduce(|a, b| {
            [
                a[0].min(b[0]),
                a[1].min(b[1]),
                a[2].max(b[2]),
                a[3].max(b[3]),
            ]
        })
        .expect("sliced plate has objects");
    let bbox_objects = archive
        .objects
        .iter()
        .enumerate()
        .map(|(index, object)| Object {
            area: f64::from(object.area),
            bbox: object.bounds,
            id: first_print_object_id + index as u32,
            layer_height: f64::from(object.layer_height),
            name: &object.name,
        })
        .collect();
    serde_json::to_vec(&Plate {
        bbox_all,
        bbox_objects,
        bed_type: "cool_plate",
        filament_colors: [archive.filament_color.clone()],
        filament_ids: [0],
        first_extruder: 0,
        first_layer_time: 0.0,
        is_seq_print: false,
        nozzle_diameter: f64::from(archive.nozzle_diameter),
        version: 2,
    })
    .expect("plate metadata is serializable")
}
