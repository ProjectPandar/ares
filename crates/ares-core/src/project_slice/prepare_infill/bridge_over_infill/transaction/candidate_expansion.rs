use std::collections::BTreeMap;

use crate::{
    OrcaFloats, SliceError,
    geometry::{CoordinateScale, Polyline},
    project_slice::{
        perimeters::{
            classic::traversal::PostClassicTraversalPrintObject,
            flow::{resolve_nominal_sparse_infill_flow, resolve_thick_solid_infill_bridge_flow},
            types::PostPerimeterInputPrintObject,
        },
        prepare_infill::{
            bridge_over_infill::types::{BridgeCandidateObject, CandidateSurface},
            external_surfaces::PreparedPostExternalSurfaces,
            surface_type_detection::types::PreparedSurfaceTypeObject,
        },
    },
};

use super::super::{
    anchored_polygon::scaled_flow_value,
    candidate_anchored_bridge::construct_candidate_anchored_bridge,
    candidate_boundary_polylines::prepare_candidate_boundary_polylines,
    candidate_bridge_angle::determine_candidate_bridge_angle,
    candidate_bridge_area::prepare_candidate_bridge_area,
    candidate_bridge_commit::{append_postprocessed_candidate, replace_candidate_layer},
    candidate_bridge_postprocessing::postprocess_candidate_bridge,
    candidate_collision_reconstruction::reconstruct_candidate_bridge_collision,
    candidate_ordering::order_candidate_surfaces,
    current_layer_context::{CurrentLayerBridgeRegion, prepare_current_layer_bridge_context},
    deep_sparse_area::{DeepSparseLayer, gather_deep_sparse_infill_area},
    layer_clustering::cluster_candidate_object,
    lower_cluster_subtraction::{ClusterBridgeHistoryLayer, subtract_filled_lower_cluster_bridges},
    sparse_anchoring::generate_sparse_infill_polylines_for_anchoring,
};
use super::geometry_error;

const TARGET_FLOW_HEIGHT_FACTOR: f32 = 0.9;

pub(super) fn prepare(
    predecessor: &PreparedPostExternalSurfaces,
    candidates: &mut [BridgeCandidateObject],
) -> Result<(), SliceError> {
    let horizontal = &predecessor.predecessor;
    let traversal = &horizontal.predecessor;
    let nozzles = &traversal.resolved.views.full.project.print.nozzle_diameter;
    for (index, candidates) in candidates.iter_mut().enumerate() {
        let traversal_object = &traversal.objects[index];
        let prelude = prelude(traversal_object);
        prepare_object(
            predecessor,
            index,
            candidates,
            ObjectView {
                horizontal: &horizontal.objects[index],
                prelude,
                nozzles,
                scale: traversal.scale,
            },
        )?;
    }
    Ok(())
}

#[derive(Clone, Copy)]
struct ObjectView<'a> {
    horizontal: &'a PreparedSurfaceTypeObject,
    prelude: &'a PostPerimeterInputPrintObject,
    nozzles: &'a OrcaFloats,
    scale: CoordinateScale,
}

fn prepare_object(
    predecessor: &PreparedPostExternalSurfaces,
    object_index: usize,
    candidates: &mut BridgeCandidateObject,
    view: ObjectView<'_>,
) -> Result<(), SliceError> {
    if candidates.surfaces_by_layer.is_empty() {
        return Ok(());
    }
    let ObjectView {
        horizontal,
        prelude,
        nozzles,
        scale,
    } = view;
    let (compensated, inputs) = prelude.as_parts();
    let (post_regions, _) = compensated.as_parts();
    let (plan, _, regions) = post_regions.as_parts();
    let region_options = regions
        .iter()
        .map(|region| region.as_parts().1)
        .collect::<Vec<_>>();
    let infill_lines = prepare_infill_lines(predecessor, object_index, candidates, view)?;
    let clusters =
        cluster_candidate_object(candidates, &plan.layers, &region_options, nozzles, scale)?;
    let deep_layers = plan
        .layers
        .iter()
        .enumerate()
        .map(|(index, planned)| DeepSparseLayer {
            planned,
            fill_surfaces: horizontal.records[index]
                .as_ref()
                .map_or(&[], |record| record.fill_surfaces.as_slice()),
            sparse_infill_density_percent: inputs[index].as_ref().map_or(0.0, |input| {
                prelude.region_options(input).sparse_infill_density.0
            }),
        })
        .collect::<Vec<_>>();

    for cluster in clusters {
        for (job_index, &layer_index) in cluster.iter().enumerate() {
            prepare_layer(LayerJob {
                candidates,
                cluster: &cluster,
                job_index,
                layer_index,
                horizontal,
                prelude,
                planned_layers: &plan.layers,
                deep_layers: &deep_layers,
                infill_lines: &infill_lines,
                nozzles,
                scale,
            })?;
        }
    }
    Ok(())
}

