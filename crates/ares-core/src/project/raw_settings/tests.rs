use super::ProjectSettingsRaw;

fn tokens(json: &str) -> Vec<(String, Option<String>)> {
    let raw: ProjectSettingsRaw = serde_json::from_str(json).unwrap();
    raw.iter()
        .map(|entry| {
            let (key, token) = entry.unwrap();
            (key.to_owned(), token)
        })
        .collect()
}

#[test]
fn scalars_render_their_export_tokens() {
    assert_eq!(
        tokens(r#"{"text":"0.42","flag":true,"off":false,"int":-5,"float":3.0,"precise":0.45}"#),
        vec![
            ("flag".to_owned(), Some("1".to_owned())),
            ("float".to_owned(), Some("3.0".to_owned())),
            ("int".to_owned(), Some("-5".to_owned())),
            ("off".to_owned(), Some("0".to_owned())),
            ("precise".to_owned(), Some("0.45".to_owned())),
            ("text".to_owned(), Some("0.42".to_owned())),
        ]
    );
}

#[test]
fn nil_null_and_objects_stay_known_without_tokens() {
    let raw: ProjectSettingsRaw =
        serde_json::from_str(r#"{"nil":"nil","none":null,"object":{"nested":1}}"#).unwrap();
    assert!(raw.contains_key("nil"));
    assert!(raw.contains_key("none"));
    assert!(raw.contains_key("object"));
    assert!(raw.iter().all(|entry| entry.unwrap().1.is_none()));
}

#[test]
fn lists_join_renderable_items_and_drop_everything_else() {
    assert_eq!(
        tokens(
            r#"{"nozzle":["0.4","0.4"],"mixed":[1,2.5],"skipped":[true,null,[1],{},"nil"],"empty":[]}"#
        ),
        vec![
            ("empty".to_owned(), None),
            ("mixed".to_owned(), Some("1;2.5".to_owned())),
            ("nozzle".to_owned(), Some("0.4;0.4".to_owned())),
            ("skipped".to_owned(), None),
        ]
    );
}

#[test]
fn duplicate_keys_keep_the_last_value() {
    assert_eq!(
        tokens(r#"{"key":"first","key":"second"}"#),
        vec![("key".to_owned(), Some("second".to_owned()))]
    );
}
