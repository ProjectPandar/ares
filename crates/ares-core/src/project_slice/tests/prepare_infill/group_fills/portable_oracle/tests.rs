use crate::geometry::{ExPolygon, Point, Polygon};

use super::*;

#[test]
fn frozen_post_encoder_preserves_source_idx_metadata_table_and_geometry_grammar() {
    let fills = [ExPolygon::new(
        Polygon::new(vec![Point::new(1, 2), Point::new(3, 4), Point::new(5, 6)]),
        vec![Polygon::new(vec![
            Point::new(7, 8),
            Point::new(9, 10),
            Point::new(11, 12),
        ])],
    )];
    let group = OracleGroup {
        region_id: 0,
        representative: OracleRepresentative {
            kind: 6,
            thickness: 0.0,
            thickness_layers: 1,
            bridge_angle: 0.0,
            extra_perimeters: 0,
        },
        params: zero_params(),
        region_id_group: &[0],
        fills: &fills,
        no_overlap: &[],
    };
    let encoded = encode(&[OracleLayer {
        stage: OracleStage::PostNarrow,
        layer_id: 7,
        layer_height: 0.0,
        print_z: 0.0,
        lock_counts: OracleLockCounts::default(),
        groups: vec![group],
    }]);

    assert_eq!(
        String::from_utf8(encoded.canonical_geometry).unwrap(),
        concat!(
            "7|0|fills|C|3 1,2 3,4 5,6\n",
            "7|0|fills|H|3 1,2 3,4 5,6|3 7,8 9,10 11,12\n"
        )
    );
    assert_eq!(
        String::from_utf8(encoded.layer_table).unwrap(),
        concat!(
            "layer\tgroups\texpolygons\tholes\tpoints\tkinds\tpatterns\troles\n",
            "7\t1\t1\t1\t6\t6\t0\t10\n"
        )
    );
    assert_eq!(
        String::from_utf8(encoded.metadata).unwrap(),
        concat!(
            "layer 7 stage post-narrow height_bits 0 print_z_bits 0 groups 1\n",
            "lock_params skin_density 0 skeleton_density 0 skin_flow 0 skeleton_flow 0\n",
            "group 0 region_id 0 surface_type 6 surface_thickness_bits 0 surface_thickness_layers 1 surface_bridge_angle_bits 0 surface_extra_perimeters 0\n",
            "params extruder 1 pattern 0 spacing_bits 0 overlap_bits 0 angle_bits 0 fixed_angle 0 bridge 0 bridge_angle_bits 0 density_bits 0 multiline 1 anchor_length_bits 0 anchor_length_max_bits 0\n",
            "flow width_bits 0 height_bits 0 spacing_bits 0 nozzle_bits 0 bridge 0 extrusion_role 10 idx 7 role_speed_bits 0\n",
            "extras lateral_1_bits 0 lateral_2_bits 0 infill_lock_depth_bits 0 skin_infill_depth_bits 0 symmetric_y 0 overhang_angle_bits 0 gyroid_optimized 0\n",
            "region_id_group 1 0\n",
            "surface expolygons 1\n",
            "end_surface\n",
            "fills expolygons 1\n",
            "end_fills\n",
            "no_overlap expolygons 0\n",
            "end_no_overlap\n",
            "end_group 0\n",
            "end_layer 7\n"
        )
    );
}

#[test]
fn totals_count_only_authoritative_fill_geometry() {
    let fills = [ExPolygon::new(
        Polygon::new(vec![Point::new(0, 0), Point::new(1, 0), Point::new(0, 1)]),
        Vec::new(),
    )];
    let no_overlap = [ExPolygon::new(Polygon::new(Vec::new()), Vec::new())];
    let layers = [OracleLayer {
        stage: OracleStage::PostNarrow,
        layer_id: 0,
        layer_height: 0.2,
        print_z: 0.2,
        lock_counts: OracleLockCounts::default(),
        groups: vec![OracleGroup {
            region_id: 0,
            representative: OracleRepresentative {
                kind: 4,
                thickness: -1.0,
                thickness_layers: 1,
                bridge_angle: -1.0,
                extra_perimeters: 0,
            },
            params: zero_params(),
            region_id_group: &[0],
            fills: &fills,
            no_overlap: &no_overlap,
        }],
    }];
    assert_eq!(
        totals(&layers),
        OracleTotals {
            layers: 1,
            groups: 1,
            fill_expolygons: 1,
            fill_holes: 0,
            fill_paths: 1,
            fill_points: 3,
            no_overlap_expolygons: 1,
            nonempty_layers: 1,
            empty_layers: 0,
        }
    );
}

fn zero_params() -> OracleParams {
    OracleParams {
        idx: 7,
        extruder: 1,
        pattern: 0,
        spacing: 0.0,
        overlap: 0.0,
        angle: 0.0,
        fixed_angle: false,
        bridge: false,
        bridge_angle: 0.0,
        density: 0.0,
        multiline: 1,
        anchor_length: 0.0,
        anchor_length_max: 0.0,
        flow: OracleFlow {
            width: 0.0,
            height: 0.0,
            spacing: 0.0,
            nozzle_diameter: 0.0,
            bridge: false,
        },
        extrusion_role: 10,
        role_speed: 0.0,
        lateral_lattice_angle_1: 0.0,
        lateral_lattice_angle_2: 0.0,
        infill_lock_depth: 0.0,
        skin_infill_depth: 0.0,
        symmetric_infill_y_axis: false,
        infill_overhang_angle: 0.0,
        gyroid_optimized: false,
    }
}
