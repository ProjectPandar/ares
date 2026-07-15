use crate::options::{
    model_config_deserialize::{decode_wire_string, decode_wire_strings, validate_wire_value},
    registry::OptionValueKind,
};

#[test]
fn every_option_value_kind_has_a_metadata_wire_branch() {
    let cases = [
        (OptionValueKind::Float, "1.25"),
        (OptionValueKind::FloatOrPercent, "1.25%"),
        (OptionValueKind::Percent, "25%"),
        (OptionValueKind::Percents, "10%, 20%"),
        (OptionValueKind::PercentsNullable, "10%, nil"),
        (OptionValueKind::Int, "-2"),
        (OptionValueKind::Bool, " 1,0"),
        (OptionValueKind::Bools, "1, 0"),
        (OptionValueKind::BoolsNullable, "1, nil"),
        (OptionValueKind::Enum, "Exact Token"),
        (OptionValueKind::Enums, "A, B"),
        (OptionValueKind::EnumsNullable, "A, nil"),
        (OptionValueKind::Floats, ""),
        (OptionValueKind::FloatsNullable, "1, nil"),
        (OptionValueKind::IntsNullable, "1, nil"),
        (OptionValueKind::Ints, ""),
        (OptionValueKind::Strings, "\"line\\n\";plain"),
        (OptionValueKind::String, "line\\nnext"),
        (OptionValueKind::Point, "1x2"),
        (OptionValueKind::Points, "1x2, 3x4"),
        (OptionValueKind::PointsGroups, "1x2,3x4#5x6"),
    ];
    assert_eq!(cases.len(), 21);
    for (kind, value) in cases {
        validate_wire_value(kind, value).unwrap_or_else(|error| {
            panic!("{kind:?} rejected {value:?}: {error:?}")
        });
    }
}

#[test]
fn malformed_concrete_wire_values_are_rejected() {
    let cases = [
        (OptionValueKind::Float, "x"),
        (OptionValueKind::FloatOrPercent, "%"),
        (OptionValueKind::Percent, "x%"),
        (OptionValueKind::Percents, "1%,x"),
        (OptionValueKind::PercentsNullable, "1%,x"),
        (OptionValueKind::Int, "1.5"),
        (OptionValueKind::Bool, "x"),
        (OptionValueKind::Bools, "1,x"),
        (OptionValueKind::BoolsNullable, "1,x"),
        (OptionValueKind::Floats, "1,x"),
        (OptionValueKind::FloatsNullable, "1,x"),
        (OptionValueKind::IntsNullable, "1,x"),
        (OptionValueKind::Ints, "1,x"),
        (OptionValueKind::Strings, "\"unterminated"),
        (OptionValueKind::String, "trailing\\"),
        (OptionValueKind::Point, "1"),
        (OptionValueKind::Points, "1x2,invalid"),
        (OptionValueKind::PointsGroups, "1x2#invalid"),
    ];
    for (kind, value) in cases {
        assert!(validate_wire_value(kind, value).is_err(), "{kind:?}={value:?}");
    }
}

#[test]
fn c_style_string_vector_only_unescapes_quoted_items() {
    assert_eq!(decode_wire_string("line\\nnext").unwrap(), "line\nnext");
    assert_eq!(
        decode_wire_strings("plain\\n;\"line\\nnext\"").unwrap(),
        ["plain\\n", "line\nnext"],
    );
    assert!(validate_wire_value(OptionValueKind::Strings, "\"trailing\\").is_err());
}
