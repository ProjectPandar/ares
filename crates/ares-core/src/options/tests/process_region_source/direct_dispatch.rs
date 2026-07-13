use serde_json::{Map, Value, json};

use super::super::super::ProcessRegionSourceOptions;
use super::{InventoryRow, expected_default, inventory, region_rows};

#[test]
fn every_region_field_directly_dispatches_a_valid_nondefault_value() {
    let rows = inventory();
    let region = region_rows(&rows);
    assert_eq!(region.len(), 149);
    for row in region {
        let alternate = alternate(row);
        assert_ne!(alternate, expected_default(row), "{}", row.key);
        let input = Map::from_iter([(row.key.clone(), alternate.clone())]);
        let parsed: ProcessRegionSourceOptions =
            serde_json::from_value(Value::Object(input)).unwrap_or_else(|error| {
                panic!("{} failed direct dispatch: {error}", row.key)
            });
        assert_eq!(
            serde_json::to_value(parsed).unwrap()[&row.key],
            alternate,
            "{}",
            row.key
        );
    }
}
#[test]
fn float_or_percent_alternates_use_the_opposite_category() {
    let rows = inventory();
    let fop = region_rows(&rows)
        .into_iter()
        .filter(|row| row.option_type == "coFloatOrPercent")
        .collect::<Vec<_>>();
    assert_eq!(fop.len(), 24);
    assert_eq!(
        fop.iter()
            .filter(|row| row.default_serialized.ends_with('%'))
            .count(),
        12
    );
    for row in fop {
        let alternate = alternate(row);
        let alternate = alternate.as_str().unwrap();
        assert_ne!(
            row.default_serialized.ends_with('%'),
            alternate.ends_with('%'),
            "{}",
            row.key
        );
    }
}

fn alternate(row: &InventoryRow) -> Value {
    match row.option_type.as_str() {
        "coBool" => Value::String(
            if row.default_serialized == "1" { "0" } else { "1" }.to_owned(),
        ),
        "coFloat" => Value::String("7.125".to_owned()),
        "coInt" => Value::String("7".to_owned()),
        "coPercent" => Value::String("37%".to_owned()),
        "coFloatOrPercent" if row.default_serialized.ends_with('%') => {
            Value::String("7.125".to_owned())
        }
        "coFloatOrPercent" => Value::String("37%".to_owned()),
        "coString" => Value::String("raw task10 value".to_owned()),
        "coInts" => json!(["7", "8", "9"]),
        "coStrings" => json!(["Direct Drive Standard", "Bowden High Flow", "custom"]),
        "coEnum" => Value::String(enum_alternate(&row.key).to_owned()),
        other => panic!("unhandled region type {other} for {}", row.key),
    }
}

fn enum_alternate(key: &str) -> &'static str {
    match key {
        "ensure_vertical_shell_thickness" => "ensure_moderate",
        "top_surface_pattern"
        | "bottom_surface_pattern"
        | "internal_solid_infill_pattern"
        | "sparse_infill_pattern"
        | "ironing_pattern" => "gyroid",
        "fuzzy_skin" => "allwalls",
        "fuzzy_skin_noise_type" => "voronoi",
        "fuzzy_skin_mode" => "combined",
        "ironing_type" => "solid",
        "counterbore_hole_bridging" => "sacrificiallayer",
        "wall_sequence" => "inner-outer-inner wall",
        "wall_direction" => "cw",
        "seam_slope_type" => "all",
        _ => panic!("unhandled region enum {key}"),
    }
}
