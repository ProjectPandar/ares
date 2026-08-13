use super::*;

#[test]
fn task22o57_gathers_exact_kind_membership_and_region_order() {
    let first_surfaces = vec![
        surface(RegionSurfaceKind::Top, rectangle(0, 0, 1_000, 1_000)),
        surface(
            RegionSurfaceKind::Internal,
            rectangle(2_000, 0, 3_000, 1_000),
        ),
        surface(
            RegionSurfaceKind::InternalSolid,
            rectangle(4_000, 0, 5_000, 1_000),
        ),
        surface(RegionSurfaceKind::Bottom, rectangle(6_000, 0, 7_000, 1_000)),
    ];
    let second_surfaces = vec![
        surface(
            RegionSurfaceKind::Internal,
            rectangle(8_000, 0, 9_000, 1_000),
        ),
        surface(
            RegionSurfaceKind::InternalSolid,
            rectangle(10_000, 0, 11_000, 1_000),
        ),
    ];
    let first_fill = vec![expolygon(
        rectangle(0, -1_000, 5_000, 2_000),
        vec![rectangle(500, -500, 1_000, 0)],
    )];
    let second_fill = vec![expolygon(
        rectangle(8_000, -1_000, 11_000, 2_000),
        Vec::new(),
    )];
    let regions = [
        region(
            &first_surfaces,
            &first_fill,
            ProcessInfillPattern::Rectilinear,
        ),
        region(
            &second_surfaces,
            &second_fill,
            ProcessInfillPattern::Lightning,
        ),
    ];
    let deep = [rectangle(-2_000, -2_000, 13_000, 3_000)];
    let lines = [line(&[(-2_000, 500), (13_000, 500)])];
    let output = prepare(&deep, &regions, &lines, 100, CoordinateScale::Normal);

    assert_eq!(
        snapshot_polygons(&output.total_top_area),
        vec![vec![(0, 0), (1_000, 0), (1_000, 1_000), (0, 1_000)]]
    );
    assert_eq!(
        snapshot_polygons(&output.lightning_area),
        vec![vec![(8_000, 0), (9_000, 0), (9_000, 1_000), (8_000, 1_000)]]
    );
    assert_eq!(total_area(&output.expansion_area), 4_000_000.0);
    assert_eq!(total_area(&output.total_fill_area), 24_000_000.0);
    assert_eq!(output.anchors.len(), 4);
}

#[test]
fn task22o57_flattens_selected_surface_contours_before_holes() {
    let top = RegionSurface::new(
        RegionSurfaceKind::Top,
        expolygon(
            rectangle(0, 0, 4_000, 4_000),
            vec![
                rectangle(500, 500, 1_000, 1_000),
                rectangle(2_000, 2_000, 3_000, 3_000),
            ],
        ),
    );
    let lightning = RegionSurface::new(
        RegionSurfaceKind::Internal,
        expolygon(
            rectangle(5_000, 0, 9_000, 4_000),
            vec![rectangle(6_000, 1_000, 7_000, 2_000)],
        ),
    );
    let surfaces = [top, lightning];
    let regions = [region(&surfaces, &[], ProcessInfillPattern::Lightning)];
    let deep = [rectangle(-1_000, -1_000, 10_000, 5_000)];
    let output = prepare(&deep, &regions, &[], 100, CoordinateScale::Normal);

    assert_eq!(output.total_top_area.len(), 3);
    assert_eq!(
        snapshot_polygons(&output.total_top_area)[1..],
        [
            vec![(500, 500), (1_000, 500), (1_000, 1_000), (500, 1_000)],
            vec![
                (2_000, 2_000),
                (3_000, 2_000),
                (3_000, 3_000),
                (2_000, 3_000)
            ],
        ]
    );
    assert_eq!(output.lightning_area.len(), 2);
    assert_eq!(
        snapshot_polygons(&output.lightning_area)[1],
        vec![
            (6_000, 1_000),
            (7_000, 1_000),
            (7_000, 2_000),
            (6_000, 2_000)
        ]
    );
}

#[test]
fn task22o57_non_lightning_internal_does_not_enter_lightning_area() {
    let surfaces = [surface(
        RegionSurfaceKind::Internal,
        rectangle(0, 0, 1_000, 1_000),
    )];
    let regions = [region(&surfaces, &[], ProcessInfillPattern::Grid)];
    let deep = [rectangle(-1_000, -1_000, 2_000, 2_000)];

    let output = prepare(&deep, &regions, &[], 100, CoordinateScale::Normal);
    assert!(output.lightning_area.is_empty());
    assert_eq!(total_area(&output.expansion_area), 1_000_000.0);
}

#[test]
fn task22o57_empty_regions_and_lines_still_normalize_deep_geometry() {
    let deep = [
        rectangle(0, 0, 1_000, 1_000),
        rectangle(500, 0, 1_500, 1_000),
    ];
    let output = prepare(&deep, &[], &[], 100, CoordinateScale::Normal);

    assert_eq!(output.deep_infill_area.len(), 1);
    assert_eq!(total_area(&output.deep_infill_area), 2_340_000.0);
    assert!(output.expansion_area.is_empty());
    assert!(output.total_fill_area.is_empty());
    assert!(output.total_top_area.is_empty());
    assert!(output.anchors.is_empty());
    assert_eq!(total_area(&output.internal_unsupported_area), 360_000.0);
}