fn prepare_infill_lines(
    predecessor: &PreparedPostExternalSurfaces,
    object_index: usize,
    candidates: &BridgeCandidateObject,
    view: ObjectView<'_>,
) -> Result<BTreeMap<usize, Vec<Polyline>>, SliceError> {
    let ObjectView {
        horizontal,
        prelude,
        nozzles,
        scale,
    } = view;
    let object_options = &predecessor
        .predecessor
        .predecessor
        .resolved
        .objects
        .iter()
        .find(|object| object.source_object_index == prelude.identity().0)
        .expect("bridge transaction retains its resolved object")
        .object;
    let (_, inputs) = prelude.as_parts();
    let mut result = BTreeMap::new();
    for &candidate_layer in candidates.surfaces_by_layer.keys() {
        let lower_layer = candidate_layer
            .checked_sub(1)
            .expect("bridge candidate always has a lower layer");
        let lines = match (&inputs[lower_layer], &horizontal.records[lower_layer]) {
            (Some(input), Some(record)) => {
                let region = prelude.region_options(input);
                if region.sparse_infill_density.0 == 0.0 {
                    Vec::new()
                } else {
                    let nominal_flow =
                        resolve_nominal_sparse_infill_flow(region, object_options, nozzles)?;
                    super::anchor_projection::validate(
                        region,
                        &record.fill_surfaces,
                        nominal_flow.spacing,
                        scale,
                    )?;
                    generate_sparse_infill_polylines_for_anchoring(
                        predecessor,
                        object_index,
                        lower_layer,
                    )?
                }
            }
            (None, None) => Vec::new(),
            _ => unreachable!("bridge anchor records remain aligned"),
        };
        result.insert(lower_layer, lines);
    }
    Ok(result)
}

struct LayerJob<'a> {
    candidates: &'a mut BridgeCandidateObject,
    cluster: &'a [usize],
    job_index: usize,
    layer_index: usize,
    horizontal: &'a PreparedSurfaceTypeObject,
    prelude: &'a PostPerimeterInputPrintObject,
    planned_layers: &'a [crate::project_slice::layers::PlannedLayer],
    deep_layers: &'a [DeepSparseLayer<'a>],
    infill_lines: &'a BTreeMap<usize, Vec<Polyline>>,
    nozzles: &'a OrcaFloats,
    scale: CoordinateScale,
}

