use serde_json::json;

use super::legacy_feature_filament::check_bookkeeping;

#[test]
fn saved_bookkeeping_clears_an_obsolete_name_without_losing_slots() {
    check_bookkeeping(
        "bookkeeping-obsolete-",
        r#";"silent_mode;layer_height";"#,
        json!(["", ";layer_height", ""]),
    );
}

#[test]
fn saved_bookkeeping_clears_an_unregistered_name_without_losing_slots() {
    check_bookkeeping(
        "bookkeeping-unregistered-",
        r#";"probe_unregistered_config_key;layer_height";"#,
        json!(["", ";layer_height", ""]),
    );
}

#[test]
fn saved_bookkeeping_preserves_already_cleared_slots_and_registered_names() {
    check_bookkeeping(
        "bookkeeping-cleared-",
        r#";";layer_height";"#,
        json!(["", ";layer_height", ""]),
    );
}
