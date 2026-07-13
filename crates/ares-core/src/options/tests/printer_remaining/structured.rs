use super::super::super::{ExtruderVariantLists, ThumbnailDefinitions};

#[test]
fn printer_options_variant_lists_preserve_raw_groups_and_expose_tokens() {
    let raw = r#"[" Direct Drive Standard,,Direct Drive High Flow ","custom,token"]"#;
    let variants: ExtruderVariantLists = serde_json::from_str(raw).unwrap();
    assert_eq!(serde_json::to_string(&variants).unwrap(), raw);
    assert_eq!(variants.0.len(), 2);
    assert_eq!(variants.0[0], " Direct Drive Standard,,Direct Drive High Flow ");
    assert_eq!(variants.0[1], "custom,token");
}

#[test]
fn printer_options_thumbnails_preserve_raw_text() {
    for raw in [
        "",
        "48x48/PNG,300x300/PNG",
        "48x48/png",
        "48x48",
        "0x48/PNG",
        "1000x48/PNG",
        "48x48/UNKNOWN",
        "wrong",
    ] {
        let json = serde_json::to_string(raw).unwrap();
        let thumbnails: ThumbnailDefinitions = serde_json::from_str(&json).unwrap();
        assert_eq!(thumbnails.as_str(), raw);
        assert_eq!(serde_json::to_string(&thumbnails).unwrap(), json);
    }
}
