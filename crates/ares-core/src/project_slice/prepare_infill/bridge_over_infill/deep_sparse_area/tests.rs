use super::{DeepSparseLayer, gather_deep_sparse_infill_area};
use crate::{
    geometry::{ClipperError, CoordinateScale, ExPolygon, Point, Polygon},
    project_slice::{
        layers::PlannedLayer,
        region_slices::{RegionSurface, RegionSurfaceKind},
    },
};

#[test]
fn task22o47_preserves_source_depth_arithmetic_and_stops_before_deeper_layers() {
    let fixture = Fixture::new(vec![
        layer(1, 0.80, vec![internal(rect(0, 0, 10, 10))], 15.0),
        layer(2, 0.81995, vec![internal(rect(20, 0, 30, 10))], 15.0),
        layer(3, 0.90, vec![internal(rect(40, 0, 50, 10))], 15.0),
        layer(4, 1.00, vec![internal(rect(60, 0, 70, 10))], 15.0),
    ]);
    let before = snapshot(&fixture);

    let output = gather(&fixture, 3, 0.2).unwrap();

    assert_eq!(bounds(&output), vec![(20, 0, 30, 10), (40, 0, 50, 10)]);
    assert_eq!(snapshot(&fixture), before);
}

#[test]
fn task22o47_multiplies_target_height_in_f32_before_promoting_to_f64() {
    let fixture = Fixture::new(vec![
        layer(1, 0.80, vec![internal(rect(0, 0, 10, 10))], 15.0),
        layer(2, 0.819_900_005, vec![internal(rect(20, 0, 30, 10))], 15.0),
        layer(3, 0.90, vec![internal(rect(40, 0, 50, 10))], 15.0),
        layer(4, 1.00, Vec::new(), 15.0),
    ]);

    let output = gather(&fixture, 3, 0.2).unwrap();

    assert_eq!(bounds(&output), vec![(40, 0, 50, 10)]);
}

#[test]
fn task22o47_always_includes_immediate_lower_layer_below_depth_threshold() {
    let fixture = Fixture::new(vec![
        layer(1, 0.40, vec![internal(rect(0, 0, 10, 10))], 15.0),
        layer(2, 0.50, vec![internal(rect(20, 0, 30, 10))], 15.0),
        layer(3, 1.00, vec![internal(rect(40, 0, 50, 10))], 15.0),
    ]);

    let output = gather(&fixture, 2, 0.2).unwrap();

    assert_eq!(bounds(&output), vec![(20, 0, 30, 10)]);
}

#[test]
fn task22o47_uses_each_lower_layer_density_and_internal_void_classification() {
    let fixture = Fixture::new(vec![
        layer(
            1,
            0.60,
            vec![
                internal(rect(0, 0, 30, 20)),
                surface(RegionSurfaceKind::InternalVoid, rect(40, 0, 50, 10)),
                surface(RegionSurfaceKind::InternalSolid, rect(10, 0, 20, 20)),
                surface(RegionSurfaceKind::Top, rect(40, 0, 45, 10)),
            ],
            15.0,
        ),
        layer(2, 0.80, vec![internal(rect(60, 0, 70, 10))], 100.0),
        layer(3, 1.00, vec![internal(rect(80, 0, 90, 10))], 15.0),
    ]);

    let output = gather(&fixture, 2, 0.6).unwrap();

    assert_eq!(
        bounds(&output),
        vec![(0, 0, 10, 20), (20, 0, 30, 20), (45, 0, 50, 10)]
    );
}

#[test]
fn task22o47_unions_and_closes_sparse_geometry_before_subtracting_solids() {
    let mut hole = rect(10, 10, 20, 20).contour().clone();
    hole.reverse();
    let donut = ExPolygon::new(rect(0, 0, 30, 30).contour().clone(), vec![hole]);
    let fixture = Fixture::new(vec![
        layer(
            1,
            0.80,
            vec![
                internal(donut),
                internal(rect(20, 0, 40, 30)),
                surface(RegionSurfaceKind::Bottom, rect(30, 5, 35, 25)),
            ],
            15.0,
        ),
        layer(2, 1.00, Vec::new(), 15.0),
    ]);

    let output = gather(&fixture, 1, 0.2).unwrap();

    assert_eq!(output.len(), 3);
    assert_eq!(
        output.iter().filter(|polygon| polygon.area() > 0.0).count(),
        1
    );
    assert_eq!(
        output.iter().filter(|polygon| polygon.area() < 0.0).count(),
        2
    );
    assert_eq!(area_mm2(&output), 1_000.0);
}

