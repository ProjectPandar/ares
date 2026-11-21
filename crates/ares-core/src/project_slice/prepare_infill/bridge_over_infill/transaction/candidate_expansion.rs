use crate::{
    ObjectOptions, OrcaFloats, ProcessRegionSourceOptions, SliceError,
    geometry::CoordinateScale,
    project_slice::{
        perimeters::types::{PostPerimeterInputPrintObject, RegionLayerIndex},
        prepare_infill::{
            bridge_over_infill::types::BridgeCandidateObject,
            external_surfaces::PreparedPostExternalSurfaces,
            surface_type_detection::types::PreparedSurfaceTypeObject,
        },
    },
};

use super::super::{
    deep_sparse_area::{DeepSparseLayer, DeepSparseRegion},
    layer_clustering::cluster_candidate_object,
};
use super::geometry_error;

pub(super) const TARGET_FLOW_HEIGHT_FACTOR: f32 = 0.9;

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
        let object_options = &traversal
            .resolved
            .objects
            .iter()
            .find(|object| object.source_object_index == prelude.identity().0)
            .expect("bridge transaction retains its resolved object")
            .object;
        prepare_object(
            predecessor,
            index,
            candidates,
            ObjectView {
                horizontal: &horizontal.objects[index],
                prelude,
                nozzles,
                scale: traversal.scale,
                object_options,
                default_region: &traversal.resolved.views.full.process.region,
            },
        )?;
    }
    Ok(())
}

#[derive(Clone, Copy)]
pub(super) struct ObjectView<'a> {
    horizontal: &'a PreparedSurfaceTypeObject,
    prelude: &'a PostPerimeterInputPrintObject,
    nozzles: &'a OrcaFloats,
    scale: CoordinateScale,
    object_options: &'a ObjectOptions,
    default_region: &'a ProcessRegionSourceOptions,
}

pub(super) fn prepare_object(
    predecessor: &PreparedPostExternalSurfaces,
    object_index: usize,
    candidates: &mut BridgeCandidateObject,
    view: ObjectView<'_>,
) -> Result<(), SliceError> {
    let ObjectView {
        horizontal,
        prelude,
        nozzles,
        scale,
        ..
    } = view;
    let (compensated, _) = prelude.as_parts();
    let (post_regions, _) = compensated.as_parts();
    let (plan, _, regions) = post_regions.as_parts();
    // `PrintObject.cpp:2593` LIGHTNING INFILL SECTION — expand `stInternal` to
    // `stInternalSolid` near candidates on layers whose lower layer uses
    // lightning sparse infill; the expanded set feeds the generator. Runs
    // before the empty-candidate early return so the generator exists even
    // when there are no bridge candidates (`make_fills` needs it regardless).
    if candidates.has_lightning_infill {
        populate_lightning_expanded_surfaces(candidates, horizontal, prelude, view.scale)?;
        let object_options = &predecessor
            .predecessor
            .predecessor
            .resolved
            .objects
            .iter()
            .find(|o| o.source_object_index == prelude.identity().0)
            .expect("bridge transaction retains its resolved object")
            .object;
        candidates.lightning_generator = build_lightning_generator(
            candidates,
            horizontal,
            regions,
            prelude,
            LightningConfiguration {
                nozzles,
                object_options,
                scale: view.scale,
            },
        )?;
    }
    // PrintObject.cpp:2728-2735: original candidates, before anchor generation
    // and surface rewrite; the resulting tree also survives to make_fills.
    crate::project_slice::fill_entities::adaptive::prepare_octrees(
        crate::project_slice::fill_entities::adaptive::SupportPreparation {
            traversal: &predecessor.predecessor.predecessor,
            object_index,
            candidates,
            horizontal,
            prelude,
            spacing: &predecessor.object_spacings[object_index],
        },
    )?;
    if candidates.surfaces_by_layer.is_empty() {
        return Ok(());
    }
    let region_options = regions
        .iter()
        .map(|region| region.as_parts().1)
        .collect::<Vec<_>>();
    let lightning_generator = candidates.lightning_generator.as_deref();
    let infill_lines = prepare_infill_lines(
        predecessor,
        object_index,
        candidates,
        view,
        lightning_generator,
    )?;
    let clusters =
        cluster_candidate_object(candidates, &plan.layers, &region_options, nozzles, scale)?;
    let deep_layers = plan
        .layers
        .iter()
        .enumerate()
        .map(|(index, planned)| DeepSparseLayer {
            planned,
            regions: regions
                .iter()
                .enumerate()
                .map(|(region_index, region)| {
                    let slot = region_index * plan.layers.len() + index;
                    DeepSparseRegion {
                        fill_surfaces: horizontal.records[slot]
                            .as_ref()
                            .map_or(&[], |record| record.fill_surfaces.as_slice()),
                        sparse_infill_density_percent: region.as_parts().1.sparse_infill_density.0,
                    }
                })
                .collect(),
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
                object_options: view.object_options,
                default_region: view.default_region,
            })?;
        }
    }
    Ok(())
}

