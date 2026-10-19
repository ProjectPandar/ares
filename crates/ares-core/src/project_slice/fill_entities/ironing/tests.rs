use super::areas::enabled;
use super::*;
use crate::ProcessIroningType;
use crate::project_slice::region_slices::RegionSurface;
use crate::project_slice::region_slices::RegionSurfaceKind;
use crate::{OrcaFloat, OrcaInt, geometry::Polygon};

fn emit(domain: ExPolygon, spacing: f64) -> LayerFillEntities {
    let settings = crate::ProjectSettings::default();
    let mut region = RegionOptions::from_base(&settings.process.region);
    region.ironing_type = ProcessIroningType::Top;
    region.ironing_pattern = ProcessInfillPattern::Concentric;
    region.top_shell_layers = OrcaInt(3);
    region.top_surface_filament_id = OrcaInt(1);
    region.filament_ironing_inset = OrcaFloat(0.0);
    region.filament_ironing_flow = crate::Percent(15.0);
    region.filament_ironing_spacing = OrcaFloat(spacing);
    let object = ObjectOptions::from_base(&settings.process.object);
    let record = PreparedSurfaceTypeRecord {
        perimeters: Vec::new(),
        thin_fills: Vec::new(),
        slices: vec![RegionSurface::new(RegionSurfaceKind::Top, domain.clone())],
        fill_surfaces: Vec::new(),
        fill_expolygons: Vec::new(),
        fill_no_overlap_expolygons: Vec::new(),
    };
    let layer = PlannedLayer {
        id: 49,
        height: 0.2,
        print_z: 10.0,
        slice_z: 9.9,
    };
    let nozzles = OrcaFloats(vec![OrcaFloat(0.4)]);
    let mut output = LayerFillEntities::default();
    append(
        &mut output,
        Input {
            record: &record,
            default_region: &settings.process.region,
            layer_slices: std::slice::from_ref(&domain),
            region: &region,
            dont_alternate_fill_direction: false,
            object: &object,
            nozzles: &nozzles,
            layer: &layer,
            layers: std::slice::from_ref(&layer),
            last_layer: true,
            spiral_mode: false,
            model_rotation_offset: 0.0,
            scale: CoordinateScale::Normal,
        },
    )
    .unwrap();
    output
}

fn domain(points: &[(i64, i64)]) -> ExPolygon {
    ExPolygon::new(
        Polygon::new(points.iter().map(|&(x, y)| Point::new(x, y)).collect()),
        Vec::new(),
    )
}

fn bounds(entity: &FillExtrusionEntity) -> (i64, i64) {
    let FillExtrusionEntity::Path(path) = entity else {
        panic!("ironing must emit fixed-flow paths");
    };
    assert_eq!(path.role, ExtrusionRole::Ironing);
    (
        path.polyline.points().iter().map(|p| p.x()).min().unwrap(),
        path.polyline.points().iter().map(|p| p.x()).max().unwrap(),
    )
}

#[test]
fn concentric_ironing_insets_first_ring_by_half_spacing() {
    for (spacing, expected_max) in [(0.1, 4_750_000), (0.2, 4_700_000)] {
        let output = emit(
            domain(&[
                (-5_000_000, -5_000_000),
                (5_000_000, -5_000_000),
                (5_000_000, 5_000_000),
                (-5_000_000, 5_000_000),
            ]),
            spacing,
        );
        assert_eq!(
            bounds(&output.collections[0].entities[0]),
            (-expected_max, expected_max)
        );
    }
}

#[test]
fn concentric_ironing_split_neck_keeps_nominal_inner_ring_pitch() {
    // Nozzle inset leaves a 0.08mm neck; the half-spacing offset then
    // splits it into two lobes, each 0.95mm wide at its outer ring.
    let output = emit(
        domain(&[
            (0, 0),
            (1_450_000, 0),
            (1_450_000, 510_000),
            (3_950_000, 510_000),
            (3_950_000, 0),
            (5_400_000, 0),
            (5_400_000, 1_500_000),
            (3_950_000, 1_500_000),
            (3_950_000, 990_000),
            (1_450_000, 990_000),
            (1_450_000, 1_500_000),
            (0, 1_500_000),
        ]),
        0.1,
    );
    let mut rings: Vec<_> = output
        .collections
        .iter()
        .flat_map(|c| &c.entities)
        .map(bounds)
        .collect();
    rings.sort_unstable();
    assert!(rings.contains(&(250_000, 1_200_000)), "{rings:?}");
    assert!(rings.contains(&(350_000, 1_100_000)), "{rings:?}");
    assert!(rings.contains(&(4_200_000, 5_150_000)), "{rings:?}");
    assert!(rings.contains(&(4_300_000, 5_050_000)), "{rings:?}");
}

#[test]
fn all_ironing_types_use_source_layer_gates() {
    let mut region = RegionOptions::from_base(&crate::ProjectSettings::default().process.region);
    region.top_shell_layers = crate::OrcaInt(3);
    region.bottom_shell_layers = crate::OrcaInt(2);
    assert!(!enabled(
        ProcessIroningType::NoIroning,
        &region,
        true,
        false
    ));
    assert!(enabled(ProcessIroningType::Solid, &region, false, false));
    assert!(enabled(ProcessIroningType::Top, &region, false, false));
    assert!(!enabled(ProcessIroningType::Topmost, &region, false, false));
    assert!(enabled(ProcessIroningType::Topmost, &region, true, false));
    region.top_shell_layers = crate::OrcaInt(0);
    assert!(!enabled(ProcessIroningType::Top, &region, false, false));
    assert!(enabled(ProcessIroningType::Top, &region, false, true));
}
