use crate::SliceOptions;
use serde_json::{Map, Value, json};

mod values;

#[test]
fn known_definition_count_preserves_unknown_and_excludes_obsolete_options() {
    let mut values = Map::new();
    values::insert_known_values(&mut values);
    values.insert("future_orca_key".to_owned(), json!(true));

    let options: SliceOptions = serde_json::from_value(Value::Object(values)).unwrap();

    assert_eq!(options.known_definition_count(), 677);
    assert_eq!(options.values().len(), 678);
    assert_eq!(options.values()["future_orca_key"], json!(true));
    assert_eq!(options.values()["silent_mode"], json!(false));
    assert!(!options.values().contains_key("tree_support_with_infill"));
}