#[test]
fn task22o47_scaled_epsilon_closes_exact_thresholds_at_both_coordinate_scales() {
    for (scale, merged_gap, separate_gap) in [
        (CoordinateScale::Normal, 199, 201),
        (CoordinateScale::LargeBed, 19, 21),
    ] {
        for (gap, expected_paths) in [(merged_gap, 1), (separate_gap, 2)] {
            let fixture = Fixture::new(vec![
                layer(
                    1,
                    0.8,
                    vec![
                        internal(raw_rect(0, 0, 10_000, 10_000)),
                        internal(raw_rect(10_000 + gap, 0, 20_000 + gap, 10_000)),
                    ],
                    15.0,
                ),
                layer(2, 1.0, Vec::new(), 15.0),
            ]);

            let output = gather_with_scale(&fixture, 1, 0.2, scale).unwrap();

            assert_eq!(output.len(), expected_paths, "{scale:?} gap {gap}");
        }
    }
}

#[test]
fn task22o47_empty_and_fully_subtracted_sparse_geometry_are_empty_success() {
    let empty = Fixture::new(vec![
        layer(
            1,
            0.8,
            vec![surface(RegionSurfaceKind::Top, rect(0, 0, 10, 10))],
            15.0,
        ),
        layer(2, 1.0, Vec::new(), 15.0),
    ]);
    assert!(gather(&empty, 1, 0.2).unwrap().is_empty());

    let covered = Fixture::new(vec![
        layer(
            1,
            0.8,
            vec![
                internal(rect(0, 0, 10, 10)),
                surface(RegionSurfaceKind::InternalSolid, rect(0, 0, 10, 10)),
            ],
            15.0,
        ),
        layer(2, 1.0, Vec::new(), 15.0),
    ]);
    assert!(gather(&covered, 1, 0.2).unwrap().is_empty());
}

#[test]
fn task22o47_returns_first_range_error_and_preserves_all_borrowed_input() {
    let fixture = Fixture::new(vec![
        layer(1, 0.8, vec![internal(outside_range())], 15.0),
        layer(2, 1.0, Vec::new(), 15.0),
    ]);
    let before = snapshot(&fixture);

    assert_eq!(
        gather(&fixture, 1, 0.2),
        Err(ClipperError::CoordinateOutOfRange)
    );
    assert_eq!(snapshot(&fixture), before);
}

struct Fixture {
    planned: Vec<PlannedLayer>,
    surfaces: Vec<Vec<RegionSurface>>,
    densities: Vec<f64>,
}

impl Fixture {
    fn new(layers: Vec<(PlannedLayer, Vec<RegionSurface>, f64)>) -> Self {
        let mut planned = Vec::new();
        let mut surfaces = Vec::new();
        let mut densities = Vec::new();
        for (layer, layer_surfaces, density) in layers {
            planned.push(layer);
            surfaces.push(layer_surfaces);
            densities.push(density);
        }
        Self {
            planned,
            surfaces,
            densities,
        }
    }

    fn views(&self) -> Vec<DeepSparseLayer<'_>> {
        self.planned
            .iter()
            .zip(&self.surfaces)
            .zip(&self.densities)
            .map(|((planned, fill_surfaces), density)| DeepSparseLayer {
                planned,
                fill_surfaces,
                sparse_infill_density_percent: *density,
            })
            .collect()
    }
}

fn gather(
    fixture: &Fixture,
    candidate_layer_index: usize,
    target_flow_height: f32,
) -> Result<Vec<Polygon>, ClipperError> {
    gather_with_scale(
        fixture,
        candidate_layer_index,
        target_flow_height,
        CoordinateScale::Normal,
    )
}

