use super::super::*;
use serde_json::json;

#[test]
fn normalizes_legacy_thumbnail_entries_with_default_format() {
    let options: SliceOptions = serde_json::from_value(json!({
        "thumbnails": "48x48, 300x300/jpg",
        "thumbnails_format": "QOI",
        "future_orca_key": true
    }))
    .unwrap();

    assert_eq!(
        options.values()["thumbnails"],
        json!("48x48/QOI, 300x300/JPG")
    );
    assert_eq!(options.values()["thumbnails_format"], json!("QOI"));
    assert_eq!(options.values()["future_orca_key"], json!(true));
}

#[test]
fn defaults_missing_or_unsupported_thumbnail_format_to_png() {
    for input in [
        json!({"thumbnails": "16x16"}),
        json!({"thumbnails": "16x16", "thumbnails_format": "unsupported"}),
        json!({"thumbnails": "16x16", "thumbnails_format": 3}),
    ] {
        let options: SliceOptions = serde_json::from_value(input).unwrap();
        assert_eq!(options.values()["thumbnails"], json!("16x16/PNG"));
    }
}

#[test]
fn normalizes_thumbnail_size_alias_through_composite_pass() {
    let options: SliceOptions = serde_json::from_value(json!({
        "thumbnail_size": "256x256"
    }))
    .unwrap();

    assert!(!options.values().contains_key("thumbnail_size"));
    assert_eq!(options.values()["thumbnails"], json!("256x256/PNG"));
}

#[test]
fn normalizes_all_supported_thumbnail_extensions() {
    let options: SliceOptions = serde_json::from_value(json!({
        "thumbnails": "1x2/png, 3x4/jpg, 5x6/qoi, 7x8/btt_tft, 9x10/colpic"
    }))
    .unwrap();

    assert_eq!(
        options.values()["thumbnails"],
        json!("1x2/PNG, 3x4/JPG, 5x6/QOI, 7x8/BTT_TFT, 9x10/COLPIC")
    );
}

#[test]
fn preserves_empty_and_non_string_thumbnail_values() {
    let empty: SliceOptions = serde_json::from_value(json!({"thumbnails": ""})).unwrap();
    assert_eq!(empty.values()["thumbnails"], json!(""));

    let non_string: SliceOptions = serde_json::from_value(json!({"thumbnails": [16, 16]})).unwrap();
    assert_eq!(non_string.values()["thumbnails"], json!([16, 16]));
}

#[test]
fn rejects_invalid_thumbnail_entries() {
    for thumbnails in [
        "16",
        "0x16",
        "16x1000",
        "16x16/bmp",
        "16x16/ jpg",
        "16x16/jpg ",
        "x16",
        "16x",
    ] {
        let error = serde_json::from_value::<SliceOptions>(json!({
            "thumbnails": thumbnails
        }))
        .unwrap_err();

        assert!(
            error
                .to_string()
                .contains("Invalid value provided for parameter thumbnails")
        );
    }
}
