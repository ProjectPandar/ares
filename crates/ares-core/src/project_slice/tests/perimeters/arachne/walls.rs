// Per-surface arachne wall generation tests (OrcaSlicer
// `PerimeterGenerator.cpp:2111-2176,2475-2531`).

use crate::{
    ObjectOptions, OrcaFloat, OrcaFloats, RegionOptions,
    geometry::{CoordinateScale, ExPolygon, Point, Polygon},
    project_slice::{
        perimeters::arachne::{
            config::validate_record,
            walls::{ArachneSpacings, InfillOverlapPercents},
        },
        region_slices::{RegionSurface, RegionSurfaceKind},
    },
};

const SCALE: CoordinateScale = CoordinateScale::Normal;

fn nozzles() -> OrcaFloats {
    OrcaFloats(vec![OrcaFloat(0.4)])
}

fn region_with(one_wall_first_layer: bool, one_wall_top: bool) -> RegionOptions {
    let mut region = RegionOptions::from_base(&crate::ProjectSettings::default().process.region);
    region.only_one_wall_first_layer.0 = one_wall_first_layer;
    region.only_one_wall_top.0 = one_wall_top;
    region
}

fn square(millimeters: i64) -> ExPolygon {
    let scaled = SCALE.checked_scale(millimeters as f64).unwrap();
    ExPolygon::new(
        Polygon::new(vec![
            Point::new(0, 0),
            Point::new(scaled, 0),
            Point::new(scaled, scaled),
            Point::new(0, scaled),
        ]),
        Vec::new(),
    )
}

fn surface(millimeters: i64) -> RegionSurface {
    RegionSurface::new(RegionSurfaceKind::Internal, square(millimeters))
}

use crate::project_slice::perimeters::arachne::config::ArachneRecordConfig;

fn config(layer_id: usize, upper: Option<usize>) -> ArachneRecordConfig {
    config_with(layer_id, upper, region_with(false, false))
}

fn config_with(
    layer_id: usize,
    upper: Option<usize>,
    region: RegionOptions,
) -> ArachneRecordConfig {
    validate_record(
        &region,
        &ObjectOptions::from_base(&crate::ProjectSettings::default().process.object),
        layer_id,
        upper,
        crate::project_slice::perimeters::arachne::config::PrintParameters {
            enable_arc_fitting: false,
            resolution: 0.012,
            nozzle_diameters: &nozzles(),
            scale: SCALE,
        },
    )
    .unwrap()
}

fn spacings() -> ArachneSpacings {
    let solid_infill_flow = crate::project_slice::perimeters::types::Flow {
        width: 0.42,
        height: 0.16,
        spacing: 0.42,
        nozzle_diameter: 0.4,
        bridge: false,
        mm3_per_mm: 0.05,
    };
    let perimeter_flow = crate::project_slice::perimeters::types::Flow {
        spacing: 0.385,
        ..solid_infill_flow
    };
    let external_perimeter_flow = crate::project_slice::perimeters::types::Flow {
        spacing: 0.39,
        ..solid_infill_flow
    };
    ArachneSpacings::new(
        SCALE,
        &perimeter_flow,
        &external_perimeter_flow,
        &solid_infill_flow,
    )
    .unwrap()
}

#[test]
fn task22w12_generates_real_variable_width_walls_for_one_island() {
    let config = config(1, Some(2));
    let generated = crate::project_slice::perimeters::arachne::walls::generate_surface_walls(
        &surface(20),
        &config,
        &spacings(),
        crate::project_slice::perimeters::arachne::walls::SurfaceWallParameters {
            wall_loops: 3,
            layer_height: 0.16,
            scale: SCALE,
        },
        None,
    )
    .unwrap();

    // Three wall levels for `wall_loops = 3` with a filled inner contour
    // (`PerimeterGenerator.cpp:2148-2176`).
    assert_eq!(generated.toolpaths.len(), 3);
    let lines = generated.toolpaths.iter().flatten().count();
    assert!(lines > 0);
    assert!(!generated.infill_contour.is_empty());
}

#[test]
fn task22w13_bottom_layer_single_wall_rule_resets_the_loop_count() {
    // `only_one_wall_first_layer` with `raft_layers = 0`
    // (`PerimeterGenerator.cpp:2121-2122`).
    let config = config_with(0, Some(1), region_with(true, false));
    let generated = crate::project_slice::perimeters::arachne::walls::generate_surface_walls(
        &surface(20),
        &config,
        &spacings(),
        crate::project_slice::perimeters::arachne::walls::SurfaceWallParameters {
            wall_loops: 3,
            layer_height: 0.16,
            scale: SCALE,
        },
        None,
    )
    .unwrap();

    assert_eq!(generated.toolpaths.len(), 1);
}

#[test]
fn task22w14_topmost_layer_single_wall_rule_resets_the_loop_count() {
    // `only_one_wall_top` on the topmost layer
    // (`PerimeterGenerator.cpp:2125-2127`).
    let config = config_with(11, None, region_with(false, true));
    let generated = crate::project_slice::perimeters::arachne::walls::generate_surface_walls(
        &surface(20),
        &config,
        &spacings(),
        crate::project_slice::perimeters::arachne::walls::SurfaceWallParameters {
            wall_loops: 3,
            layer_height: 0.16,
            scale: SCALE,
        },
        None,
    )
    .unwrap();

    assert!(config.is_topmost_layer);
    assert_eq!(generated.toolpaths.len(), 1);
}

