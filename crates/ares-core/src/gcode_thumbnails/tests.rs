use super::*;

#[test]
fn parses_all_upstream_thumbnail_formats_in_order() {
    let definitions =
        parse_thumbnail_definitions("1x2/png,3x4/jpg,5x6/qoi,7x8/btt_tft,9x10/colpic", None)
            .unwrap();

    assert_eq!(
        definitions,
        vec![
            GCodeThumbnailDefinition {
                format: GCodeThumbnailFormat::Png,
                width: 1.0,
                height: 2.0,
            },
            GCodeThumbnailDefinition {
                format: GCodeThumbnailFormat::Jpg,
                width: 3.0,
                height: 4.0,
            },
            GCodeThumbnailDefinition {
                format: GCodeThumbnailFormat::Qoi,
                width: 5.0,
                height: 6.0,
            },
            GCodeThumbnailDefinition {
                format: GCodeThumbnailFormat::BttTft,
                width: 7.0,
                height: 8.0,
            },
            GCodeThumbnailDefinition {
                format: GCodeThumbnailFormat::ColPic,
                width: 9.0,
                height: 10.0,
            },
        ]
    );
    assert_eq!(GCodeThumbnailFormat::Png.as_str(), "PNG");
    assert_eq!(GCodeThumbnailFormat::Jpg.as_str(), "JPG");
    assert_eq!(GCodeThumbnailFormat::Qoi.as_str(), "QOI");
    assert_eq!(GCodeThumbnailFormat::BttTft.as_str(), "BTT_TFT");
    assert_eq!(GCodeThumbnailFormat::ColPic.as_str(), "COLPIC");
}

#[test]
fn applies_default_extension_rules() {
    assert_eq!(
        parse_thumbnail_definitions("16x16", None).unwrap()[0].format,
        GCodeThumbnailFormat::Png
    );
    assert_eq!(
        parse_thumbnail_definitions("16x16", Some("qoi")).unwrap()[0].format,
        GCodeThumbnailFormat::Qoi
    );
    assert_eq!(
        parse_thumbnail_definitions("16x16", Some("unsupported")).unwrap()[0].format,
        GCodeThumbnailFormat::Png
    );
    assert_eq!(
        parse_thumbnail_definitions("16x16/", Some("jpg")).unwrap()[0].format,
        GCodeThumbnailFormat::Jpg
    );
}

#[test]
fn returns_empty_definitions_for_empty_input() {
    assert_eq!(parse_thumbnail_definitions("", None).unwrap(), Vec::new());
}

#[test]
fn distinguishes_thumbnail_parse_errors() {
    assert_eq!(
        parse_thumbnail_definitions("16", None),
        Err(ThumbnailParseError::InvalidValue)
    );
    assert_eq!(
        parse_thumbnail_definitions("0x16", None),
        Err(ThumbnailParseError::OutOfRange)
    );
    assert_eq!(
        parse_thumbnail_definitions("16x16/bmp", None),
        Err(ThumbnailParseError::InvalidExtension)
    );
    assert_eq!(
        parse_thumbnail_definitions("16x16/jpg ", None),
        Err(ThumbnailParseError::InvalidExtension)
    );
}

#[test]
fn exposes_upstream_error_message_fragments() {
    assert!(
        thumbnail_error_string(ThumbnailParseError::InvalidValue)
            .contains("Invalid input format")
    );
    assert!(thumbnail_error_string(ThumbnailParseError::OutOfRange).contains("out of range"));
    assert!(
        thumbnail_error_string(ThumbnailParseError::InvalidExtension).contains("extension")
    );
}
