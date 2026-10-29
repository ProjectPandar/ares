use super::{EPSILON, LayerConfig};
use super::{groups, infill_area_threshold, layer_configs};
use crate::{ProcessInfillPattern, project_slice::perimeters::types::PerimeterFlows};

fn config(layer_id: usize, height: f64, maximum: f64) -> LayerConfig {
    LayerConfig {
        active: true,
        kind: crate::project_slice::region_slices::RegionSurfaceKind::Internal,
        pattern: ProcessInfillPattern::CrossHatch,
        maximum_height: maximum,
        area_threshold: 0.0,
        layer_id,
        layer_height: height,
        flows: PerimeterFlows {
            perimeter_flow: flow(),
            ext_perimeter_flow: flow(),
            overhang_flow: flow(),
            solid_infill_flow: flow(),
        },
    }
}

fn flow() -> crate::project_slice::perimeters::types::Flow {
    crate::project_slice::perimeters::types::Flow {
        width: 0.4,
        height: 0.2,
        spacing: 0.36,
        nozzle_diameter: 0.4,
        bridge: false,
        mm3_per_mm: 0.08,
    }
}

#[test]
fn grouping_skips_first_layer_and_caps_combined_height() {
    let configs = (0..6).map(|id| config(id, 0.2, 0.4)).collect::<Vec<_>>();

    assert_eq!(groups(&configs), [0, 0, 2, 0, 2, 1]);
}

#[test]
fn grouping_combines_layers_strictly_below_cap_plus_epsilon() {
    let configs = (0..5)
        .map(|id| config(id, 0.1, 0.4 + 2.0 * EPSILON))
        .collect::<Vec<_>>();

    assert_eq!(groups(&configs), [0, 0, 0, 0, 4]);
}

/// `LayerRegion::infill_area_threshold` (`LayerRegion.cpp:975-979`): the
/// squared `coord_t` solid-infill spacing, not `minimum_sparse_infill_area`.
#[test]
fn area_threshold_is_squared_scaled_solid_infill_spacing() {
    let threshold = infill_area_threshold(flow(), crate::geometry::CoordinateScale::Normal);
    assert_eq!(threshold, 360_000.0 * 360_000.0);
}

/// `PrintObject::combine_infill` filters with `layerms.front()`'s
/// `infill_area_threshold()` (`PrintObject.cpp:4230-4237`): each layer config
/// carries its own solid-infill-flow threshold, not the KSR
/// `minimum_sparse_infill_area` of 15mm².
#[test]
fn layer_configs_take_the_threshold_from_the_solid_infill_flow() {
    let scale = crate::geometry::CoordinateScale::Normal;
    let prepared = crate::project_slice::perimeters::prepare_post_classic_traversal(
        crate::test_fixtures::ksr_project(),
    )
    .unwrap();
    let traversal = &prepared.objects[0];
    let nozzles = crate::OrcaFloats(vec![crate::OrcaFloat(0.4)]);
    let configs = layer_configs(traversal, &nozzles, scale).unwrap();
    let inputs = &traversal
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .object
        .records;
    assert_eq!(configs.len(), inputs.len());
    for (config, input) in configs.iter().zip(inputs) {
        let flow = input.as_ref().unwrap().solid_infill_flow;
        assert_eq!(config.area_threshold, infill_area_threshold(flow, scale));
        assert!(config.area_threshold < 1.0e12);
    }
}
