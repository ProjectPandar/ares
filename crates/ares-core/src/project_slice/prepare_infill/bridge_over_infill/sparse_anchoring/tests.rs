use super::{SparseAnchoringLayer, generate_sparse_infill_polylines_for_anchoring};
use std::fmt::Write;

use crate::{
    FloatOrPercent, ObjectOptions, OrcaFloat, OrcaFloats, OrcaInt, Percent, ProjectSettings,
    RegionOptions,
    geometry::{ClipperError, CoordinateScale, ExPolygon, Point, Polygon},
    project_slice::{
        layers::PlannedLayer,
        region_slices::{RegionSurface, RegionSurfaceKind},
    },
};
use sha2::{Digest, Sha256};

#[test]
fn task22o46_single_internal_uses_owned_deterministic_output_without_mutation() {
    let fixture = fixture(vec![RegionSurface::internal(rectangle(
        0, 0, 12_000_000, 8_000_000,
    ))]);
    let before = fixture_snapshot(&fixture);
    let first = generate(&fixture, CoordinateScale::Normal).unwrap();
    let second = generate(&fixture, CoordinateScale::Normal).unwrap();

    assert!(!first.is_empty());
    assert_eq!(first, second);
    assert_eq!(fixture_snapshot(&fixture), before);
    assert_eq!(
        digest(&first),
        "4fd3d0a9c5537e37860c3e076dd7a039a92106c86571dab1fab0c30b0e9b0907"
    );
}

#[test]
fn task22o46_nominal_object_height_drives_spacing_and_projection_changes_output() {
    let baseline = fixture(vec![RegionSurface::internal(rectangle(
        0, 0, 12_000_000, 8_000_000,
    ))]);
    let expected = generate(&baseline, CoordinateScale::Normal).unwrap();

    let mut candidate = baseline.clone();
    candidate.planned.height = 0.35;
    assert_eq!(
        generate(&candidate, CoordinateScale::Normal).unwrap(),
        expected
    );

    let mut candidates = Vec::new();
    let mut candidate = baseline.clone();
    candidate.object.layer_height = OrcaFloat(0.25);
    candidates.push(candidate);
    let mut candidate = baseline.clone();
    candidate.region.sparse_infill_line_width = FloatOrPercent::Float(0.5);
    candidates.push(candidate);
    let mut candidate = baseline.clone();
    candidate.region.sparse_infill_density = Percent(25.0);
    candidates.push(candidate);
    let mut candidate = baseline.clone();
    candidate.region.infill_direction = OrcaFloat(30.0);
    candidates.push(candidate);
    let mut candidate = baseline.clone();
    candidate.region.infill_anchor = FloatOrPercent::Percent(Percent(100.0));
    candidates.push(candidate);
    let mut candidate = baseline.clone();
    candidate.region.infill_anchor_max = FloatOrPercent::Float(0.5);
    candidates.push(candidate);
    let mut candidate = baseline.clone();
    candidate.planned.print_z = 2.8;
    candidates.push(candidate);

    for (index, candidate) in candidates.iter().enumerate() {
        assert_ne!(
            generate(candidate, CoordinateScale::Normal).unwrap(),
            expected,
            "projection candidate {index}"
        );
    }
}

#[test]
fn task22o46_equal_sparse_keys_coalesce_and_priority_precedes_internal_filtering() {
    let sparse = vec![
        RegionSurface::internal(rectangle(0, 0, 8_000_000, 8_000_000)),
        RegionSurface::internal(rectangle(10_000_000, 0, 18_000_000, 8_000_000)),
    ];
    let baseline = fixture(sparse.clone());
    let baseline_output = generate(&baseline, CoordinateScale::Normal).unwrap();

    let mut independently_filled = Vec::new();
    for surface in sparse.iter().cloned() {
        independently_filled
            .extend(generate(&fixture(vec![surface]), CoordinateScale::Normal).unwrap());
    }
    assert_ne!(baseline_output, independently_filled);

    let mut bridge = RegionSurface::new(
        RegionSurfaceKind::BottomBridge,
        rectangle(0, 0, 2_000_000, 8_000_000),
    );
    bridge.set_bridge_angle(std::f64::consts::FRAC_PI_4);
    let mut prioritized = vec![
        RegionSurface::new(
            RegionSurfaceKind::Top,
            rectangle(2_000_000, 0, 4_000_000, 8_000_000),
        ),
        RegionSurface::new(RegionSurfaceKind::InternalSolid, priority_donut()),
        bridge,
    ];
    prioritized.extend(sparse);
    let prioritized = fixture(prioritized);
    let prioritized_output = generate(&prioritized, CoordinateScale::Normal).unwrap();

    assert!(!baseline_output.is_empty());
    assert!(!prioritized_output.is_empty());
    assert_ne!(baseline_output, prioritized_output);
}

