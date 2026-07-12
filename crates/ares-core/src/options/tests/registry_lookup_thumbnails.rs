#[test]
fn exposes_thumbnails_option_definition_lookup() {
    let thumbnails = crate::option_definition("thumbnails").unwrap();
    assert_eq!(thumbnails.kind, crate::OptionValueKind::String);
    assert_eq!(thumbnails.default_value, "48x48/PNG,300x300/PNG");
    assert!(thumbnails.source.contains("PrintConfig.hpp:1616"));
    assert!(thumbnails.source.contains("PrintConfig.cpp:6956-6961"));

    let format = crate::option_definition("thumbnails_format").unwrap();
    assert_eq!(format.kind, crate::OptionValueKind::Enum);
    assert_eq!(format.default_value, "PNG");
    assert!(format.source.contains("PrintConfig.hpp:397-399"));
    assert!(format.source.contains("PrintConfig.cpp:542-549"));
    assert!(format.source.contains("PrintConfig.cpp:6963-6978"));
}
