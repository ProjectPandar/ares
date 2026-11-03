//! Beta fuzzy-region masks are restricted to the layer below before perimeter fuzzing.

use super::first_cap_layer_object;

#[tokio::test]
async fn unsupported_cap_keeps_native_smooth_walls() {
    let project = include_bytes!(
        "../../../../tests/parity/fuzzy_skin_supported_area/unsupported_cap_ripple.3mf"
    );
    let (reference, _) = crate::runner::native_stream_case::qualify(
        (
            "fuzzy_skin_supported_area",
            &[
                "345efec64c7b95a05ce5cf96df0dde66a158835c",
                "Feature/FuzzySkin/FuzzySkin.cpp:480-488 group_region_by_fuzzify",
                "Feature/FuzzySkin/FuzzySkin.cpp:587-593 restrict_to_supported",
                "Feature/FuzzySkin/FuzzySkin.cpp:722-857 apply_fuzzy_skin",
            ],
        ),
        "unsupported_cap_ripple",
        project,
        "14dca2ee591608cdde18f8801ed3b49dc597746d1267bb0afb6970d6114fcfdf",
        false,
    );
    assert!(reference.native.contains("; fuzzy_skin = external\n"));
    assert!(
        reference
            .native
            .contains("; fuzzy_skin_noise_type = ripple\n")
    );
    assert!(reference.native.contains("; enable_overhang_speed = 0\n"));
    let native = first_cap_layer_object(&reference.native);
    assert!(native.contains(";TYPE:Overhang wall\n"));
    assert!(!native.contains(";TYPE:Outer wall\n"));
    std::fs::write(reference.root.join("native.first_cap_object.gcode"), native).unwrap();

    let api = crate::runner::application::slice_project_plate_validated(
        project,
        ares_core::GenerationMetadata::deterministic(2026, 10, 31, 0, 0, 0),
        None,
        &ares_core::ConfigOverrides::default(),
        ares_core::SlicerContext::new(0),
    )
    .await
    .unwrap();
    std::fs::write(reference.root.join("ares.api.gcode"), &api).unwrap();
    let api = first_cap_layer_object(std::str::from_utf8(&api).unwrap());
    std::fs::write(reference.root.join("ares.first_cap_object.gcode"), api).unwrap();
    assert_eq!(
        api, native,
        "the whole unsupported-cap object must match runtime beta fuzzy support restriction: {:?}",
        reference.root,
    );
}
