use serde_json::{Map, Value, json};

pub(super) fn insert_known_restart_speed_seam_values(values: &mut Map<String, Value>) {
    for (key, value) in [
        ("bbl_calib_mark_logo", json!(true)),
        ("deretraction_speed", json!([0])),
        ("disable_m73", json!(false)),
        ("retract_restart_extra", json!([0])),
        ("retract_restart_extra_toolchange", json!([0])),
        ("retraction_speed", json!([30])),
        ("seam_gap", json!("10%")),
        ("seam_position", json!("aligned")),
        ("staggered_inner_seams", json!(false)),
        ("use_firmware_retraction", json!(false)),
    ] {
        values.insert(key.to_owned(), value);
    }
}