fn prepare_layer(job: LayerJob<'_>) -> Result<(), SliceError> {
    let LayerJob {
        candidates,
        cluster,
        job_index,
        layer_index,
        horizontal,
        prelude,
        planned_layers,
        deep_layers,
        infill_lines,
        nozzles,
        scale,
    } = job;
    let raw = candidates
        .surfaces_by_layer
        .remove(&layer_index)
        .expect("cluster layer retains its candidate map entry");
    let ordered = order_candidate_surfaces(raw);
    let first = ordered
        .first()
        .expect("raw candidate map entries are nonempty");
    let (_, inputs) = prelude.as_parts();
    let first_input = inputs[first.source.layer_index]
        .as_ref()
        .expect("candidate retains its source input");
    let first_region = prelude.region_options(first_input);
    let first_flow = resolve_thick_solid_infill_bridge_flow(first_region, nozzles)?;
    let target_flow_height = first_flow.height * TARGET_FLOW_HEIGHT_FACTOR;
    let deep = gather_deep_sparse_infill_area(deep_layers, layer_index, target_flow_height, scale)
        .map_err(geometry_error)?;
    let history = cluster[..job_index]
        .iter()
        .map(|&index| ClusterBridgeHistoryLayer {
            print_z: planned_layers[index].print_z,
            candidates: candidates.surfaces_by_layer[&index].as_slice(),
        })
        .collect::<Vec<_>>();
    let deep = subtract_filled_lower_cluster_bridges(
        &deep,
        &history,
        planned_layers[layer_index].print_z,
        f64::from(target_flow_height),
    )
    .map_err(geometry_error)?;
    let record = horizontal.records[layer_index]
        .as_ref()
        .expect("candidate layer retains its fill record");
    let regions = [CurrentLayerBridgeRegion {
        fill_surfaces: &record.fill_surfaces,
        fill_expolygons: &record.fill_expolygons,
        sparse_infill_pattern: first_region.sparse_infill_pattern,
    }];
    let context = prepare_current_layer_bridge_context(
        &deep,
        &regions,
        &infill_lines[&(layer_index - 1)],
        scaled_flow_value(first_flow.spacing, scale),
        scale,
    )
    .map_err(geometry_error)?;
    let completed = prepare_candidates(ordered, context, prelude, nozzles, scale)?;
    let current = candidates.surfaces_by_layer.entry(layer_index).or_default();
    replace_candidate_layer(current, completed);
    Ok(())
}

fn prepare_candidates(
    ordered: Vec<CandidateSurface>,
    context: super::super::current_layer_context::CurrentLayerBridgeContext,
    prelude: &PostPerimeterInputPrintObject,
    nozzles: &OrcaFloats,
    scale: CoordinateScale,
) -> Result<Vec<CandidateSurface>, SliceError> {
    let super::super::current_layer_context::CurrentLayerBridgeContext {
        deep_infill_area,
        lightning_area,
        mut expansion_area,
        total_fill_area,
        total_top_area,
        anchors,
        internal_unsupported_area,
    } = context;
    let (_, inputs) = prelude.as_parts();
    let mut completed = Vec::with_capacity(ordered.len());
    for candidate in ordered {
        let input = inputs[candidate.source.layer_index]
            .as_ref()
            .expect("candidate retains its source input");
        let region = prelude.region_options(input);
        let flow = resolve_thick_solid_infill_bridge_flow(region, nozzles)?;
        let scaled_spacing = scaled_flow_value(flow.spacing, scale);
        let area = prepare_candidate_bridge_area(
            &candidate.new_polygons,
            &deep_infill_area,
            &internal_unsupported_area,
            &expansion_area,
            scaled_spacing,
        )
        .map_err(geometry_error)?;
        let Some(boundaries) = prepare_candidate_boundary_polylines(
            &area,
            &total_fill_area,
            scaled_spacing,
            flow.spacing,
        )
        .map_err(geometry_error)?
        else {
            continue;
        };
        let angle = determine_candidate_bridge_angle(
            &area.area_to_be_bridge,
            &anchors,
            &boundaries,
            region,
            input.model_rotation_rad,
            scale,
        );
        let initial = construct_candidate_anchored_bridge(
            &area.area_to_be_bridge,
            boundaries,
            &anchors,
            &lightning_area,
            flow,
            angle,
            scale,
        )
        .map_err(geometry_error)?;
        let collision = reconstruct_candidate_bridge_collision(
            &area.area_to_be_bridge,
            initial,
            flow,
            angle,
            &completed,
            scale,
        )
        .map_err(geometry_error)?;
        let postprocessed = postprocess_candidate_bridge(
            collision,
            expansion_area,
            &area.limiting_area,
            &total_fill_area,
            &total_top_area,
            flow,
            scale,
        )
        .map_err(geometry_error)?;
        expansion_area =
            append_postprocessed_candidate(&mut completed, candidate.source, postprocessed);
    }
    Ok(completed)
}

fn prelude(traversal: &PostClassicTraversalPrintObject) -> &PostPerimeterInputPrintObject {
    &traversal
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .object
}
