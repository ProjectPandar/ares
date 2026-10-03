//! `PrintObject.cpp:2024-2151::discover_vertical_shells` physical cache ownership.

use crate::{
    ProcessEnsureVerticalShellThickness, SliceError,
    geometry::{JoinType, offset_expolygons_paths, offset_paths, union_polygons_paths},
    project_slice::{
        perimeters::{
            flow::{FlowContext, PerimeterFlowRole, resolve_role_flow},
            types::RegionLayerIndex,
        },
        prepare_infill::fill_surfaces::PreparedPostFillSurfacePreparation,
    },
};

use super::{cache, types::PhysicalShellCache};

pub(super) fn build(
    prepared: &PreparedPostFillSurfacePreparation,
    object_index: usize,
    num_layers: usize,
    shared: bool,
) -> Result<PhysicalShellCache, SliceError> {
    let traversal = &prepared.predecessor;
    let prelude = &traversal.objects[object_index]
        .predecessor
        .predecessor
        .predecessor
        .predecessor;
    let input_object = &prelude.object;
    let (compensated, _) = input_object.as_parts();
    let (post_regions, lslices) = compensated.as_parts();
    let (plan, _, regions) = post_regions.as_parts();
    let object = &prepared.objects[object_index];
    if !shared {
        let holes = (0..num_layers)
            .map(|layer_index| {
                (0..regions.len())
                    .filter_map(|region_index| {
                        let slot = input_object.region_layer_record_index(RegionLayerIndex {
                            region_index,
                            layer_index,
                        });
                        object.records[slot].as_ref()
                    })
                    .flat_map(|record| &record.fill_expolygons)
                    .flat_map(|polygon| std::iter::once(polygon.contour()).chain(polygon.holes()))
                    .cloned()
                    .collect()
            })
            .collect();
        return Ok(PhysicalShellCache::RegionalHoles(holes));
    }
    let enabled = regions.iter().any(|region| {
        region.options.ensure_vertical_shell_thickness
            == ProcessEnsureVerticalShellThickness::EnsureAll
    });
    if !enabled {
        return Ok(PhysicalShellCache::Shared(
            (0..num_layers).map(|_| cache::empty()).collect(),
        ));
    }
    let settings = &traversal.resolved.views.full;
    let options = &traversal
        .resolved
        .objects
        .iter()
        .find(|object| object.source_object_index == plan.source_object_index)
        .expect("vertical shells retain their resolved object")
        .object;
    let scale = traversal.scale;
    let mut layers = Vec::with_capacity(num_layers);
    for (layer_index, planned) in plan.layers.iter().take(num_layers).enumerate() {
        let mut combined = cache::empty();
        let mut perimeter_offset = 0.0_f32;
        let mut perimeter_min_spacing = f32::MAX;
        for (region_index, region) in regions.iter().enumerate() {
            let slot = input_object.region_layer_record_index(RegionLayerIndex {
                region_index,
                layer_index,
            });
            // Native LayerRegion::flow is defined even when this region has no slices.
            let context = FlowContext {
                layer: planned,
                initial_layer_width: settings.process.print.initial_layer_line_width,
                region: &region.options,
                object: options,
                nozzle_diameters: &settings.project.print.nozzle_diameter,
            };
            let solid_flow = resolve_role_flow(context, PerimeterFlowRole::SolidInfill)?;
            let record = object.records[slot].as_ref();
            let slices = record.map_or(&[][..], |record| record.slices.as_slice());
            let fill = record.map_or(&[][..], |record| record.fill_expolygons.as_slice());
            let regional = cache::build(
                slices,
                fill,
                ProcessEnsureVerticalShellThickness::EnsureAll,
                solid_flow.scaled_spacing(scale),
            )?;
            combined.top_surfaces.extend(regional.top_surfaces);
            combined.bottom_surfaces.extend(regional.bottom_surfaces);
            combined.holes.extend(regional.holes);
            let perimeters = slices
                .iter()
                .map(|surface| u32::from(surface.as_parts().5))
                .max()
                .unwrap_or(0)
                .wrapping_add(region.options.wall_loops.0 as u32);
            if perimeters > 0 {
                let external_flow = resolve_role_flow(context, PerimeterFlowRole::External)?;
                let internal_flow = resolve_role_flow(context, PerimeterFlowRole::Internal)?;
                let external_spacing = external_flow.scaled_spacing(scale);
                let internal_spacing = internal_flow.scaled_spacing(scale);
                perimeter_offset = perimeter_offset.max(
                    0.5_f32 * (external_flow.scaled_width(scale) + external_spacing) as f32
                        + (perimeters as f32 - 1.0_f32) * internal_spacing as f32,
                );
                perimeter_min_spacing =
                    perimeter_min_spacing.min(external_spacing.min(internal_spacing) as f32);
            }
        }
        combined.top_surfaces =
            union_polygons_paths(&combined.top_surfaces).map_err(geometry_error)?;
        combined.bottom_surfaces =
            union_polygons_paths(&combined.bottom_surfaces).map_err(geometry_error)?;
        if perimeter_offset > 0.0 {
            let expansion = 0.3_f32 * perimeter_min_spacing;
            let expanded =
                offset_expolygons_paths(&lslices[layer_index], expansion, JoinType::Miter, 3.0)
                    .map_err(geometry_error)?;
            combined.holes.extend(
                offset_paths(
                    &expanded,
                    -perimeter_offset - expansion,
                    JoinType::Miter,
                    3.0,
                )
                .map_err(geometry_error)?,
            );
        }
        combined.holes = union_polygons_paths(&combined.holes).map_err(geometry_error)?;
        layers.push(combined);
    }
    Ok(PhysicalShellCache::Shared(layers))
}

fn geometry_error(_: crate::geometry::ClipperError) -> SliceError {
    SliceError::InvalidInput(
        "vertical-shell cache geometry is outside the supported Clipper range".to_owned(),
    )
}
