//! `candidate_expansion` per-layer job preparation, split from
//! `candidate_expansion.rs` for the <400 LOC rule.

use std::collections::BTreeMap;

use crate::{
    CancelToken, ProcessInfillPattern, SliceError,
    geometry::Polyline,
    parallel::{par_map_try, par_range_map_ordered_try},
    project_slice::{
        perimeters::{
            classic::traversal::PostClassicTraversalPrintObject,
            flow::resolve_thick_solid_infill_bridge_flow,
            types::{PostPerimeterInputPrintObject, RegionLayerIndex},
        },
        prepare_infill::{
            bridge_over_infill::types::{BridgeCandidateObject, CandidateSurface},
            external_surfaces::PreparedPostExternalSurfaces,
        },
    },
};

use super::super::super::{
    anchored_polygon::{BridgeSampling, scaled_flow_value},
    candidate_anchored_bridge::construct_candidate_anchored_bridge,
    candidate_boundary_polylines::prepare_candidate_boundary_polylines,
    candidate_bridge_angle::{CandidateBridgeAngleInput, determine_candidate_bridge_angle},
    candidate_bridge_area::{CandidateBridgeAreaInput, prepare_candidate_bridge_area},
    candidate_bridge_commit::{append_postprocessed_candidate, replace_candidate_layer},
    candidate_bridge_postprocessing::postprocess_candidate_bridge,
    candidate_collision_reconstruction::reconstruct_candidate_bridge_collision,
    candidate_ordering::order_candidate_surfaces,
    current_layer_context::{CurrentLayerBridgeRegion, prepare_current_layer_bridge_context},
    deep_sparse_area::{DeepSparseLayer, gather_deep_sparse_infill_area},
    lower_cluster_subtraction::{ClusterBridgeHistoryLayer, subtract_filled_lower_cluster_bridges},
    sparse_anchoring::AnchoringContext,
};
use super::super::geometry_error;
use super::TARGET_FLOW_HEIGHT_FACTOR;

mod anchor;
use anchor::prepare_anchor_layer;

pub(super) fn prepare_infill_lines(
    predecessor: &PreparedPostExternalSurfaces,
    object_index: usize,
    candidates: &BridgeCandidateObject,
    view: super::ObjectView<'_>,
    (uses_random_template, cancel): (bool, &CancelToken),
) -> Result<BTreeMap<usize, Vec<Polyline>>, SliceError> {
    cancel.check()?;
    let resolved = predecessor
        .predecessor
        .predecessor
        .print_object_config(object_index)
        .0;
    let (plan, _, _) = view.prelude.as_parts().0.as_parts().0.as_parts();
    // Fill.cpp:1398/1443 gives the anchoring filler the same PrintObject box.
    let grid_bounds = resolved
        .all_regions
        .iter()
        .any(|region| {
            region.sparse_infill_pattern == crate::ProcessInfillPattern::Grid
                && region.fill_multiline.0 > 1
        })
        .then(|| {
            crate::project_slice::fill_entities::extended_object_bounds(
                &predecessor.predecessor.predecessor.project.objects()[plan.source_object_index],
                resolved.print_objects[plan.transform_index].transform,
                view.scale,
            )
        });
    let adaptive_fields = &candidates.adaptive_infill_fields;
    let prepare = |candidate_layer: usize, lightning_generator| {
        cancel.check()?;
        let lower_layer = candidate_layer
            .checked_sub(1)
            .expect("bridge candidate always has a lower layer");
        let lines = prepare_anchor_layer(
            predecessor,
            object_index,
            lower_layer,
            view,
            AnchoringContext {
                lightning_generator,
                adaptive_fields,
                grid_bounds,
            },
        )?;
        Ok((lower_layer, lines))
    };
    if uses_random_template || candidates.lightning_generator.is_some() {
        return candidates
            .surfaces_by_layer
            .keys()
            .map(|&key| prepare(key, candidates.lightning_generator.as_deref()))
            .collect();
    }
    let layers = candidates
        .surfaces_by_layer
        .keys()
        .copied()
        .collect::<Vec<_>>();
    // PrintObject.cpp:3374–3385: each candidate's lower layer owns its anchor lines.
    let lines =
        par_range_map_ordered_try(layers.len(), cancel, |index| prepare(layers[index], None))?;
    Ok(lines.into_iter().collect())
}

