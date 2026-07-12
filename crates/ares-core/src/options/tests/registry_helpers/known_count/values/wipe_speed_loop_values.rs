use serde_json::{Map, Value, json};

pub(super) fn insert_known_wipe_speed_loop_values(values: &mut Map<String, Value>) {
    for (key, value) in [
        ("role_based_wipe_speed", json!(true)),
        ("wipe_before_external_loop", json!(false)),
        ("wipe_on_loops", json!(false)),
        ("wipe_speed", json!("80%")),
    ] {
        values.insert(key.to_owned(), value);
    }
}
