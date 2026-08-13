use std::collections::BTreeMap;

use crate::{
    FloatOrPercent, OrcaFloat, OrcaFloats, OrcaInt, ProjectSettings, SliceError,
    geometry::CoordinateScale, project_slice::layers::PlannedLayer,
};

use super::{candidate, snapshot, square, view};
use crate::project_slice::prepare_infill::bridge_over_infill::{
    layer_clustering::{cluster_candidate_layers, cluster_candidate_object},
    types::BridgeCandidateObject,
};

#[test]
fn task22o54_empty_single_and_overlapping_layers_keep_ascending_order() {
    assert!(
        cluster_candidate_layers(&[], CoordinateScale::Normal)
            .unwrap()
            .is_empty()
    );
    let first = [candidate(0, 0, vec![square(0, 0, 1_000_000)])];
    let second = [candidate(1, 0, vec![square(500_000, 0, 1_000_000)])];
    let layers = [view(7, 1.0, 0.8, &first), view(11, 1.2, 0.8, &second)];
    let expected = vec![vec![7, 11]];
    assert_eq!(
        cluster_candidate_layers(&layers, CoordinateScale::Normal).unwrap(),
        expected
    );
    assert_eq!(
        cluster_candidate_layers(&layers, CoordinateScale::Normal).unwrap(),
        expected
    );
    assert_eq!(
        cluster_candidate_layers(&layers[..1], CoordinateScale::Normal).unwrap(),
        vec![vec![7]]
    );
}

#[test]
fn task22o54_strict_z_boundary_uses_f32_factor_then_f64_epsilon() {
    let first = [candidate(0, 0, vec![square(0, 0, 10)])];
    let second = [candidate(1, 0, vec![square(0, 0, 10)])];
    let current_print_z = 2.0;
    let equality_previous_z = current_print_z - f64::from(1.0_f32 * 0.9_f32) - 1.0e-4;
    let equal = [
        view(0, equality_previous_z, 1.0, &first),
        view(1, current_print_z, 1.0, &second),
    ];
    assert_eq!(
        cluster_candidate_layers(&equal, CoordinateScale::Normal).unwrap(),
        vec![vec![0, 1]]
    );
    let below = [
        view(
            0,
            f64::from_bits(equality_previous_z.to_bits() - 1),
            1.0,
            &first,
        ),
        view(1, current_print_z, 1.0, &second),
    ];
    assert_eq!(
        cluster_candidate_layers(&below, CoordinateScale::Normal).unwrap(),
        vec![vec![0], vec![1]]
    );
    let above = [
        view(
            0,
            f64::from_bits(equality_previous_z.to_bits() + 1),
            1.0,
            &first,
        ),
        view(1, current_print_z, 1.0, &second),
    ];
    assert_eq!(
        cluster_candidate_layers(&above, CoordinateScale::Normal).unwrap(),
        vec![vec![0, 1]]
    );

    let f32_promotion_witness = [
        view(0, 1.639_900_01, 0.4, &first),
        view(1, 2.0, 0.4, &second),
    ];
    assert_eq!(
        cluster_candidate_layers(&f32_promotion_witness, CoordinateScale::Normal).unwrap(),
        vec![vec![0], vec![1]]
    );

    let epsilon_order_witness = [
        view(0, 0.248_356_788_999_999_9, 2.225_426_2, &first),
        view(1, 2.251_340_223_295_654_3, 2.225_426_2, &second),
    ];
    assert_eq!(
        cluster_candidate_layers(&epsilon_order_witness, CoordinateScale::Normal).unwrap(),
        vec![vec![0], vec![1]]
    );
}

#[test]
fn task22o54_edge_touch_is_empty_and_only_previous_tail_coverage_is_consulted() {
    let first = [candidate(0, 0, vec![square(0, 0, 10_000_000)])];
    let tail = [candidate(1, 0, vec![square(16_000_000, 0, 10_000_000)])];
    let third = [candidate(2, 0, vec![square(0, 0, 2_000_000)])];
    let layers = [
        view(0, 1.0, 1.0, &first),
        view(1, 1.1, 1.0, &tail),
        view(2, 1.2, 1.0, &third),
    ];
    assert_eq!(
        cluster_candidate_layers(&layers, CoordinateScale::Normal).unwrap(),
        vec![vec![0, 1], vec![2]]
    );

    let touching = [candidate(3, 0, vec![square(40_000_000, 0, 1)])];
    let touch_layers = [view(1, 1.0, 1.0, &tail), view(3, 1.1, 1.0, &touching)];
    assert_eq!(
        cluster_candidate_layers(&touch_layers, CoordinateScale::Normal).unwrap(),
        vec![vec![1], vec![3]]
    );
}

