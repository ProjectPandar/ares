use crate::{
    OrcaFloats, SliceError,
    geometry::CoordinateScale,
    project_slice::{
        perimeters::types::PostPerimeterInputPrintObject,
        prepare_infill::{
            bridge_over_infill::types::BridgeCandidateObject,
            external_surfaces::PreparedPostExternalSurfaces,
            surface_type_detection::types::PreparedSurfaceTypeObject,
        },
    },
};

use super::super::{deep_sparse_area::DeepSparseLayer, layer_clustering::cluster_candidate_object};
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
pub(super) struct ObjectView<'a> {
    horizontal: &'a PreparedSurfaceTypeObject,
    prelude: &'a PostPerimeterInputPrintObject,
    nozzles: &'a OrcaFloats,
    scale: CoordinateScale,
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
    } = view;
    let (compensated, inputs) = prelude.as_parts();
    let (post_regions, _) = compensated.as_parts();
    let (plan, _, regions) = post_regions.as_parts();
    // `PrintObject.cpp:2593` LIGHTNING INFILL SECTION — expand `stInternal` to
    // `stInternalSolid` near candidates on layers whose lower layer uses
    // lightning sparse infill; the expanded set feeds the generator. Runs
    // before the empty-candidate early return so the generator exists even
    // when there are no bridge candidates (`make_fills` needs it regardless).
    if candidates.has_lightning_infill {
        populate_lightning_expanded_surfaces(candidates, horizontal, view.scale)?;
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
            plan,
            nozzles,
            object_options,
            view.scale,
        )?;
    }
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

/// `PrintObject.cpp:2593` two-phase lightning expansion. For each layer whose
/// lower layer uses `ipLightning` sparse infill with `stInternal` area, expand
/// the layer's own `stInternal` into `stInternalSolid` near the bridge
/// candidates (`expand(new_polys, scaled(3))`). The expanded `fill_surfaces`
/// per layer is stored on `candidates.lightning_expanded_surfaces` for the
/// generator build (anchoring + emit).
pub(super) fn populate_lightning_expanded_surfaces(
    candidates: &mut BridgeCandidateObject,
    horizontal: &PreparedSurfaceTypeObject,
    scale: CoordinateScale,
) -> Result<(), SliceError> {
    use crate::ProcessInfillPattern;
    use crate::project_slice::region_slices::RegionSurfaceKind;
    for (&layer_index, candidate_surfaces) in &candidates.surfaces_by_layer {
        // Phase-1 gate: layer_index's lower layer uses lightning + has stInternal.
        let Some(lower) = layer_index.checked_sub(1) else {
            continue;
        };
        let Some(lower_record) = horizontal.records[lower].as_ref() else {
            continue;
        };
        // Does the lower layer use lightning? Check via the candidates'
        // has_lightning_infill flag plus whether the lower layer still has
        // `Internal` (stInternal) sparse surfaces. The pattern check per-region
        // is folded into `has_lightning_infill` (object-level).
        let has_internal = lower_record
            .fill_surfaces
            .iter()
            .any(|s| s.as_parts().0 == RegionSurfaceKind::Internal);
        if !has_internal {
            continue;
        }
        // Phase-2: expand this layer's fill_surfaces near its own candidates.
        let Some(record) = horizontal.records[layer_index].as_ref() else {
            continue;
        };
        let new_polygons: Vec<crate::geometry::Polygon> = candidate_surfaces
            .iter()
            .flat_map(|c| c.new_polygons.clone())
            .collect();
        let expanded = super::super::lightning_expansion::expand_layer(
            &record.fill_surfaces,
            &new_polygons,
            scale,
        )
        .map_err(geometry_error)?;
        candidates
            .lightning_expanded_surfaces
            .insert(layer_index, expanded);
        let _ = ProcessInfillPattern::Lightning;
    }
    Ok(())
}

/// `prepare_lightning_infill_data` on the expanded surfaces — build the
/// per-object `FillLightning::Generator`. `layer_internal_outlines` = the
/// `stInternal`/`stInternalVoid` expolygons of `record.fill_surfaces`, using
/// the expanded set where `lightning_expanded_surfaces` has one.
pub(super) fn build_lightning_generator(
    candidates: &BridgeCandidateObject,
    horizontal: &PreparedSurfaceTypeObject,
    regions: &[crate::project_slice::region_slices::PostRegion],
    plan: &crate::project_slice::layers::PlannedPrintObject,
    nozzles: &OrcaFloats,
    object_options: &crate::ObjectOptions,
    scale: CoordinateScale,
) -> Result<Option<std::rc::Rc<crate::fill::lightning::generator::Generator>>, SliceError> {
    use crate::project_slice::region_slices::RegionSurfaceKind;
    let Some(lightning_region) = regions
        .iter()
        .find(|r| r.as_parts().1.sparse_infill_pattern == crate::ProcessInfillPattern::Lightning)
    else {
        return Ok(None);
    };
    let options = lightning_region.as_parts().1;
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
    let internal_outlines = |surfaces: &[crate::project_slice::region_slices::RegionSurface]| {
        surfaces
            .iter()
            .filter(|s| {
                matches!(
                    s.as_parts().0,
                    RegionSurfaceKind::Internal | RegionSurfaceKind::InternalVoid
                )
            })
            .flat_map(|s| {
                let mut v = vec![s.as_parts().1.contour().clone()];
                v.extend(s.as_parts().1.holes().iter().cloned());
                v
            })
            .collect::<Vec<_>>()
    };
    let layer_internal_outlines: Vec<Vec<crate::geometry::Polygon>> = (0..plan.layers.len())
        .map(|layer_id| {
            if let Some(expanded) = candidates.lightning_expanded_surfaces.get(&layer_id) {
                internal_outlines(expanded)
            } else {
                horizontal
                    .records
                    .get(layer_id)
                    .and_then(|r| r.as_ref())
                    .map(|r| internal_outlines(&r.fill_surfaces))
                    .unwrap_or_default()
            }
        })
        .collect();
    if std::env::var("ARES_DEBUG_LIGHTNING").is_ok() {
        eprintln!(
            "LIGHTNING-IN rec_some={:?} outlines={:?}",
            horizontal
                .records
                .iter()
                .map(|r| r.is_some())
                .collect::<Vec<_>>(),
            layer_internal_outlines
                .iter()
                .map(Vec::len)
                .collect::<Vec<_>>()
        );
    }
    crate::fill::lightning::generator::Generator::new(
        layer_internal_outlines,
        layer_height,
        sparse_width,
        line_width,
        nozzle,
        options.fill_multiline.0,
        options.sparse_infill_density.0,
        options.lightning_overhang_angle.0,
        options.lightning_prune_angle.0,
        options.lightning_straightening_angle.0,
        scale,
    )
    .map(|g| Some(std::rc::Rc::new(g)))
    .map_err(|e| SliceError::InvalidInput(format!("lightning generator: {e:?}")))
}

mod layers;
use layers::{LayerJob, prelude, prepare_infill_lines, prepare_layer};
