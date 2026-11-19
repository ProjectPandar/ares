#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum BedTemperatureFormula {
    #[serde(rename = "by_first_filament")]
    FirstFilament,
    #[default]
    #[serde(rename = "by_highest_temp")]
    HighestTemp,
}

/// Bed-type → first-layer bed temperature option key, mirroring upstream
/// `get_bed_temp_1st_layer_key` (`PrintConfig.hpp:489`).
pub(crate) fn first_layer_bed_temperature_key_for(bed_type: &str) -> Option<&'static str> {
    match bed_type {
        "Cool Plate" => Some("cool_plate_temp_initial_layer"),
        "Textured Cool Plate" => Some("textured_cool_plate_temp_initial_layer"),
        "Engineering Plate" => Some("eng_plate_temp_initial_layer"),
        "High Temp Plate" => Some("hot_plate_temp_initial_layer"),
        "Textured PEI Plate" => Some("textured_plate_temp_initial_layer"),
        "Supertack Plate" | "SuperTack Plate" => Some("supertack_plate_temp_initial_layer"),
        _ => None,
    }
}
