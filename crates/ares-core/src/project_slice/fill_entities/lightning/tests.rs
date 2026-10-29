use super::*;
use crate::geometry::Point;
use crate::project_slice::{
    group_fills::{RepresentativeSurface, SurfaceFillParams, SurfaceFillPattern},
    perimeters::types::Flow,
    region_slices::RegionSurfaceKind,
};
use crate::{ExtrusionRole, ProcessGapFillTarget, ProcessInfillPattern};

#[test]
fn lightning_clips_to_fill_surface_offset_before_anchoring() {
    let polygon = Polygon::new(vec![
        Point::new(4_223_233, 4_223_233),
        Point::new(-4_223_228, 4_223_233),
        Point::new(-4_223_228, -4_223_228),
        Point::new(4_223_233, -4_223_228),
    ]);
    let scale = CoordinateScale::Normal;
    let generator = Generator::new(
        vec![vec![polygon.clone()]; 43],
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
            scale,
        },
    )
    .unwrap();
    let domain = ExPolygon::new(polygon, Vec::new());
    let fill = SurfaceFill {
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
            pattern: SurfaceFillPattern::Configured(ProcessInfillPattern::Lightning),
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
    };
    let mut output = LayerFillEntities::default();
    append(&mut output, &generator, &fill, 5, scale).unwrap();
    let entities = output
        .collections
        .iter()
        .flat_map(|collection| &collection.entities)
        .map(|entity| match entity {
            FillExtrusionEntity::Path(path) => path,
            _ => panic!("lightning must emit fixed-flow paths"),
        })
        .collect::<Vec<_>>();
    assert!(!entities.is_empty());
    let points = entities
        .iter()
        .flat_map(|path| path.polyline.points())
        .collect::<Vec<_>>();
    assert_eq!(points.iter().map(|p| p.x()).max(), Some(4_019_693));
    assert!(
        points
            .iter()
            .all(|p| p.x() >= -4_019_688 && p.y() >= -4_019_688 && p.y() <= 4_019_693)
    );
    assert!(
        entities
            .iter()
            .any(|path| path.polyline.points().len() >= 3),
        "clipped boundary endpoint must receive its contour anchor"
    );
}
