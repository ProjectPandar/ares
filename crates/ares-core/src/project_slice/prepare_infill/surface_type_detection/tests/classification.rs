use crate::{
    geometry::{ExPolygon, Point, Polygon},
    project_slice::{
        prepare_infill::surface_type_detection::{
            GeometryStep,
            cracks::crack_threshold,
            geometry::{opening_offset, paths, safety_difference},
            stage::{apply_spiral_surface_types, classify_slices},
        },
        region_slices::{RegionSurface, RegionSurfaceKind},
    },
};

fn rectangle(x0: i64, y0: i64, x1: i64, y1: i64) -> ExPolygon {
    ExPolygon::new(
        Polygon::new(vec![
            Point::new(x0, y0),
            Point::new(x1, y0),
            Point::new(x1, y1),
            Point::new(x0, y1),
        ]),
        Vec::new(),
    )
}

fn kinds(surfaces: &[RegionSurface]) -> Vec<RegionSurfaceKind> {
    surfaces
        .iter()
        .map(|surface| surface.as_parts().0)
        .collect()
}

#[test]
fn spiral_mode_turns_the_last_base_layer_top_and_body_layers_internal() {
    let surface = || RegionSurface::internal(rectangle(0, 0, 1_000, 1_000));
    let mut last_base = vec![surface()];
    let mut body = vec![surface()];

    apply_spiral_surface_types(&mut last_base, true, 4, 5, 50);
    apply_spiral_surface_types(&mut body, true, 49, 5, 50);

    assert_eq!(kinds(&last_base), [RegionSurfaceKind::Top]);
    assert_eq!(kinds(&body), [RegionSurfaceKind::Internal]);
}

#[test]
fn arithmetic_order_distinguishes_the_source_casts() {
    let width = 16_777_217_i64;
    assert_eq!(opening_offset(width).to_bits(), 0x49cc_cccd);
    assert_ne!(opening_offset(width), (width as f64 / 10.0) as f32);
    assert_eq!(crack_threshold(width), -25_165_826.0_f32);
    assert_ne!(crack_threshold(width), (-(width as f32) * 1.5_f32));
}

#[test]
fn single_layer_bottom_wins_and_preserves_source_metadata() {
    let source = vec![RegionSurface::internal_with_metadata(
        rectangle(0, 0, 1_000, 1_000),
        0.3,
        2,
        0.75,
        4,
    )];
    let output =
        classify_slices(&source, None, None, 100, RegionSurfaceKind::BottomBridge).unwrap();
    assert_eq!(kinds(&output), [RegionSurfaceKind::Bottom]);
    let (_, _, thickness, layers, angle, extra) = output[0].as_parts();
    assert_eq!((thickness, layers, angle, extra), (0.3, 2, 0.75, 4));
}

#[test]
fn final_top_preserves_metadata_when_lower_difference_is_empty() {
    let geometry = rectangle(0, 0, 1_000, 1_000);
    let source = vec![RegionSurface::internal_with_metadata(
        geometry.clone(),
        0.4,
        3,
        1.25,
        5,
    )];
    let lower = vec![geometry];
    let output = classify_slices(
        &source,
        None,
        Some(&lower),
        100,
        RegionSurfaceKind::BottomBridge,
    )
    .unwrap();
    assert_eq!(kinds(&output), [RegionSurfaceKind::Top]);
    let (_, _, thickness, layers, angle, extra) = output[0].as_parts();
    assert_eq!((thickness, layers, angle, extra), (0.4, 3, 1.25, 5));
}

#[test]
fn covered_interior_is_fresh_internal_geometry() {
    let geometry = rectangle(0, 0, 1_000, 1_000);
    let source = vec![RegionSurface::internal_with_metadata(
        geometry.clone(),
        0.4,
        3,
        1.25,
        5,
    )];
    let neighbor = vec![geometry];
    let output = classify_slices(
        &source,
        Some(&neighbor),
        Some(&neighbor),
        100,
        RegionSurfaceKind::BottomBridge,
    )
    .unwrap();
    assert_eq!(kinds(&output), [RegionSurfaceKind::Internal]);
    let (_, _, thickness, layers, angle, extra) = output[0].as_parts();
    assert_eq!((thickness, layers, angle, extra), (-1.0, 1, -1.0, 0));
}

