use serde_json::{Map, Value};

use super::{
    ObjectOptionOverrides, ObjectOptions, ProcessObjectSourceOptions, inventory, object_rows,
    types,
};

#[test]
fn object_options_inventory_base_projection_matches_all_126_fields() {
    let rows = inventory();
    let object = object_rows(&rows);
    let values = object
        .iter()
        .map(|row| {
            let value = non_default_value(
                row.key.as_str(),
                row.option_type.as_str(),
                row.default_serialized.as_str(),
            );
            assert_ne!(value.as_str().unwrap(), row.default_serialized, "{}", row.key);
            (row.key.clone(), value)
        })
        .collect::<Map<_, _>>();
    let base: ProcessObjectSourceOptions = serde_json::from_value(Value::Object(values)).unwrap();
    let effective = ObjectOptions::from_base(&base);
    let sparse = ObjectOptionOverrides::default();

    types::assert_base_and_sparse(&effective, &base, &sparse);
}

fn non_default_value(key: &str, option_type: &str, default: &str) -> Value {
    let value = match option_type {
        "coBool" if default == "0" => "1",
        "coBool" => "0",
        "coFloat" => "42.25",
        "coFloatOrPercent" => "37%",
        "coInt" => "17",
        "coPercent" => "37%",
        "coEnum" => non_default_enum(key),
        other => panic!("unexpected object option type {other}"),
    };
    Value::String(value.to_owned())
}

fn non_default_enum(key: &str) -> &'static str {
    match key {
        "brim_type" => "no_brim",
        "dont_filter_internal_bridges" => "nofilter",
        "enable_extra_bridge_layer" => "apply_to_all",
        "gap_fill_target" => "everywhere",
        "seam_position" => "random",
        "slicing_mode" => "close_holes",
        "support_base_pattern" => "hollow",
        "support_interface_pattern" => "grid",
        "support_ironing_pattern" => "gyroid",
        "support_style" => "organic",
        "support_type" => "tree(manual)",
        "wall_generator" => "classic",
        other => panic!("unexpected object enum {other}"),
    }
}