fn gather_with_scale(
    fixture: &Fixture,
    candidate_layer_index: usize,
    target_flow_height: f32,
    scale: CoordinateScale,
) -> Result<Vec<Polygon>, ClipperError> {
    gather_deep_sparse_infill_area(
        &fixture.views(),
        candidate_layer_index,
        target_flow_height,
        scale,
    )
}

fn layer(
    id: usize,
    print_z: f64,
    surfaces: Vec<RegionSurface>,
    density: f64,
) -> (PlannedLayer, Vec<RegionSurface>, f64) {
    (
        PlannedLayer {
            id,
            height: 0.2,
            print_z,
            slice_z: print_z - 0.1,
        },
        surfaces,
        density,
    )
}

fn surface(kind: RegionSurfaceKind, expolygon: ExPolygon) -> RegionSurface {
    RegionSurface::new(kind, expolygon)
}

fn internal(expolygon: ExPolygon) -> RegionSurface {
    RegionSurface::internal(expolygon)
}

fn rect(min_x: i64, min_y: i64, max_x: i64, max_y: i64) -> ExPolygon {
    const SCALE: i64 = 1_000_000;
    raw_rect(min_x * SCALE, min_y * SCALE, max_x * SCALE, max_y * SCALE)
}

fn raw_rect(min_x: i64, min_y: i64, max_x: i64, max_y: i64) -> ExPolygon {
    ExPolygon::new(
        Polygon::new(vec![
            Point::new(min_x, min_y),
            Point::new(max_x, min_y),
            Point::new(max_x, max_y),
            Point::new(min_x, max_y),
        ]),
        Vec::new(),
    )
}

fn bounds(polygons: &[Polygon]) -> Vec<(i64, i64, i64, i64)> {
    let mut output = polygons
        .iter()
        .map(|polygon| {
            let points = polygon.points();
            (
                points
                    .iter()
                    .map(|point| point.x() / 1_000_000)
                    .min()
                    .unwrap(),
                points
                    .iter()
                    .map(|point| point.y() / 1_000_000)
                    .min()
                    .unwrap(),
                points
                    .iter()
                    .map(|point| point.x() / 1_000_000)
                    .max()
                    .unwrap(),
                points
                    .iter()
                    .map(|point| point.y() / 1_000_000)
                    .max()
                    .unwrap(),
            )
        })
        .collect::<Vec<_>>();
    output.sort_unstable();
    output
}

fn area_mm2(polygons: &[Polygon]) -> f64 {
    polygons.iter().map(Polygon::area).sum::<f64>() / 1_000_000_000_000.0
}

fn snapshot(fixture: &Fixture) -> (Vec<PlannedLayer>, Vec<u64>, Vec<u8>) {
    let mut surfaces = Vec::new();
    for layer in &fixture.surfaces {
        for surface in layer {
            append_surface_snapshot(&mut surfaces, surface);
        }
    }
    (
        fixture.planned.clone(),
        fixture
            .densities
            .iter()
            .map(|value| value.to_bits())
            .collect(),
        surfaces,
    )
}

fn append_surface_snapshot(output: &mut Vec<u8>, surface: &RegionSurface) {
    let (kind, expolygon, thickness, layers, angle, extra) = surface.as_parts();
    output.extend_from_slice(
        format!(
            "{kind:?}:{}:{layers}:{}:{extra}|",
            thickness.to_bits(),
            angle.to_bits()
        )
        .as_bytes(),
    );
    for polygon in std::iter::once(expolygon.contour()).chain(expolygon.holes()) {
        for point in polygon.points() {
            output.extend_from_slice(format!("{},{};", point.x(), point.y()).as_bytes());
        }
        output.push(b'|');
    }
}

fn outside_range() -> ExPolygon {
    ExPolygon::new(
        Polygon::new(vec![
            Point::new(0x4000_0000_0000_0000, 0),
            Point::new(0x4000_0000_0000_0000, 10),
            Point::new(0x3fff_ffff_ffff_ffff, 10),
        ]),
        Vec::new(),
    )
}