#[derive(Clone, Copy)]
pub(super) struct ClusterContext<'a> {
    pub(super) view: super::ObjectView<'a>,
    pub(super) planned_layers: &'a [crate::project_slice::layers::PlannedLayer],
    pub(super) deep_layers: &'a [DeepSparseLayer<'a>],
    pub(super) infill_lines: &'a BTreeMap<usize, Vec<Polyline>>,
    pub(super) cancel: &'a CancelToken,
}

type Cluster = Vec<(usize, Vec<CandidateSurface>)>;

pub(super) fn prepare_clusters(
    candidates: &mut BTreeMap<usize, Vec<CandidateSurface>>,
    clusters: Vec<Vec<usize>>,
    context: ClusterContext<'_>,
    uses_random_template: bool,
) -> Result<(), SliceError> {
    let jobs = clusters
        .into_iter()
        .map(|layers| {
            context.cancel.check()?;
            Ok(layers
                .into_iter()
                .map(|index| {
                    (
                        index,
                        candidates
                            .remove(&index)
                            .expect("cluster layer has candidates"),
                    )
                })
                .collect::<Cluster>())
        })
        .collect::<Result<Vec<_>, SliceError>>()?;
    // PrintObject.cpp:3747–4032: layers depend only on earlier layers in their cluster.
    let completed = if uses_random_template {
        jobs.into_iter()
            .map(|job| prepare_cluster(job, context))
            .collect::<Result<Vec<_>, _>>()?
    } else {
        let results = par_map_try(jobs, context.cancel, |job| {
            Ok(prepare_cluster(job, context))
        })?;
        context.cancel.check()?;
        results.into_iter().collect::<Result<Vec<_>, _>>()?
    };
    context.cancel.check()?;
    candidates.extend(completed.into_iter().flatten());
    Ok(())
}

fn prepare_cluster(
    mut layers: Cluster,
    context: ClusterContext<'_>,
) -> Result<Cluster, SliceError> {
    context.cancel.check()?;
    for index in 0..layers.len() {
        context.cancel.check()?;
        let (previous, current) = layers.split_at_mut(index);
        let (layer_index, current) = &mut current[0];
        let result = prepare_layer(*layer_index, std::mem::take(current), previous, context)?;
        replace_candidate_layer(current, result);
    }
    Ok(layers)
}

fn prepare_layer(
    layer_index: usize,
    raw: Vec<CandidateSurface>,
    previous: &[(usize, Vec<CandidateSurface>)],
    ClusterContext {
        view,
        planned_layers,
        deep_layers,
        infill_lines,
        cancel,
    }: ClusterContext<'_>,
) -> Result<Vec<CandidateSurface>, SliceError> {
    let super::ObjectView {
        horizontal,
        prelude,
        nozzles,
        scale,
        ..
    } = view;
    let ordered = order_candidate_surfaces(raw);
    let first = ordered
        .first()
        .expect("raw candidate map entries are nonempty");
    let (_, inputs) = prelude.as_parts();
    let first_input = inputs[prelude.region_layer_record_index(RegionLayerIndex {
        region_index: first.source.region_index,
        layer_index: first.source.layer_index,
    })]
    .as_ref()
    .expect("candidate retains its source input");
    let first_region = prelude.region_options(first_input);
    let first_flow = resolve_thick_solid_infill_bridge_flow(first_region, nozzles)?;
    let target_flow_height = first_flow.height * TARGET_FLOW_HEIGHT_FACTOR;
    let deep = gather_deep_sparse_infill_area(deep_layers, layer_index, target_flow_height, scale)
        .map_err(geometry_error)?;
    let history = previous
        .iter()
        .map(|(index, candidates)| ClusterBridgeHistoryLayer {
            print_z: planned_layers[*index].print_z,
            candidates,
        })
        .collect::<Vec<_>>();
    let deep = subtract_filled_lower_cluster_bridges(
        &deep,
        &history,
        planned_layers[layer_index].print_z,
        f64::from(target_flow_height),
    )
    .map_err(geometry_error)?;
    let regions = prelude
        .as_parts()
        .0
        .as_parts()
        .0
        .as_parts()
        .2
        .iter()
        .enumerate()
        .map(|(region_index, region)| {
            let slot = prelude.region_layer_record_index(RegionLayerIndex {
                region_index,
                layer_index,
            });
            let (fill_surfaces, fill_expolygons) =
                horizontal.records[slot]
                    .as_ref()
                    .map_or((&[][..], &[][..]), |record| {
                        (
                            record.fill_surfaces.as_slice(),
                            record.fill_expolygons.as_slice(),
                        )
                    });
            CurrentLayerBridgeRegion {
                fill_surfaces,
                fill_expolygons,
                sparse_infill_pattern: region.as_parts().1.sparse_infill_pattern,
            }
        })
        .collect::<Vec<_>>();
    let context = prepare_current_layer_bridge_context(
        &deep,
        &regions,
        &infill_lines[&(layer_index - 1)],
        scaled_flow_value(first_flow.spacing, scale),
        scale,
    )
    .map_err(geometry_error)?;
    prepare_candidates(ordered, context, view, cancel)
}