/// `PrintObject.cpp:2593` two-phase lightning expansion. For each layer whose
/// lower layer uses `ipLightning` sparse infill with `stInternal` area, expand
/// the layer's own `stInternal` into `stInternalSolid` near the bridge
/// candidates (`expand(new_polys, scaled(3))`). The expanded `fill_surfaces`
/// per region/layer slot is stored on `candidates.lightning_expanded_surfaces` for the
/// generator build (anchoring + emit).
pub(super) fn populate_lightning_expanded_surfaces(
    candidates: &mut BridgeCandidateObject,
    horizontal: &PreparedSurfaceTypeObject,
    prelude: &PostPerimeterInputPrintObject,
    scale: CoordinateScale,
) -> Result<(), SliceError> {
    use crate::ProcessInfillPattern;
    use crate::project_slice::region_slices::RegionSurfaceKind;
    let (post_regions, _) = prelude.object.as_parts();
    let (_, _, regions) = post_regions.as_parts();
    for (&layer_index, candidate_surfaces) in &candidates.surfaces_by_layer {
        // Phase-1 gate: layer_index's lower layer uses lightning + has stInternal.
        let Some(lower) = layer_index.checked_sub(1) else {
            continue;
        };
        let lower_uses_lightning = regions
            .iter()
            .enumerate()
            .filter(|(_, region)| {
                region.as_parts().1.sparse_infill_pattern == ProcessInfillPattern::Lightning
            })
            .filter_map(|(region_index, _)| {
                let slot = prelude.region_layer_record_index(RegionLayerIndex {
                    region_index,
                    layer_index: lower,
                });
                horizontal.records[slot].as_ref()
            })
            .any(|record| {
                record
                    .fill_surfaces
                    .iter()
                    .any(|surface| surface.as_parts().0 == RegionSurfaceKind::Internal)
            });
        if !lower_uses_lightning {
            continue;
        }
        // Phase-2: expand this layer's fill_surfaces near its own candidates.
        for region_index in 0..regions.len() {
            let slot = prelude.region_layer_record_index(RegionLayerIndex {
                region_index,
                layer_index,
            });
            let Some(record) = horizontal.records[slot].as_ref() else {
                break;
            };
            if !record
                .fill_surfaces
                .iter()
                .any(|surface| surface.as_parts().0 == RegionSurfaceKind::Internal)
            {
                break;
            }
            let new_polygons = candidate_surfaces
                .iter()
                .filter(|candidate| candidate.source.region_index == region_index)
                .map(|candidate| candidate.new_polygons.as_slice());
            let expanded = super::super::lightning_expansion::expand_region(
                &record.fill_surfaces,
                new_polygons,
                scale,
            )
            .map_err(geometry_error)?;
            candidates
                .lightning_expanded_surfaces
                .insert(slot, expanded);
        }
    }
    Ok(())
}

/// `prepare_lightning_infill_data` on the expanded surfaces — build the
/// Configuration read by `FillLightning::Generator::Generator`
/// (`Fill/Lightning/Generator.cpp:66-99`), resolved for one print object.
pub(super) struct LightningConfiguration<'a> {
    pub(super) nozzles: &'a OrcaFloats,
    pub(super) object_options: &'a crate::ObjectOptions,
    pub(super) scale: CoordinateScale,
}

