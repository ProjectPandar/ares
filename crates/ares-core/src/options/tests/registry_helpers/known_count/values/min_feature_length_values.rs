use serde_json::{Map, Value, json};

pub(super) fn insert_known_min_feature_length_values(values: &mut Map<String, Value>) {
    for (key, value) in [
        ("max_volumetric_extrusion_rate_slope", json!(0)),
        (
            "max_volumetric_extrusion_rate_slope_segment_length",
            json!(3.0),
        ),
        ("min_bead_width", json!(85)),
        ("min_feature_size", json!(25)),
        ("min_length_factor", json!(0.5)),
    ] {
        values.insert(key.to_owned(), value);
    }
}