#[test]
fn task22w15_ext_perimeter_spacing2_averages_the_plain_spacings() {
    let spacings = spacings();
    // `PerimeterGenerator.cpp:2103`: 0.5 * (0.39f32 + 0.385f32) truncates
    // through the source f32 sum and `scaled<coord_t>` (`Point.hpp:662-668`).
    assert_eq!(spacings.ext_perimeter_spacing2, 387_499);
}

#[test]
fn task22w16_infill_boundary_applies_the_wall_overlap_percent() {
    let generated = crate::project_slice::perimeters::arachne::walls::generate_surface_walls(
        &surface(20),
        &config(1, Some(2)),
        &spacings(),
        crate::project_slice::perimeters::arachne::walls::SurfaceWallParameters {
            wall_loops: 3,
            layer_height: 0.16,
            scale: SCALE,
        },
        None,
    )
    .unwrap();
    let boundary = crate::project_slice::perimeters::arachne::walls::surface_infill_boundary(
        generated.infill_contour,
        crate::project_slice::perimeters::arachne::walls::WallCounts {
            wall_count: generated.toolpaths.len(),
            loop_number: generated.toolpaths.len() as i32 - 1,
        },
        &spacings(),
        0.012,
        crate::project_slice::perimeters::arachne::walls::BoundaryOverlap {
            overlaps: InfillOverlapPercents {
                infill_wall_overlap: 20.0,
                top_bottom_infill_wall_overlap: 25.0,
            },
            top_or_bottom: false,
            scale: SCALE,
            top_expolygons: &generated.top_expolygons,
        },
    )
    .unwrap();

    // Three walls keep non-empty infill surfaces; the no-overlap set stays
    // inside the overlap-expanded fill surfaces.
    assert!(!boundary.fill_surfaces.is_empty());
    assert!(!boundary.fill_no_overlap.is_empty());
}

#[test]
fn task22w17_infill_boundary_truncates_offsets_before_halving() {
    // case-w4hWv8 layer 2 (Anker M5 0.2 nozzle, exact 10 mm cube): the arachne
    // inner contour of the wall call is the exact square [±4361464]^2 and
    // `solid_infill_spacing` is 209269. Upstream computes
    // `coord_t(209269 * 0.6) = 125561` (`PerimeterGenerator.cpp:2506`) and
    // `coord_t(scale_(0.15 * 209269)) = 31390` for the overlap inset
    // (`PerimeterGenerator.cpp:2492-2501`) before halving, so the offset2
    // deltas land on the half-integer ClipperLib rounding boundary
    // (floor(x + 0.5)). Skipping either truncation moves the no-overlap square
    // by one scaled unit per side and flips the concentric corner incenter
    // rounding, changing the wall-B width from 0.264891 to 0.26489.
    let inner_contour = vec![ExPolygon::new(
        Polygon::new(vec![
            Point::new(4361464, 4361464),
            Point::new(-4361464, 4361464),
            Point::new(-4361464, -4361464),
            Point::new(4361464, -4361464),
        ]),
        Vec::new(),
    )];
    let spacings = ArachneSpacings {
        perimeter_spacing: 209269,
        perimeter_width: 209269,
        ext_perimeter_width: 220000,
        ext_perimeter_spacing: 209269,
        ext_perimeter_spacing2: 209269,
        solid_infill_spacing: 209269,
    };
    let boundary = crate::project_slice::perimeters::arachne::walls::surface_infill_boundary(
        inner_contour,
        crate::project_slice::perimeters::arachne::walls::WallCounts {
            wall_count: 3,
            loop_number: 2,
        },
        &spacings,
        12500.0,
        crate::project_slice::perimeters::arachne::walls::BoundaryOverlap {
            overlaps: InfillOverlapPercents {
                infill_wall_overlap: 15.0,
                top_bottom_infill_wall_overlap: 15.0,
            },
            top_or_bottom: false,
            scale: SCALE,
            top_expolygons: &[],
        },
    )
    .unwrap();

    let corners = |expolygons: &[ExPolygon]| -> Vec<(i64, i64)> {
        let mut points = expolygons
            .iter()
            .flat_map(|expolygon| expolygon.contour().points().iter())
            .map(|point| (point.x(), point.y()))
            .collect::<Vec<_>>();
        points.sort_unstable();
        points
    };
    // offset2(-62780.5, +62780.5) rounds each side half-up: [−4361463, 4361465]^2.
    assert_eq!(
        corners(&boundary.fill_no_overlap),
        vec![
            (-4361463, -4361463),
            (-4361463, 4361465),
            (4361465, -4361463),
            (4361465, 4361465),
        ]
    );
    // offset2(-62780.5, 31390 + 62780.5): [−4392853, 4392855]^2.
    assert_eq!(
        corners(&boundary.fill_surfaces),
        vec![
            (-4392853, -4392853),
            (-4392853, 4392855),
            (4392855, -4392853),
            (4392855, 4392855),
        ]
    );
}
