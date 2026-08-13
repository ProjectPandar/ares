use super::super::fill_surface;
use super::support::point;
use crate::geometry::{ClipperError, CoordinateScale, ExPolygon, Polygon};

const HI_RANGE: i64 = 0x3fff_ffff_ffff_ffff;

fn params(spacing: f64, density: f32) -> super::super::CrossHatchFillParams {
    super::super::CrossHatchFillParams {
        z: 0.0,
        spacing,
        overlap: 0.0,
        angle: 0.0,
        density,
        multiline: 1,
        anchor_length: 0.0,
        anchor_length_max: 20.0,
        dont_sort: false,
    }
}

fn rectangle(min_x: i64, min_y: i64, max_x: i64, max_y: i64) -> Polygon {
    Polygon::new(vec![
        point(min_x, min_y),
        point(max_x, min_y),
        point(max_x, max_y),
        point(min_x, max_y),
    ])
}

#[test]
fn task22o45_fully_consumed_public_inset_returns_empty_without_mutating_source() {
    let surface = ExPolygon::new(
        rectangle(0, 0, 100, 100),
        vec![Polygon::new(vec![
            point(10, 10),
            point(10, 90),
            point(90, 90),
            point(90, 10),
        ])],
    );
    let before = surface.clone();

    let actual = fill_surface(
        &surface,
        params(40.0 * CoordinateScale::Normal.factor(), 1.0),
        CoordinateScale::Normal,
    );

    assert_eq!(actual, Ok(Vec::new()));
    assert_eq!(surface, before);
}

#[test]
fn task22o45_short_clipped_remnant_returns_empty_and_skips_connection() {
    let surface = ExPolygon::new(rectangle(25, 25, 175, 175), Vec::new());
    let before = surface.clone();

    let actual = fill_surface(
        &surface,
        params(100.0 * CoordinateScale::Normal.factor(), 1.0),
        CoordinateScale::Normal,
    );

    assert_eq!(actual, Ok(Vec::new()));
    assert_eq!(surface, before);
}

#[test]
fn task22o45_initial_offset_range_error_is_direct_and_source_is_unchanged() {
    let surface = ExPolygon::new(
        rectangle(HI_RANGE + 1, 0, HI_RANGE + 1_000_001, 1_000_000),
        Vec::new(),
    );
    let before = surface.clone();

    let actual = fill_surface(
        &surface,
        params(100.0 * CoordinateScale::Normal.factor(), 1.0),
        CoordinateScale::Normal,
    );

    assert_eq!(actual, Err(ClipperError::CoordinateOutOfRange));
    assert_eq!(surface, before);
}

#[test]
fn task22o45_later_grid_alignment_range_error_is_atomic_and_nonmutating() {
    let min_x = -HI_RANGE + 10_000_000;
    let surface = ExPolygon::new(
        rectangle(min_x, 0, min_x + 1_000_000, 1_000_000),
        Vec::new(),
    );
    let before = surface.clone();

    let actual = fill_surface(
        &surface,
        params(
            100.0 * CoordinateScale::Normal.factor(),
            f32::from_bits(0x24cf_869f),
        ),
        CoordinateScale::Normal,
    );

    assert_eq!(actual, Err(ClipperError::CoordinateOutOfRange));
    assert_eq!(surface, before);
}

#[test]
fn task22o45_rotated_open_intersection_range_error_is_direct_and_nonmutating() {
    let min = HI_RANGE - 10_000_000;
    let surface = ExPolygon::new(
        rectangle(min, min, min + 1_000_000, min + 1_000_000),
        Vec::new(),
    );
    let before = surface.clone();
    let mut fill_params = params(100_000.0 * CoordinateScale::Normal.factor(), 1.0);
    fill_params.angle = f32::from_bits(0x3f49_0fdb);

    let actual = fill_surface(&surface, fill_params, CoordinateScale::Normal);

    assert_eq!(actual, Err(ClipperError::CoordinateOutOfRange));
    assert_eq!(surface, before);
}

#[test]
fn task22o45_later_split_component_error_discards_the_completed_prefix() {
    let density = f32::from_bits(0x28a8_6f44);
    let spacing = 20_000.0 * CoordinateScale::Normal.factor();
    let mut fill_params = params(spacing, density);
    fill_params.z = f64::from_bits(0x423a_e4d0_7a66_bf37);
    let grid = 1_155_080_177_667_469_056;
    let safe_component = ExPolygon::new(
        rectangle(grid - 50_000, grid - 50_000, grid + 50_000, grid + 50_000),
        Vec::new(),
    );
    let safe_output = fill_surface(&safe_component, fill_params, CoordinateScale::Normal).unwrap();
    assert!(!safe_output.is_empty());

    let bad_min_x = -HI_RANGE + 10_000_000;
    // The inward offset removes this narrow corridor; frozen Clipper sibling order emits the
    // right safe lobe before the far-left lobe whose grid alignment fails.
    let surface = ExPolygon::new(
        Polygon::new(vec![
            point(bad_min_x, grid - 50_000),
            point(bad_min_x + 1_000_000, grid - 50_000),
            point(bad_min_x + 1_000_000, grid - 5_000),
            point(grid - 50_000, grid - 5_000),
            point(grid - 50_000, grid - 50_000),
            point(grid + 50_000, grid - 50_000),
            point(grid + 50_000, grid + 50_000),
            point(grid - 50_000, grid + 50_000),
            point(grid - 50_000, grid + 5_000),
            point(bad_min_x + 1_000_000, grid + 5_000),
            point(bad_min_x + 1_000_000, grid + 50_000),
            point(bad_min_x, grid + 50_000),
        ]),
        Vec::new(),
    );
    let before = surface.clone();

    let actual = fill_surface(&surface, fill_params, CoordinateScale::Normal);

    assert_eq!(actual, Err(ClipperError::CoordinateOutOfRange));
    assert_eq!(surface, before);
}
