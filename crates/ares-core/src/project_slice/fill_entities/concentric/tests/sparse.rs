use super::super::append_configured;
use super::*;
use crate::{ProcessGapFillTarget, ProcessInfillPattern};

fn fill(width: i64, overlap: f64) -> SurfaceFill {
    let polygon = ExPolygon::new(
        Polygon::new(vec![
            Point::new(0, 0),
            Point::new(width, 0),
            Point::new(width, width),
            Point::new(0, width),
        ]),
        Vec::new(),
    );
    SurfaceFill {
        region_id: 0,
        representative: RepresentativeSurface {
            kind: RegionSurfaceKind::Internal,
            thickness: 0.2,
            thickness_layers: 1,
            bridge_angle: 0.0,
            extra_perimeters: 0,
        },
        expolygons: vec![polygon.clone()],
        no_overlap_expolygons: vec![polygon],
        region_id_group: vec![0],
        params: SurfaceFillParams {
            extruder: 1,
            pattern: SurfaceFillPattern::Configured(ProcessInfillPattern::Concentric),
            spacing: 0.4,
            overlap,
            angle: 0.0,
            fixed_angle: false,
            bridge: false,
            bridge_angle: 0.0,
            density: 15.0,
            multiline: 1,
            adaptive_line_spacing: 0.0,
            anchor_length: 0.0,
            anchor_length_max: 0.0,
            flow: Flow {
                width: 0.4,
                height: 0.2,
                spacing: 0.357,
                nozzle_diameter: 0.4,
                bridge: false,
                mm3_per_mm: 0.08,
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
fn sparse_concentric_applies_overlap_minus_half_spacing() {
    for (overlap, low, high) in [
        (0.0, 200_000, 9_800_000),
        (0.1, 100_000, 9_900_000),
        (0.3, -100_000, 10_100_000),
    ] {
        let mut output = LayerFillEntities::default();
        append_configured(
            &mut output,
            fill(10_000_000, overlap),
            0.2,
            0.4,
            CoordinateScale::Normal,
        )
        .unwrap();
        let FillExtrusionEntity::Path(first) = &output.collections[0].entities[0] else {
            panic!("sparse concentric must emit fixed-flow paths");
        };
        assert_eq!(
            first.polyline.points().iter().map(|p| p.x()).min(),
            Some(low)
        );
        assert_eq!(
            first.polyline.points().iter().map(|p| p.x()).max(),
            Some(high)
        );
    }
}

#[test]
fn sparse_concentric_reconstructs_width_from_quantized_spacing() {
    let mut input = fill(10_000_000, 0.0);
    input.params.spacing = 0.40707963705062866;
    input.params.flow.width = 0.45;
    input.params.flow.spacing = input.params.spacing as f32;
    let mut output = LayerFillEntities::default();
    append_configured(&mut output, input, 0.2, 0.4, CoordinateScale::Normal).unwrap();
    let FillExtrusionEntity::Path(first) = &output.collections[0].entities[0] else {
        panic!("sparse concentric must emit fixed-flow paths");
    };
    assert_eq!(format!("{:.6}", first.width), "0.449999");
    assert!((first.mm3_per_mm - 0.0814158).abs() < 1e-8);
    assert_eq!(first.role, ExtrusionRole::InternalInfill);
}

#[test]
fn sparse_concentric_disappears_when_offset_erases_domain() {
    let mut output = LayerFillEntities::default();
    append_configured(
        &mut output,
        fill(300_000, 0.0),
        0.2,
        0.4,
        CoordinateScale::Normal,
    )
    .unwrap();
    assert!(output.collections.is_empty());
}
