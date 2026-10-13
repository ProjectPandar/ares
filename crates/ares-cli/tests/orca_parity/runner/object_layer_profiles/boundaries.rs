use super::{ABSENT, compare_case, compare_profile};

macro_rules! input {
    ($label:literal) => {
        include_bytes!(concat!(
            "../../../../../../tests/parity/object-layer-profiles/",
            $label,
            ".3mf"
        ))
        .as_slice()
    };
}

#[tokio::test]
async fn object_profile_nonfatal_import_records_preserve_complete_native_output() {
    for (label, input, changes) in [
        ("empty", input!("empty"), false),
        ("four-values", input!("four-values"), false),
        ("odd-count", input!("odd-count"), false),
        ("duplicate", input!("duplicate"), true),
        ("unknown-ordinal", input!("unknown-ordinal"), false),
        ("malformed-record", input!("malformed-record"), false),
        ("malformed-number", input!("malformed-number"), true),
        ("numeric-prefix", input!("numeric-prefix"), true),
        ("hex-number", input!("hex-number"), true),
        ("member-order", input!("member-order"), true),
    ] {
        compare_case(label, input, ABSENT, None, changes).await;
    }
}

#[tokio::test]
async fn object_profile_invalid_top_and_first_height_regenerate_complete_native_output() {
    for (label, input) in [
        ("wrong-top", input!("wrong-top")),
        ("wrong-first", input!("wrong-first")),
    ] {
        compare_case(label, input, ABSENT, None, false).await;
    }
}

#[tokio::test]
async fn object_profile_repeated_z_knots_preserve_complete_native_output() {
    compare_profile("repeat-z", input!("repeat-z")).await;
}

#[tokio::test]
async fn object_profiles_keep_original_ordinals_across_distinct_objects() {
    compare_case(
        "two-objects",
        input!("two-objects"),
        input!("two-objects-absent"),
        None,
        true,
    )
    .await;
}

#[tokio::test]
async fn object_profiles_keep_original_ordinals_after_plate_selection() {
    compare_case(
        "selected-plate",
        input!("selected-plate"),
        input!("selected-plate-absent"),
        Some(1),
        true,
    )
    .await;
}
