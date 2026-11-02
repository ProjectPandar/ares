//! `candidate_expansion` per-layer job preparation, split from
//! `candidate_expansion.rs` for the <400 LOC rule.

use std::collections::BTreeMap;

use crate::{
    OrcaFloats, SliceError,
    geometry::{CoordinateScale, Polyline},
    project_slice::{
        perimeters::{
            classic::traversal::PostClassicTraversalPrintObject,
            flow::{resolve_nominal_sparse_infill_flow, resolve_thick_solid_infill_bridge_flow},
            types::{PostPerimeterInputPrintObject, RegionLayerIndex},
        },
        prepare_infill::{
            bridge_over_infill::types::{BridgeCandidateObject, CandidateSurface},
            external_surfaces::PreparedPostExternalSurfaces,
            surface_type_detection::types::PreparedSurfaceTypeObject,
        },
    },
};

use super::super::super::{
    anchored_polygon::scaled_flow_value,
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
    sparse_anchoring::generate_sparse_infill_polylines_for_anchoring,
};
use super::super::geometry_error;
use super::TARGET_FLOW_HEIGHT_FACTOR;

pub(super) fn prepare_infill_lines(
    predecessor: &PreparedPostExternalSurfaces,
    object_index: usize,
    candidates: &BridgeCandidateObject,
    view: super::ObjectView<'_>,
    lightning_generator: Option<&crate::fill::lightning::generator::Generator>,
) -> Result<BTreeMap<usize, Vec<Polyline>>, SliceError> {
    let super::ObjectView {
        horizontal,
        prelude,
        nozzles,
        scale,
        ..
    } = view;
    let resolved = predecessor
        .predecessor
        .predecessor
        .resolved
        .objects
        .iter()
        .find(|object| object.source_object_index == prelude.identity().0)
        .expect("bridge transaction retains its resolved object");
    let object_options = &resolved.object;
    let (plan, _, _) = prelude.as_parts().0.as_parts().0.as_parts();
    // Fill.cpp:1398/1443 gives the anchoring filler the same PrintObject box.
    let stars_bounds = resolved
        .all_regions
        .iter()
        .any(|region| {
            matches!(
                region.sparse_infill_pattern,
                crate::ProcessInfillPattern::TriHexagon
                    | crate::ProcessInfillPattern::Grid
                    | crate::ProcessInfillPattern::Triangles
            ) && region.fill_multiline.0 > 1
        })
        .then(|| {
            crate::project_slice::fill_entities::extended_stars_bounds(
                &predecessor.predecessor.predecessor.project.objects()[plan.source_object_index],
                resolved.print_objects[plan.transform_index].transform,
                scale,
            )
        });
    let (_, inputs) = prelude.as_parts();
    let region_count = prelude.as_parts().0.as_parts().0.as_parts().2.len();
    let mut result = BTreeMap::new();
    for &candidate_layer in candidates.surfaces_by_layer.keys() {
        let lower_layer = candidate_layer
            .checked_sub(1)
            .expect("bridge candidate always has a lower layer");
        for region_index in 0..region_count {
            let slot = prelude.region_layer_record_index(RegionLayerIndex {
                region_index,
                layer_index: lower_layer,
            });
            let input = match (&inputs[slot], &horizontal.records[slot]) {
                (Some(input), Some(_)) => input,
                (None, None) => continue,
                _ => unreachable!("bridge anchor records remain aligned"),
            };
            let region = prelude.region_options(input);
            if region.sparse_infill_density.0 == 0.0 {
                continue;
            }
            let nominal_flow = resolve_nominal_sparse_infill_flow(region, object_options, nozzles)?;
            super::super::anchor_projection::validate(region, nominal_flow.spacing, scale)?;
        }
        let lines = generate_sparse_infill_polylines_for_anchoring(
            predecessor,
            object_index,
            lower_layer,
            lightning_generator,
            stars_bounds,
        )?;
        result.insert(lower_layer, lines);
    }
    Ok(result)
}

pub(super) struct LayerJob<'a> {
    pub(super) candidates: &'a mut BridgeCandidateObject,
    pub(super) cluster: &'a [usize],
    pub(super) job_index: usize,
    pub(super) layer_index: usize,
    pub(super) horizontal: &'a PreparedSurfaceTypeObject,
    pub(super) prelude: &'a PostPerimeterInputPrintObject,
    pub(super) planned_layers: &'a [crate::project_slice::layers::PlannedLayer],
    pub(super) deep_layers: &'a [DeepSparseLayer<'a>],
    pub(super) infill_lines: &'a BTreeMap<usize, Vec<Polyline>>,
    pub(super) nozzles: &'a OrcaFloats,
    pub(super) scale: CoordinateScale,
    pub(super) object_options: &'a crate::ObjectOptions,
    pub(super) default_region: &'a crate::ProcessRegionSourceOptions,
}

pub(super) fn prepare_layer(job: LayerJob<'_>) -> Result<(), SliceError> {
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
        object_options,
        default_region,
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
    let completed = prepare_candidates(
        ordered,
        context,
        super::ObjectView {
            horizontal,
            prelude,
            nozzles,
            scale,
            object_options,
            default_region,
        },
    )?;
    let current = candidates.surfaces_by_layer.entry(layer_index).or_default();
    replace_candidate_layer(current, completed);
    Ok(())
}

pub(super) fn prepare_candidates(
    ordered: Vec<CandidateSurface>,
    context: super::super::super::current_layer_context::CurrentLayerBridgeContext,
    view: super::ObjectView<'_>,
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
        let input = inputs[prelude.region_layer_record_index(RegionLayerIndex {
            region_index: candidate.source.region_index,
            layer_index: candidate.source.layer_index,
        })]
        .as_ref()
        .expect("candidate retains its source input");
        let region = prelude.region_options(input);
        let flow = resolve_thick_solid_infill_bridge_flow(region, nozzles)?;
        let scaled_spacing = scaled_flow_value(flow.spacing, scale);
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
