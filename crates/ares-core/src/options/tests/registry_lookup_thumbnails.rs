#[test]
fn exposes_thumbnails_option_definition_lookup() {
    let thumbnails = crate::option_definition("thumbnails").unwrap();
    assert_eq!(thumbnails.kind, crate::OptionValueKind::String);
    assert_eq!(thumbnails.default_value, "48x48/PNG,300x300/PNG");

    let format = crate::option_definition("thumbnails_format").unwrap();
    assert_eq!(format.kind, crate::OptionValueKind::Enum);
    assert_eq!(format.default_value, "PNG");
}

#[test]
fn thumbnail_default_is_not_a_loaded_composite() {
    let absent: crate::ProjectSettings = serde_json::from_str("{}").unwrap();
    assert_eq!(absent.printer.remaining.thumbnails.as_str(), "48x48/PNG,300x300/PNG");
    let present: crate::ProjectSettings = serde_json::from_str(
        r#"{"thumbnails":"48x48/PNG,300x300/PNG"}"#,
    ).unwrap();
    assert_eq!(present.printer.remaining.thumbnails.as_str(), "48x48/PNG, 300x300/PNG");
}