#[test]
fn task22o54_production_composition_selects_region_zero_o48_flow() {
    let mut region_zero =
        crate::RegionOptions::from_base(&ProjectSettings::default().process.region);
    let mut candidate_region = region_zero.clone();
    region_zero.internal_solid_filament_id = OrcaInt(1);
    region_zero.bridge_line_width = FloatOrPercent::Float(0.8);
    region_zero.bridge_flow = OrcaFloat(1.0);
    candidate_region.internal_solid_filament_id = OrcaInt(2);
    candidate_region.bridge_line_width = FloatOrPercent::Float(0.4);
    candidate_region.bridge_flow = OrcaFloat(1.0);
    let nozzles = OrcaFloats(vec![OrcaFloat(0.4), OrcaFloat(0.4)]);
    let mut surfaces_by_layer = BTreeMap::new();
    surfaces_by_layer.insert(0, vec![candidate(0, 1, vec![square(0, 0, 10)])]);
    surfaces_by_layer.insert(1, vec![candidate(1, 1, vec![square(0, 0, 10)])]);
    let object = BridgeCandidateObject {
        has_lightning_infill: false,
        surfaces_by_layer,
    };
    let planned = [planned(0, 1.0), planned(1, 1.5)];
    let before = object
        .surfaces_by_layer
        .values()
        .flat_map(|candidates| snapshot(candidates))
        .collect::<Vec<_>>();
    assert_eq!(
        cluster_candidate_object(
            &object,
            &planned,
            &[&region_zero, &candidate_region],
            &nozzles,
            CoordinateScale::Normal,
        )
        .unwrap(),
        vec![vec![0, 1]]
    );

    let mut reduced_flow = region_zero.clone();
    reduced_flow.bridge_flow = OrcaFloat(0.25);
    assert_eq!(
        cluster_candidate_object(
            &object,
            &planned,
            &[&reduced_flow, &candidate_region],
            &nozzles,
            CoordinateScale::Normal,
        )
        .unwrap(),
        vec![vec![0], vec![1]]
    );
    assert_eq!(
        object
            .surfaces_by_layer
            .values()
            .flat_map(|candidates| snapshot(candidates))
            .collect::<Vec<_>>(),
        before
    );
}

#[test]
fn task22o54_production_composition_preserves_flow_and_geometry_errors() {
    let region = crate::RegionOptions::from_base(&ProjectSettings::default().process.region);
    let valid = BridgeCandidateObject {
        has_lightning_infill: false,
        surfaces_by_layer: BTreeMap::new(),
    };
    assert!(matches!(
        cluster_candidate_object(
            &valid,
            &[],
            &[&region],
            &OrcaFloats::default(),
            CoordinateScale::Normal,
        ),
        Err(SliceError::InvalidInput(message)) if message == "invalid Orca option nozzle_diameter"
    ));
    let mut infinite_ratio = region.clone();
    infinite_ratio.bridge_flow = OrcaFloat(f64::INFINITY);
    assert!(matches!(
        cluster_candidate_object(
            &valid,
            &[],
            &[&infinite_ratio],
            &OrcaFloats(vec![OrcaFloat(0.4)]),
            CoordinateScale::Normal,
        ),
        Err(SliceError::InvalidInput(message)) if message == "invalid Orca option bridge_flow"
    ));

    let mut surfaces_by_layer = BTreeMap::new();
    surfaces_by_layer.insert(0, vec![candidate(0, 0, vec![square(1_i64 << 62, 0, 10)])]);
    let invalid = BridgeCandidateObject {
        has_lightning_infill: false,
        surfaces_by_layer,
    };
    assert!(matches!(
        cluster_candidate_object(
            &invalid,
            &[planned(0, 1.0)],
            &[&region],
            &OrcaFloats(vec![OrcaFloat(0.4)]),
            CoordinateScale::Normal,
        ),
        Err(SliceError::InvalidInput(message))
            if message == "bridge candidate-layer coverage is outside the supported Clipper range"
    ));
}

fn planned(id: usize, print_z: f64) -> PlannedLayer {
    PlannedLayer {
        id,
        height: 0.2,
        print_z,
        slice_z: print_z - 0.1,
    }
}