#[test]
fn task22o46_forwards_scale_and_returns_first_natural_range_error_atomically() {
    let valid = fixture(vec![RegionSurface::internal(rectangle(
        0, 0, 12_000_000, 8_000_000,
    ))]);
    assert_ne!(
        digest(&generate(&valid, CoordinateScale::Normal).unwrap()),
        digest(&generate(&valid, CoordinateScale::LargeBed).unwrap())
    );
    assert_eq!(
        digest(&generate(&valid, CoordinateScale::LargeBed).unwrap()),
        "6acf8f8f358ba166dd5c2f37f2e67f1ff640309cb77c8551920ec9692b14109e"
    );

    let invalid = fixture(vec![
        RegionSurface::internal(outside_clipper_range()),
        RegionSurface::internal(rectangle(0, 0, 4_000_000, 4_000_000)),
    ]);
    let invalid_before = fixture_snapshot(&invalid);
    assert_eq!(
        generate(&invalid, CoordinateScale::Normal),
        Err(ClipperError::CoordinateOutOfRange)
    );
    assert_eq!(fixture_snapshot(&invalid), invalid_before);

    let fill_invalid = fixture(vec![RegionSurface::internal(outside_fill_range())]);
    let fill_invalid_before = fixture_snapshot(&fill_invalid);
    assert_eq!(
        generate(&fill_invalid, CoordinateScale::Normal),
        Err(ClipperError::CoordinateOutOfRange)
    );
    assert_eq!(fixture_snapshot(&fill_invalid), fill_invalid_before);

    let later = fixture(vec![
        RegionSurface::new(
            RegionSurfaceKind::InternalSolid,
            rectangle(0, 0, 4_000_000, 4_000_000),
        ),
        RegionSurface::internal(outside_clipper_range()),
    ]);
    let later_before = fixture_snapshot(&later);
    assert_eq!(
        generate(&later, CoordinateScale::Normal),
        Err(ClipperError::CoordinateOutOfRange)
    );
    assert_eq!(fixture_snapshot(&later), later_before);
}

#[derive(Clone)]
struct Fixture {
    planned: PlannedLayer,
    surfaces: Vec<RegionSurface>,
    region: RegionOptions,
    object: ObjectOptions,
    nozzles: OrcaFloats,
}

fn fixture(surfaces: Vec<RegionSurface>) -> Fixture {
    let settings = ProjectSettings::default();
    let mut region = RegionOptions::from_base(&settings.process.region);
    region.sparse_infill_density = Percent(15.0);
    region.sparse_infill_line_width = FloatOrPercent::Float(0.45);
    region.sparse_infill_filament_id = OrcaInt(1);
    region.infill_anchor = FloatOrPercent::Percent(Percent(400.0));
    region.infill_anchor_max = FloatOrPercent::Float(20.0);
    let mut object = ObjectOptions::from_base(&settings.process.object);
    object.layer_height = OrcaFloat(0.2);
    Fixture {
        planned: PlannedLayer {
            id: 10,
            height: 0.28,
            print_z: 2.2,
            slice_z: 2.06,
        },
        surfaces,
        region,
        object,
        nozzles: OrcaFloats(vec![OrcaFloat(0.4)]),
    }
}

fn generate(
    fixture: &Fixture,
    scale: CoordinateScale,
) -> Result<Vec<crate::geometry::Polyline>, ClipperError> {
    generate_sparse_infill_polylines_for_anchoring(SparseAnchoringLayer {
        planned: &fixture.planned,
        fill_surfaces: &fixture.surfaces,
        region_options: &fixture.region,
        object_options: &fixture.object,
        nozzle_diameters: &fixture.nozzles,
        scale,
    })
}

fn rectangle(min_x: i64, min_y: i64, max_x: i64, max_y: i64) -> ExPolygon {
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

fn priority_donut() -> ExPolygon {
    ExPolygon::new(
        rectangle(4_000_000, 0, 8_000_000, 8_000_000)
            .contour()
            .clone(),
        vec![
            rectangle(5_000_000, 2_000_000, 7_000_000, 6_000_000)
                .contour()
                .clone(),
        ],
    )
}

fn fixture_snapshot(
    fixture: &Fixture,
) -> (
    PlannedLayer,
    Vec<u8>,
    RegionOptions,
    ObjectOptions,
    OrcaFloats,
) {
    (
        fixture.planned.clone(),
        surface_snapshot(&fixture.surfaces),
        fixture.region.clone(),
        fixture.object.clone(),
        fixture.nozzles.clone(),
    )
}

fn surface_snapshot(surfaces: &[RegionSurface]) -> Vec<u8> {
    let mut output = Vec::new();
    for surface in surfaces {
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
    output
}

fn digest(paths: &[crate::geometry::Polyline]) -> String {
    let mut bytes = Vec::new();
    for path in paths {
        bytes.extend_from_slice(&(path.points().len() as u64).to_le_bytes());
        for point in path.points() {
            bytes.extend_from_slice(&point.x().to_le_bytes());
            bytes.extend_from_slice(&point.y().to_le_bytes());
        }
    }
    Sha256::digest(bytes)
        .iter()
        .fold(String::with_capacity(64), |mut output, byte| {
            write!(output, "{byte:02x}").unwrap();
            output
        })
}

fn outside_clipper_range() -> ExPolygon {
    ExPolygon::new(
        Polygon::new(vec![
            Point::new(0x4000_0000_0000_0000, 0),
            Point::new(0x4000_0000_0000_0000, 10),
            Point::new(0x3fff_ffff_ffff_ffff, 10),
        ]),
        Vec::new(),
    )
}

fn outside_fill_range() -> ExPolygon {
    let min = 0x4000_0000_0000_0000;
    rectangle(min, 0, min + 1_000_000, 1_000_000)
}
