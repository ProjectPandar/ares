use serde_json::{Map, Value, json};

pub(super) fn insert_known_scarf_seam_values(values: &mut Map<String, Value>) {
    for (key, value) in [
        ("scarf_angle_threshold", json!(155)),
        ("scarf_joint_flow_ratio", json!(1)),
        ("scarf_joint_speed", json!("100%")),
        ("scarf_overhang_threshold", json!(40)),
        ("seam_slope_conditional", json!(false)),
        ("seam_slope_entire_loop", json!(false)),
        ("seam_slope_inner_walls", json!(false)),
        ("seam_slope_min_length", json!(20)),
        ("seam_slope_start_height", json!(0)),
        ("seam_slope_steps", json!(10)),
        ("seam_slope_type", json!("none")),
    ] {
        values.insert(key.to_owned(), value);
    }
}
