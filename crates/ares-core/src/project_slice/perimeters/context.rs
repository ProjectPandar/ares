use crate::{ProcessPerimeterGenerator, project::effective_config::types::ResolvedProjectObject};

use super::{
    super::compensation::PostCompensationPrintObject,
    types::{
        PerimeterDispatch, PerimeterInputRecord, PostPerimeterInputPrintObject,
        PreparedObjectFlows, RegionLayerIndex,
    },
};

const EPSILON: f64 = 1e-4;

pub(in crate::project_slice) fn prepare_perimeter_contexts(
    objects: Vec<PostCompensationPrintObject>,
    flows: Vec<PreparedObjectFlows>,
    resolved_objects: &[ResolvedProjectObject],
    spiral_mode: bool,
) -> Vec<PostPerimeterInputPrintObject> {
    let contexts = resolved_objects
        .iter()
        .flat_map(|resolved| {
            resolved
                .print_objects
                .iter()
                .enumerate()
                .map(move |(transform_index, _)| (resolved, transform_index))
        })
        .collect::<Vec<_>>();
    assert_eq!(objects.len(), flows.len());
    assert_eq!(objects.len(), contexts.len());

    objects
        .into_iter()
        .zip(flows)
        .zip(contexts)
        .map(|((object, flows), (resolved, transform_index))| {
            let records =
                prepare_object_contexts(&object, flows, resolved, transform_index, spiral_mode);
            PostPerimeterInputPrintObject { object, records }
        })
        .collect()
}

fn prepare_object_contexts(
    object: &PostCompensationPrintObject,
    flows: PreparedObjectFlows,
    resolved: &ResolvedProjectObject,
    transform_index: usize,
    global_spiral_mode: bool,
) -> Vec<Option<PerimeterInputRecord>> {
    let (post_region, _) = object.as_parts();
    let (plan, _, regions) = post_region.as_parts();
    assert_eq!(plan.source_object_index, resolved.source_object_index);
    assert_eq!(plan.transform_index, transform_index);
    let [region] = regions else {
        unreachable!("perimeter preflight already validated the single-region boundary")
    };
    let (region_id, region_options, layers) = region.as_parts();
    assert_eq!(plan.layers.len(), layers.len());
    assert_eq!(plan.layers.len(), flows.layers.len());

    let model_rotation_rad = if region_options.align_infill_direction_to_model.0 {
        let (m00, m10) = resolved.print_objects[transform_index]
            .transform
            .first_xy_column();
        m10.atan2(m00)
    } else {
        0.0
    };
    let bottom_shell_layers = usize::try_from(region_options.bottom_shell_layers.0)
        .expect("earlier slicing stages validate bottom_shell_layers");

    plan.layers
        .iter()
        .zip(layers)
        .zip(flows.layers)
        .enumerate()
        .map(|(layer_index, ((layer, region_layer), flows))| {
            assert_eq!(flows.is_some(), !region_layer.surfaces().is_empty());
            flows.map(|flows| {
                let upper_layer_index =
                    (layer_index + 1 < plan.layers.len()).then_some(layer_index + 1);
                let spiral_mode = global_spiral_mode
                    && layer.id >= bottom_shell_layers
                    && layer.print_z >= region_options.bottom_shell_thickness.0 - EPSILON;
                let dispatch = match (resolved.object.wall_generator, spiral_mode) {
                    (ProcessPerimeterGenerator::Arachne, false) => PerimeterDispatch::Arachne,
                    (ProcessPerimeterGenerator::Arachne, true)
                    | (ProcessPerimeterGenerator::Classic, _) => PerimeterDispatch::Classic,
                };
                PerimeterInputRecord {
                    source_object_index: plan.source_object_index,
                    transform_index: plan.transform_index,
                    planned_layer_index: layer_index,
                    layer_id: layer.id,
                    region_id,
                    compatible_region_ids: [region_id],
                    current: RegionLayerIndex {
                        region_index: 0,
                        layer_index,
                    },
                    lower_layer_index: layer_index.checked_sub(1),
                    upper_layer_index,
                    upper_same_region: upper_layer_index.map(|layer_index| RegionLayerIndex {
                        region_index: 0,
                        layer_index,
                    }),
                    layer_height: layer.height,
                    slice_z: layer.slice_z,
                    perimeter_flow: flows.perimeter_flow,
                    ext_perimeter_flow: flows.ext_perimeter_flow,
                    overhang_flow: flows.overhang_flow,
                    solid_infill_flow: flows.solid_infill_flow,
                    spiral_mode,
                    model_rotation_rad,
                    dispatch,
                }
            })
        })
        .collect()
}