fn prepare_candidates(
    ordered: Vec<CandidateSurface>,
    context: super::super::super::current_layer_context::CurrentLayerBridgeContext,
    view: super::ObjectView<'_>,
    cancel: &CancelToken,
) -> Result<Vec<CandidateSurface>, SliceError> {
    let super::ObjectView {
        prelude,
        nozzles,
        scale,
        ..
    } = view;
    let super::super::super::current_layer_context::CurrentLayerBridgeContext {
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
        cancel.check()?;
        let input = inputs[prelude.region_layer_record_index(RegionLayerIndex {
            region_index: candidate.source.region_index,
            layer_index: candidate.source.layer_index,
        })]
        .as_ref()
        .expect("candidate retains its source input");
        let region = prelude.region_options(input);
        let flow = resolve_thick_solid_infill_bridge_flow(region, nozzles)?;
        let turning = matches!(
            region.sparse_infill_pattern,
            ProcessInfillPattern::HilbertCurve | ProcessInfillPattern::OctagramSpiral
        );
        let sampling = BridgeSampling::new(flow, scale, turning);
        let scaled_spacing = sampling.spacing;
        let area = prepare_candidate_bridge_area(CandidateBridgeAreaInput {
            candidate_polygons: &candidate.new_polygons,
            deep_infill_area: &deep_infill_area,
            internal_unsupported_area: &internal_unsupported_area,
            expansion_area: &expansion_area,
            scaled_spacing,
            scale,
        })
        .map_err(geometry_error)?;
        let Some(boundaries) =
            prepare_candidate_boundary_polylines(&area, &total_fill_area, scaled_spacing)
                .map_err(geometry_error)?
        else {
            continue;
        };
        let configured_angle = super::lower_direction::configured_angle(
            view,
            &candidate,
            &area.area_to_be_bridge,
            !anchors.is_empty(),
        )
        .map_err(geometry_error)?;
        let angle = determine_candidate_bridge_angle(CandidateBridgeAngleInput {
            area_to_be_bridge: &area.area_to_be_bridge,
            anchors: &anchors,
            boundary_polylines: &boundaries,
            region,
            model_rotation_rad: input.model_rotation_rad,
            configured_angle,
            scale,
        });
        let initial = construct_candidate_anchored_bridge(
            &area.area_to_be_bridge,
            boundaries,
            &anchors,
            &lightning_area,
            (sampling, angle, scale),
        )
        .map_err(geometry_error)?;
        let collision = reconstruct_candidate_bridge_collision(
            &area.area_to_be_bridge,
            initial,
            (sampling, angle),
            &completed,
        )
        .map_err(geometry_error)?;
        let postprocessed = postprocess_candidate_bridge(
            collision,
            expansion_area,
            (&area.limiting_area, &total_fill_area, &total_top_area),
            (sampling, scale),
        )
        .map_err(geometry_error)?;
        expansion_area =
            append_postprocessed_candidate(&mut completed, candidate.source, postprocessed);
    }
    Ok(completed)
}

pub(super) fn prelude(
    traversal: &PostClassicTraversalPrintObject,
) -> &PostPerimeterInputPrintObject {
    &traversal
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .object
}