#[test]
fn lower_void_uses_the_source_selected_bottom_kind() {
    let geometry = rectangle(0, 0, 2_000, 2_000);
    let source = vec![RegionSurface::internal(geometry.clone())];
    let upper = vec![geometry];
    for kind in [RegionSurfaceKind::BottomBridge, RegionSurfaceKind::Bottom] {
        let output = classify_slices(&source, Some(&upper), Some(&[]), 100, kind).unwrap();
        assert!(kinds(&output).contains(&kind));
    }
}

#[test]
fn actual_out_of_range_geometry_uses_the_stable_o17_error() {
    let source = vec![RegionSurface::internal(rectangle(
        i64::MAX - 1_000,
        0,
        i64::MAX,
        1_000,
    ))];
    let error = match classify_slices(
        &source,
        Some(&[]),
        Some(&[]),
        100,
        RegionSurfaceKind::BottomBridge,
    ) {
        Err(error) => error,
        Ok(_) => panic!("out-of-range O17 geometry must fail"),
    };
    assert_eq!(
        error,
        crate::SliceError::InvalidInput(
            "surface-type detection geometry is outside the supported Clipper range".to_owned()
        )
    );
}

#[test]
fn narrow_external_difference_collapses_during_source_opening() {
    let geometry = rectangle(0, 0, 100, 1_000);
    let source = vec![RegionSurface::internal(geometry.clone())];
    let lower = vec![geometry];
    let output = classify_slices(
        &source,
        Some(&[]),
        Some(&lower),
        1_000,
        RegionSurfaceKind::BottomBridge,
    )
    .unwrap();
    assert!(!kinds(&output).contains(&RegionSurfaceKind::Top));
}

#[test]
fn terminal_partial_overlap_reconstructs_top_with_default_metadata() {
    let source = vec![RegionSurface::internal_with_metadata(
        rectangle(0, 0, 2_000, 1_000),
        0.4,
        3,
        1.25,
        5,
    )];
    let lower = vec![rectangle(0, 0, 1_000, 1_000)];
    let output = classify_slices(
        &source,
        None,
        Some(&lower),
        100,
        RegionSurfaceKind::BottomBridge,
    )
    .unwrap();
    let top = output
        .iter()
        .find(|surface| surface.as_parts().0 == RegionSurfaceKind::Top)
        .unwrap();
    let (_, _, thickness, layers, angle, extra) = top.as_parts();
    assert_eq!((thickness, layers, angle, extra), (-1.0, 1, -1.0, 0));
}

#[test]
fn safety_difference_expands_only_the_clip_by_exactly_ten_units() {
    let subject = rectangle(0, 0, 100, 100);
    let clip = rectangle(105, 0, 200, 100);
    let output = safety_difference(
        std::slice::from_ref(&subject),
        std::slice::from_ref(&clip),
        GeometryStep::TopSafetyDifference,
    )
    .unwrap();
    let points = output[0].contour().points();
    assert_eq!(points.iter().map(|point| point.x()).min(), Some(0));
    assert_eq!(points.iter().map(|point| point.x()).max(), Some(95));
}

#[test]
fn multiple_holed_surfaces_flatten_contour_then_holes_in_surface_order() {
    let first_hole = Polygon::new(vec![
        Point::new(10, 10),
        Point::new(10, 20),
        Point::new(20, 20),
        Point::new(20, 10),
    ]);
    let second_hole = Polygon::new(vec![
        Point::new(110, 10),
        Point::new(110, 20),
        Point::new(120, 20),
        Point::new(120, 10),
    ]);
    let first = ExPolygon::new(rectangle(0, 0, 50, 50).into_parts().0, vec![first_hole]);
    let second = ExPolygon::new(rectangle(100, 0, 150, 50).into_parts().0, vec![second_hole]);
    let surfaces = vec![
        RegionSurface::new(RegionSurfaceKind::Top, first.clone()),
        RegionSurface::new(RegionSurfaceKind::Top, second.clone()),
    ];
    assert_eq!(
        paths(&surfaces),
        vec![
            first.contour().clone(),
            first.holes()[0].clone(),
            second.contour().clone(),
            second.holes()[0].clone(),
        ]
    );
}

#[test]
fn minimal_valid_coordinates_remain_accepted() {
    let source = vec![RegionSurface::internal(rectangle(0, 0, 1, 1))];
    assert!(classify_slices(&source, None, None, 1, RegionSurfaceKind::BottomBridge,).is_ok());
}
