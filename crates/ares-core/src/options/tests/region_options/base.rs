use serde_json::{Map, Value, json};

use super::{ProcessRegionSourceOptions, RegionOptions, inventory, region_rows, types};

#[test]
fn region_options_project_all_149_non_default_process_fields() {
    let rows = inventory();
    let values = region_rows(&rows)
        .into_iter()
        .map(|row| {
            (
                row.key.clone(),
                non_default_value(&row.key, &row.option_type, &row.default_serialized),
            )
        })
        .collect::<Map<_, _>>();
    let source: ProcessRegionSourceOptions =
        serde_json::from_value(Value::Object(values)).unwrap();
    let actual = serde_json::to_value(&source).unwrap();
    let defaults = serde_json::to_value(ProcessRegionSourceOptions::default()).unwrap();
    for row in region_rows(&rows) {
        assert_ne!(actual[&row.key], defaults[&row.key], "{}", row.key);
    }
    let effective = RegionOptions::from_base(&source);

    types::assert_concrete_types_and_identity(&effective, &source);
    assert_eq!(effective.filament_ironing_flow, source.ironing_flow);
    assert_eq!(effective.filament_ironing_spacing, source.ironing_spacing);
    assert_eq!(effective.filament_ironing_inset, source.ironing_inset);
    assert_eq!(effective.filament_ironing_speed, source.ironing_speed);
}

pub(super) fn non_default_value(key: &str, option_type: &str, default: &str) -> Value {
    match option_type {
        "coBool" if default == "0" => json!("1"),
        "coBool" => json!("0"),
        "coFloat" => json!("42.25"),
        "coFloatOrPercent" => json!("37%"),
        "coInt" => json!("17"),
        "coInts" => json!(["7", "8"]),
        "coPercent" => json!("37%"),
        "coString" => json!("non-default"),
        "coStrings" => json!(["first", "second"]),
        "coEnum" => json!(non_default_enum(key)),
        other => panic!("unexpected region option type {other}"),
    }
}

fn non_default_enum(key: &str) -> &'static str {
    match key {
        "bottom_surface_pattern" => "archimedeanchords",
        "counterbore_hole_bridging" => "partiallybridge",
        "ensure_vertical_shell_thickness" => "ensure_critical_only",
        "fuzzy_skin" => "allwalls",
        "fuzzy_skin_mode" => "extrusion",
        "fuzzy_skin_noise_type" => "billow",
        "internal_solid_infill_pattern" => "grid",
        "ironing_pattern" => "concentric",
        "ironing_type" => "top",
        "seam_slope_type" => "external",
        "sparse_infill_pattern" => "gyroid",
        "top_surface_pattern" => "hilbertcurve",
        "wall_direction" => "cw",
        "wall_sequence" => "outer wall/inner wall",
        other => panic!("unexpected region enum {other}"),
    }
}