/// per-object `FillLightning::Generator`. `layer_internal_outlines` = the
/// `stInternal`/`stInternalVoid` expolygons of every region's fill surfaces.
/// Expanded surfaces replace only their matching canonical region/layer slot.
pub(super) fn build_lightning_generator(
    candidates: &BridgeCandidateObject,
    horizontal: &PreparedSurfaceTypeObject,
    regions: &[crate::project_slice::region_slices::PostRegion],
    prelude: &PostPerimeterInputPrintObject,
    LightningConfiguration {
        nozzles,
        object_options,
        scale,
    }: LightningConfiguration<'_>,
) -> Result<Option<std::rc::Rc<crate::fill::lightning::generator::Generator>>, SliceError> {
    use crate::project_slice::region_slices::RegionSurfaceKind;
    let (post_regions, _) = prelude.object.as_parts();
    let (plan, _, _) = post_regions.as_parts();
    if !regions.iter().any(|region| {
        let options = region.as_parts().1;
        options.sparse_infill_density.0 > 0.0
            && options.sparse_infill_pattern == crate::ProcessInfillPattern::Lightning
    }) {
        return Ok(None);
    }
    // Generator.cpp:70 uses the first shared region, not the activating region.
    let options = regions[0].as_parts().1;
    // `*std::max_element(nozzle_diameters)` (Generator.cpp:72) — the MAX nozzle
    // diameter, not the min.
    let nozzle = nozzles
        .0
        .iter()
        .map(|v| v.0)
        .reduce(f64::max)
        .unwrap_or(0.4);
    // `scaled<float>(sparse_infill_line_width.get_abs_value(nozzle))`
    // (Generator.cpp:79) — `get_abs_value` resolves the configured
    // FloatOrPercent against the DOUBLE nozzle (no fallback chain, no f32
    // narrowing), then `scaled<float>` = `f32(mm*1e6)`. Resolving via the
    // f32-narrowed `resolved_width` first loses the last ulp (0.42 → 419999
    // instead of 420000).
    let sparse_width = match options.sparse_infill_line_width {
        crate::FloatOrPercent::Float(value) => value,
        crate::FloatOrPercent::Percent(percent) => percent.0 / 100.0 * nozzle,
    };
    // `line_width.get_abs_value(nozzle)` — the EPSILON fallback source
    // (Generator.cpp:85-87), resolved in f64 for the same reason.
    let line_width = match object_options.line_width {
        crate::FloatOrPercent::Float(value) => value,
        crate::FloatOrPercent::Percent(percent) => percent.0 / 100.0 * nozzle,
    };
    // `object_config.layer_height` (Generator.cpp:77) — the configured ordinary
    // layer height, NOT the first/planned layer height.
    let layer_height = object_options.layer_height.0;
    let layer_internal_outlines: Vec<Vec<crate::geometry::Polygon>> = (0..plan.layers.len())
        .map(|layer_id| {
            let mut outlines = Vec::new();
            for region_index in 0..regions.len() {
                let slot = prelude.region_layer_record_index(RegionLayerIndex {
                    region_index,
                    layer_index: layer_id,
                });
                let surfaces: &[crate::project_slice::region_slices::RegionSurface] =
                    match candidates.lightning_expanded_surfaces.get(&slot) {
                        Some(expanded) => expanded.as_slice(),
                        None => horizontal.records[slot]
                            .as_ref()
                            .map_or(&[], |record| record.fill_surfaces.as_slice()),
                    };
                for surface in surfaces.iter().filter(|surface| {
                    matches!(
                        surface.as_parts().0,
                        RegionSurfaceKind::Internal | RegionSurfaceKind::InternalVoid
                    )
                }) {
                    let (_, expolygon, ..) = surface.as_parts();
                    outlines.push(expolygon.contour().clone());
                    outlines.extend(expolygon.holes().iter().cloned());
                }
            }
            outlines
        })
        .collect();
    crate::fill::lightning::generator::Generator::new(
        layer_internal_outlines,
        crate::fill::lightning::generator::InfillSettings {
            layer_height,
            infill_line_width: sparse_width,
            line_width,
            nozzle_diameter: nozzle,
            multiline: options.fill_multiline.0,
            density_percent: options.sparse_infill_density.0,
            overhang_angle_deg: options.lightning_overhang_angle.0,
            prune_angle_deg: options.lightning_prune_angle.0,
            straightening_angle_deg: options.lightning_straightening_angle.0,
            scale,
        },
    )
    .map(|g| Some(std::rc::Rc::new(g)))
    .map_err(|e| SliceError::InvalidInput(format!("lightning generator: {e:?}")))
}

mod layers;
mod lower_direction;
use layers::{LayerJob, prelude, prepare_infill_lines, prepare_layer};
