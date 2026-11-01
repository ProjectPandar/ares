//! Layer-0 wall order under an outer+inner brim: `brim_puts_outer_walls_first`
//! (PerimeterGenerator.cpp:3204-3211) covers btOuterAndInner, and :1856-1865
//! reverses the whole entity table so the outer wall continues inwards from
//! the finished brim. Slices the outer-inner-brim narrow-neck fixture and
//! pins that the first wall marker in the Ares output is the outer wall.

#[tokio::test]
async fn layer_zero_prints_outer_wall_first_under_outer_inner_brim() {
    let project = include_bytes!("../../../tests/parity/outer-inner-brim/narrow-neck.3mf");
    let gcode = ares_core::slice_project_plate_validated(
        project,
        ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
        None,
        &ares_core::ConfigOverrides::default(),
        ares_core::SlicerContext::new(0),
    )
    .await
    .unwrap();
    let text = std::str::from_utf8(&gcode).unwrap();
    let outer = text.find(";TYPE:Outer wall").expect("an outer wall marker");
    let inner = text.find(";TYPE:Inner wall").expect("an inner wall marker");
    assert!(
        outer < inner,
        "layer 0 must continue inwards from the brim: outer wall first"
    );
}
