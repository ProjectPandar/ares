use crate::{
    geometry::{CoordinateScale, ExPolygon, Point, Polygon},
    project_slice::perimeters::classic::{
        gap_extrusion::{GapFillCollection, PreparedGapExtrusionSurface},
        perimeter_append::{
            AppendedPerimeterCollections, InactiveOuterBrimReordering,
            InactiveOverhangReorientation, InactivePostCollectionBranches, InactiveWallReordering,
        },
        top_split::{PreparedTopSplitSurface, TopSplitOutcome, TopSplitUpperSource},
    },
};

use super::super::{
    geometry::{stage_record, stage_surface},
    preflight::{TestSurfaceInput, validate_surface_for_test},
    types::ValidatedRecord,
};

#[test]
fn task22o15_narrow_remaining_collapses_before_internal_append() {
    let output = stage_surface(
        &source(vec![rectangle(0, 0, 400, 2_000)]),
        &top(Vec::new(), Vec::new()),
        validated(-1, 1_000, 0.0, 0.0),
    )
    .unwrap();
    assert!(output.fill_surfaces.is_empty());
    assert!(output.fill_no_overlap.is_empty());
}

#[test]
fn task22o15_top_intersection_expansion_and_no_overlap_are_literal() {
    let output = stage_surface(
        &source(Vec::new()),
        &top(
            vec![rectangle(500, 500, 1_500, 1_500)],
            vec![rectangle(0, 0, 2_000, 2_000)],
        ),
        validated(0, 1_000, 0.0, 20.0),
    )
    .unwrap();
    assert_eq!(
        surface_contours(&output.fill_surfaces),
        vec![vec![
            (2_200, 2_200),
            (-200, 2_200),
            (-200, -200),
            (2_200, -200)
        ]],
    );
    assert_eq!(
        expolygon_contours(&output.fill_no_overlap),
        vec![vec![(2_000, 2_000), (0, 2_000), (0, 0), (2_000, 0)]],
    );
    for surface in &output.fill_surfaces {
        let (kind, _, thickness, layers, angle, extra) = surface.as_parts();
        assert_eq!(kind as u8, 4);
        assert_eq!((thickness, layers, angle, extra), (-1.0, 1, -1.0, 0));
    }
}

#[test]
fn task22o15_raw_resolution_changes_geometry_from_one_fifth_tolerance() {
    let remaining = vec![ExPolygon::new(
        polygon(&[
            (0, 0),
            (5_000, 5_000),
            (10_000, 0),
            (10_000, 20_000),
            (0, 20_000),
        ]),
        Vec::new(),
    )];
    let mut raw = validated(-1, 1_000, 0.0, 0.0);
    raw.overlap.scaled_resolution = 12_000.0;
    let mut adjusted = raw;
    adjusted.overlap.scaled_resolution = 2_400.0;
    let raw = stage_surface(
        &source(remaining.clone()),
        &top(Vec::new(), Vec::new()),
        raw,
    )
    .unwrap();
    let adjusted =
        stage_surface(&source(remaining), &top(Vec::new(), Vec::new()), adjusted).unwrap();
    assert!(raw.fill_no_overlap.is_empty());
    assert_eq!(
        expolygon_contours(&adjusted.fill_no_overlap),
        vec![vec![
            (10_000, 20_000),
            (0, 20_000),
            (0, 0),
            (5_000, 5_000),
            (10_000, 0),
        ]],
    );
}

#[test]
fn task22o15_multiple_surfaces_aggregate_in_source_order() {
    let sources = [
        source(vec![rectangle(0, 0, 4_000, 4_000)]),
        source(vec![rectangle(10_000, 0, 14_000, 4_000)]),
    ];
    let tops = [top(Vec::new(), Vec::new()), top(Vec::new(), Vec::new())];
    let validated = ValidatedRecord {
        surfaces: vec![
            validated(-1, 1_000, 0.0, 0.0),
            validated(-1, 1_000, 0.0, 0.0),
        ],
    };
    let output = stage_record(&sources, &tops, &validated).unwrap();
    assert_eq!(
        surface_contours(&output.fill_surfaces),
        vec![
            vec![(4_000, 4_000), (0, 4_000), (0, 0), (4_000, 0)],
            vec![(14_000, 4_000), (10_000, 4_000), (10_000, 0), (14_000, 0)],
        ],
    );
    assert_eq!(
        expolygon_contours(&output.fill_no_overlap),
        vec![
            vec![(4_000, 4_000), (0, 4_000), (0, 0), (4_000, 0)],
            vec![(14_000, 4_000), (10_000, 4_000), (10_000, 0), (14_000, 0)],
        ],
    );
}

fn validated(
    loop_number: i32,
    solid_infill_spacing: i64,
    ordinary_percent: f64,
    top_percent: f64,
) -> super::super::types::ValidatedSurface {
    validate_surface_for_test(TestSurfaceInput {
        loop_number,
        external_spacing: 1_000,
        perimeter_spacing: 800,
        solid_infill_spacing,
        layer_id: 3,
        has_upper: true,
        ordinary_percent,
        top_percent,
        scale: CoordinateScale::Normal,
    })
    .unwrap()
}

fn source(remaining: Vec<ExPolygon>) -> PreparedGapExtrusionSurface {
    PreparedGapExtrusionSurface {
        source_index: 7,
        inactive: InactivePostCollectionBranches {
            overhang_reorientation: InactiveOverhangReorientation::Disabled {
                overhang_reverse_internal_only: false,
            },
            wall_reordering: InactiveWallReordering::InnerOuter {
                outer_brim: InactiveOuterBrimReordering::WidthNotPositive { brim_width: 0.0 },
            },
        },
        appended: AppendedPerimeterCollections::default(),
        medial: None,
        gap_fill: GapFillCollection::default(),
        remaining,
    }
}

fn top(top_fills: Vec<ExPolygon>, fill_clip: Vec<ExPolygon>) -> PreparedTopSplitSurface {
    PreparedTopSplitSurface {
        source_index: 7,
        initial_loop_number: 0,
        effective_loop_number: 0,
        normal_first_offset: Vec::new(),
        smaller_first_offset: Vec::new(),
        remaining: Vec::new(),
        top_fills,
        fill_clip,
        outcome: TopSplitOutcome::Disabled,
        upper_source: TopSplitUpperSource::WholeLayer,
    }
}

fn rectangle(min_x: i64, min_y: i64, max_x: i64, max_y: i64) -> ExPolygon {
    ExPolygon::new(
        polygon(&[
            (min_x, min_y),
            (max_x, min_y),
            (max_x, max_y),
            (min_x, max_y),
        ]),
        Vec::new(),
    )
}

fn polygon(points: &[(i64, i64)]) -> Polygon {
    Polygon::new(points.iter().map(|&(x, y)| Point::new(x, y)).collect())
}

fn surface_contours(
    surfaces: &[crate::project_slice::region_slices::RegionSurface],
) -> Vec<Vec<(i64, i64)>> {
    surfaces
        .iter()
        .map(|surface| contour(surface.as_parts().1))
        .collect()
}

fn expolygon_contours(expolygons: &[ExPolygon]) -> Vec<Vec<(i64, i64)>> {
    expolygons.iter().map(contour).collect()
}

fn contour(expolygon: &ExPolygon) -> Vec<(i64, i64)> {
    expolygon
        .contour()
        .points()
        .iter()
        .map(|point| (point.x(), point.y()))
        .collect()
}
