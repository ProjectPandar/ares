use super::*;
use crate::fill::lightning::generator::Generator;
use crate::geometry::Point;
use crate::geometry::{CoordinateScale, ExPolygon, Polygon};
use crate::project_slice::group_fills::SurfaceFill;
use crate::project_slice::{
    group_fills::{RepresentativeSurface, SurfaceFillParams, SurfaceFillPattern},
    perimeters::types::Flow,
    region_slices::RegionSurfaceKind,
};
use crate::{ExtrusionRole, ProcessGapFillTarget, ProcessInfillPattern};

fn fill(pattern: ProcessInfillPattern) -> SurfaceFill {
    let polygon = Polygon::new(vec![
        Point::new(4_223_233, 4_223_233),
        Point::new(-4_223_228, 4_223_233),
        Point::new(-4_223_228, -4_223_228),
        Point::new(4_223_233, -4_223_228),
    ]);
    let domain = ExPolygon::new(polygon, Vec::new());
    SurfaceFill {
        region_id: 0,
        representative: RepresentativeSurface {
            kind: RegionSurfaceKind::Internal,
            thickness: 0.2,
            thickness_layers: 1,
            bridge_angle: 0.0,
            extra_perimeters: 0,
        },
        expolygons: vec![domain.clone()],
        no_overlap_expolygons: vec![domain],
        region_id_group: vec![0],
        params: SurfaceFillParams {
            extruder: 1,
            pattern: SurfaceFillPattern::Configured(pattern),
            spacing: 0.40707963705062866,
            overlap: 0.0,
            angle: 0.0,
            fixed_angle: false,
            bridge: false,
            bridge_angle: 0.0,
            density: 15.0,
            multiline: 1,
            adaptive_line_spacing: 0.0,
            anchor_length: 1.6283185,
            anchor_length_max: 20.0,
            flow: Flow {
                width: 0.45,
                height: 0.2,
                spacing: 0.40707964,
                nozzle_diameter: 0.4,
                bridge: false,
                mm3_per_mm: 0.08141593,
            },
            flow_ratio: 1.0,
            extrusion_role: ExtrusionRole::InternalInfill,
            idx: 0,
            loop_clipping: 0,
            role_speed: 0.0,
            lateral_lattice_angle_1: 0.0,
            lateral_lattice_angle_2: 0.0,
            infill_lock_depth: 0.0,
            skin_infill_depth: 0.0,
            horiz_move: 0.0,
            symmetric_infill_y_axis: false,
            infill_overhang_angle: 0.0,
            gyroid_optimized: false,
            filter_out_gap_fill: 0.0,
            gap_fill_target: ProcessGapFillTarget::Nowhere,
        },
    }
}

#[test]
fn tpmsfk_sparse_dispatch_clips_to_the_half_spacing_inset() {
    let mut fill = fill(ProcessInfillPattern::TpmsFk);
    // Cancel TPMS-FK's -45-degree correction so this tests exact clipping
    // bounds, without integer quantization from a rotate/unrotate roundtrip.
    fill.params.angle = std::f32::consts::FRAC_PI_4;
    let input = std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/parity/replay/ender3-classic.3mf"),
    )
    .unwrap();
    let traversal =
        crate::project_slice::perimeters::prepare_post_classic_traversal(&input).unwrap();
    let mut result = Vec::new();
    dispatch_tail(
        fill.params.pattern,
        fill,
        &traversal,
        42,
        8.6,
        Point::new(0, 0),
        None,
        None,
        &mut result,
    )
    .unwrap();
    assert!(!result.is_empty());
    for point in result.iter().flat_map(|line| line.points()) {
        assert!((-4_019_688..=4_019_693).contains(&point.x()), "{point:?}");
        assert!((-4_019_688..=4_019_693).contains(&point.y()), "{point:?}");
    }
}

#[test]
fn lightning_sparse_dispatch_connects_clipped_boundary_endpoints() {
    let fill = fill(ProcessInfillPattern::Lightning);
    let generator = Generator::new(
        vec![vec![fill.expolygons[0].contour().clone()]; 43],
        crate::fill::lightning::generator::InfillSettings {
            layer_height: 0.2,
            infill_line_width: 0.45,
            line_width: 0.45,
            nozzle_diameter: 0.4,
            multiline: 1,
            density_percent: 15.0,
            overhang_angle_deg: 45.0,
            prune_angle_deg: 45.0,
            straightening_angle_deg: 45.0,
            scale: CoordinateScale::Normal,
        },
    )
    .unwrap();
    let input = std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/parity/replay/ender3-classic.3mf"),
    )
    .unwrap();
    let traversal =
        crate::project_slice::perimeters::prepare_post_classic_traversal(&input).unwrap();
    let mut result = Vec::new();
    dispatch_tail(
        fill.params.pattern,
        fill,
        &traversal,
        5,
        1.2,
        Point::new(0, 0),
        Some(&generator),
        None,
        &mut result,
    )
    .unwrap();
    assert!(!result.is_empty());
    assert_eq!(
        result
            .iter()
            .flat_map(|line| line.points())
            .map(|p| p.x())
            .max(),
        Some(4_019_693)
    );
    assert!(
        result.iter().any(|line| line.points().len() >= 3),
        "sparse anchoring must include the clipped contour anchor"
    );
}
