use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq)]
pub(crate) struct PlateJson {
    pub bbox_all: [f64; 4],
    pub bbox_objects: Vec<PlateObjectBounds>,
    pub bed_type: String,
    pub filament_colors: Vec<String>,
    pub filament_ids: Vec<i32>,
    pub first_extruder: i32,
    pub first_layer_time: f64,
    pub is_seq_print: bool,
    pub nozzle_diameter: f64,
    pub version: u32,
}

#[derive(Debug, Deserialize, PartialEq)]
pub(crate) struct PlateObjectBounds {
    pub area: f64,
    pub bbox: [f64; 4],
    pub id: i32,
    pub layer_height: f64,
    pub name: String,
}
