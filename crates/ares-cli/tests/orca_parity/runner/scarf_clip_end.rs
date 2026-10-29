//! `Polyline3::clip_end` (Polyline.cpp:683-702) clips with
//! `last_point + v * (distance / sqrt(lsqr))`, then casts to coord_t.
//! `ExtrusionLoopSloped::clip_front` (ExtrusionEntity.cpp:551-573)
//! reaches it by reversing the scarf start. Replacing that displacement
//! with weighted lerp shifts the Voron all/inner0 first travel by one
//! coord_t unit: X103.638 instead of Native X103.637 at line 3423.

#[test]
fn voron_scarf_clip_end_displacement() {
    crate::runner::native_stream_case::check(
        (
            "scarf-clip-end",
            &[
                "Polyline.cpp:683-702:clip_end-additive-displacement-and-squared-length-gate",
                "ExtrusionEntity.cpp:527-573:clip_slope-and-reversed-front-clipping",
                "GCode.cpp:6366-6381:first-path-point-travel",
            ],
        ),
        "voron-all-inner0",
        include_bytes!("../../../../../tests/parity/scarf-clip-end/voron-all-inner0.3mf"),
        "63c8325570c7c2ab40beff07ef2894d696ace59457e94a53af1d18fb42a792ae",
        (false, |_, _| {}),
    );
}
