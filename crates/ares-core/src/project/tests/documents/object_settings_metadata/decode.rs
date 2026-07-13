use crate::{
    FloatOrPercent, OrcaBool, OrcaFloat, OrcaInt, Percent, ProcessBrimType,
    ProcessExtraBridgeLayer, ProcessGapFillTarget, ProcessInfillPattern,
    ProcessInternalBridgeFilter, ProcessPerimeterGenerator, ProcessSeamPosition,
    ProcessSlicingMode, ProcessSupportBasePattern, ProcessSupportInterfacePattern,
    ProcessSupportStyle, ProcessSupportType, SliceError,
};

use super::{
    cases::object_metadata_cases, expected::DECLARATION_ORDER, object_overrides, parse_single,
};

macro_rules! define_canonical_decode_test {
    ($($field:ident => ($lexical:literal, $expected:expr)),* $(,)?) => {
        fn present_fields(overrides: &crate::options::ObjectOptionOverrides) -> Vec<&'static str> {
            let mut present = Vec::new();
            $(
                if overrides.$field.is_some() {
                    present.push(stringify!($field));
                }
            )*
            present
        }

        #[test]
        fn object_settings_metadata_decodes_each_canonical_key_through_isolated_real_xml() {
            let table_order = [$(stringify!($field)),*];
            assert_eq!(table_order, DECLARATION_ORDER);

            $(
                {
                    let key = stringify!($field);
                    let object = parse_single(key, $lexical).unwrap();
                    let overrides = object_overrides(&object);
                    let expected = $expected;
                    assert_eq!(overrides.$field.as_ref(), Some(&expected), "{key}");
                    assert_eq!(present_fields(overrides), [key], "{key}");
                }
            )*
        }
    };
}

object_metadata_cases!(define_canonical_decode_test);

#[test]
fn object_settings_metadata_malformed_xml_names_context_key_and_exact_codec_fragment() {
    for (key, value, reason) in [
        ("enable_support", "not-bool", "expected Orca boolean 0 or 1"),
        ("raft_layers", "not-int", "invalid digit found in string"),
        ("layer_height", "not-float", "invalid float literal"),
        (
            "support_ironing_flow",
            "not-percent",
            "invalid float literal",
        ),
        (
            "line_width",
            "not-float-or-percent",
            "invalid float literal",
        ),
        ("support_type", "not-an-enum-token", "unknown variant"),
    ] {
        let error = parse_single(key, value).unwrap_err();
        let SliceError::InvalidInput(message) = error else {
            panic!("unexpected error for {key}: {error}");
        };
        assert!(
            message.contains("invalid project model settings XML:"),
            "{message}"
        );
        assert!(
            message.contains(&format!("invalid Orca object option {key}:")),
            "{message}"
        );
        assert!(message.contains(reason), "{message}");
    }
}
