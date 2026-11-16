//! `fill_entities::generate_layer` — per-layer fill entity generation, split
//! from `fill_entities.rs` for the <400 LOC rule.

use crate::{
    ExtrusionRole, ProcessInfillPattern, SliceError,
    project_slice::{
        group_fills::{SurfaceFillPattern, group_fills},
        prepare_infill::external_surfaces::PreparedPostExternalSurfaces,
    },
};

use super::types::LayerFillEntities;

pub(in crate::project_slice) struct LayerGenerationOptions<'a> {
    pub(in crate::project_slice) lightning_generator:
        Option<&'a crate::fill::lightning::generator::Generator>,
    pub(in crate::project_slice) grid_bounds: Option<crate::geometry::BoundingBox>,
    pub(in crate::project_slice) object_spacing: &'a super::adaptive::ObjectSpacing,
}

pub(in crate::project_slice) fn generate_layer_with_active(
    prepared: &PreparedPostExternalSurfaces,
    object_index: usize,
    layer_index: usize,
    options: LayerGenerationOptions<'_>,
) -> Result<LayerFillEntities, SliceError> {
    let LayerGenerationOptions {
        lightning_generator,
        grid_bounds,
        object_spacing,
    } = options;
    let grouped = group_fills(prepared, object_index, layer_index)?;
    let traversal = &prepared.predecessor.predecessor;
    let minimum_nozzle_diameter = traversal
        .resolved
        .views
        .full
        .project
        .print
        .nozzle_diameter
        .0
        .iter()
        .map(|value| value.0)
        .reduce(f64::min)
        .expect("validated print configuration has a nozzle diameter");
    let traversal_object = &traversal.objects[object_index];
    let prelude = &traversal_object
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .object;
    let (compensated, _) = prelude.as_parts();
    let (post_regions, object_slices) = compensated.as_parts();
    let (plan, _, _) = post_regions.as_parts();
    let layer = &plan.layers[layer_index];
    let resolved_object = traversal
        .resolved
        .objects
        .iter()
        .find(|object| object.source_object_index == prelude.identity().0)
        .expect("fill generation retains its resolved object");
    let object_options = &resolved_object.object;
    let mut output = LayerFillEntities::default();
    // `PrintObject::bounding_box` (Print.hpp:347) is symmetric around zero;
    // `Fill::_infill_direction` uses that center, not quantized slice bounds.
    let object_reference = crate::geometry::Point::new(0, 0);
    // `FillRectilinear::extended_object_bounding_box()` (FillRectilinear.cpp:2741) — the
    // object bbox transpose-merged + sqrt(2)-scaled so the consistent-pattern
    // vertical-line grid is anchored object-wide, independent of rotation.
    // Computed lazily: only consistent-pattern fills consume it, and a fully
    // compensated-away object has no contours to bound (`Fill.cpp:1224`
    // derives the bbox from the mesh `m_size`, never requiring contours).
    let mut extended_object_bounds_x: Option<(i64, i64)> = None;
    let mut extended_object_bounds = || {
        let (extended_object_min_x, extended_object_max_x) = *extended_object_bounds_x
            .get_or_insert_with(|| {
                let bounds = super::object_bounds::object(
                    &traversal.project.objects()[plan.source_object_index],
                    resolved_object.print_objects[plan.transform_index].transform,
                    traversal.scale,
                );
                super::extended_object_bounding_box_x(bounds)
            });
        super::types::ObjectFillGeometry {
            scale: traversal.scale,
            extended_object_min_x,
            extended_object_max_x,
        }
    };
    // `FillLockedZag::set_lock_region_param` (`Fill.cpp:1309`) — the skin/
    // skeleton density+flow maps, built per-layer in `group_fills`.
    let crate::project_slice::group_fills::GroupedFills {
        surface_fills,
        lock_region_param,
    } = grouped;

    for fill in surface_fills {
        let region_id = fill.region_id;
        let collection_start = output.collections.len();
        // Fill.cpp:1235-1242: each filler uses its printing region's runtime flag.
        let region = &resolved_object.all_regions[region_id];
        let call = super::types::SurfaceCall {
            layer_id: layer.id,
            dont_alternate_fill_direction: region.zaa_enabled.0
                && region.zaa_dont_alternate_fill_direction.0,
            reference: object_reference,
            scale: traversal.scale,
            link_max_length: 0.0,
            dont_adjust: fill.params.bridge,
        };
        match fill.params.pattern {
            SurfaceFillPattern::Configured(ProcessInfillPattern::SpiralInset) => {
                return Err(crate::SliceError::UnsupportedProjectFeature(
                    "infill pattern: spiralinset".to_owned(),
                ));
            }
            SurfaceFillPattern::Configured(ProcessInfillPattern::CrossHatch) => {
                super::crosshatch::append(&mut output, fill, layer.print_z, traversal.scale)?;
            }
            SurfaceFillPattern::Configured(ProcessInfillPattern::Grid) => {
                super::grid::append(
                    &mut output,
                    fill,
                    traversal.scale,
                    object_reference,
                    super::grid::LayerGeometry {
                        layer_id: layer.id,
                        bounds: grid_bounds,
                    },
                )?;
            }
            SurfaceFillPattern::Configured(ProcessInfillPattern::Cubic) => {
                super::cubic::append(
                    &mut output,
                    fill,
                    super::cubic::LayerGeometry {
                        print_z: layer.print_z,
                        layer_id: layer.id,
                    },
                    traversal.scale,
                    object_reference,
                )?;
            }
            SurfaceFillPattern::Configured(ProcessInfillPattern::Triangles) => {
                super::triangles::append(
                    &mut output,
                    fill,
                    traversal.scale,
                    object_reference,
                    super::triangles::LayerGeometry { layer_id: layer.id },
                )?;
            }
            SurfaceFillPattern::Configured(ProcessInfillPattern::Gyroid) => {
                super::gyroid::append(
                    &mut output,
                    fill,
                    layer.print_z,
                    layer.height,
                    traversal.scale,
                )?;
            }
            SurfaceFillPattern::Configured(
                pattern
                @ (ProcessInfillPattern::AdaptiveCubic | ProcessInfillPattern::SupportCubic),
            ) => {
                // Fill.cpp:1386 selects the tree using the fill's printing region.
                let line_spacing = object_spacing.for_pattern(pattern, region_id);
                super::adaptive::append(
                    &mut output,
                    fill,
                    super::adaptive::LayerGeometry {
                        print_z: layer.print_z,
                        layer_id: layer.id,
                    },
                    super::adaptive::AdaptiveObject::new(traversal, object_index, line_spacing),
                    traversal.scale,
                )?;
            }
            SurfaceFillPattern::Configured(ProcessInfillPattern::ThreeDHoneycomb) => {
                super::three_d_honeycomb::append(
                    &mut output,
                    fill,
                    layer.print_z,
                    layer.height,
                    traversal.scale,
                )?;
            }
            SurfaceFillPattern::Configured(ProcessInfillPattern::Honeycomb) => {
                super::honeycomb::append(&mut output, fill, call)?;
            }
            SurfaceFillPattern::Configured(ProcessInfillPattern::Line) => {
                super::line::append(&mut output, fill, call)?;
            }
            SurfaceFillPattern::Configured(
                pattern @ (ProcessInfillPattern::Rectilinear
                | ProcessInfillPattern::AlignedRectilinear
                | ProcessInfillPattern::ZigZag
                | ProcessInfillPattern::CrossZag
                | ProcessInfillPattern::Monotonic
                | ProcessInfillPattern::MonotonicLine),
            ) => {
                super::monotonic::append(
                    &mut output,
                    fill,
                    pattern,
                    call,
                    extended_object_bounds(),
                )?;
            }
            SurfaceFillPattern::Configured(ProcessInfillPattern::TriHexagon) => {
                // `tri-hexagon` maps to `ipStars` (`PrintConfig.cpp:238`).
                super::stars::append(
                    &mut output,
                    fill,
                    traversal.scale,
                    object_reference,
                    super::stars::LayerGeometry { layer_id: layer.id },
                )?;
            }
            SurfaceFillPattern::Configured(ProcessInfillPattern::TpmsFk) => {
                super::tpms_fk::append(&mut output, fill, layer.print_z, traversal.scale)?;
            }
            SurfaceFillPattern::Configured(ProcessInfillPattern::TpmsD) => {
                super::tpms_d::append(&mut output, fill, layer.print_z, traversal.scale)?;
            }
            SurfaceFillPattern::Configured(ProcessInfillPattern::LateralLattice) => {
                super::lateral_lattice::append(
                    &mut output,
                    fill,
                    layer,
                    traversal.scale,
                    object_reference,
                )?;
            }
            SurfaceFillPattern::Configured(ProcessInfillPattern::LateralHoneycomb) => {
                super::lateral_honeycomb::append(
                    &mut output,
                    fill,
                    layer,
                    traversal.scale,
                    object_reference,
                )?;
            }
            SurfaceFillPattern::Configured(ProcessInfillPattern::QuarterCubic) => {
                super::quartercubic::append(
                    &mut output,
                    fill,
                    layer.print_z,
                    traversal.scale,
                    object_reference,
                )?;
            }
            SurfaceFillPattern::Configured(ProcessInfillPattern::LockedZag) => {
                super::locked_zag::append(
                    &mut output,
                    fill,
                    &lock_region_param,
                    call,
                    extended_object_bounds(),
                )?;
            }
            SurfaceFillPattern::Configured(
                pattern @ (ProcessInfillPattern::HilbertCurve
                | ProcessInfillPattern::ArchimedeanChords
                | ProcessInfillPattern::OctagramSpiral),
            ) if matches!(
                fill.params.extrusion_role,
                ExtrusionRole::TopSolidInfill
                    | ExtrusionRole::BottomSurface
                    | ExtrusionRole::SolidInfill
                    // Sparse plane-path fills still emit InternalInfill.
                    | ExtrusionRole::InternalInfill
            ) =>
            {
                let object_bounding_box = super::object_bounds::object(
                    &traversal.project.objects()[plan.source_object_index],
                    resolved_object.print_objects[plan.transform_index].transform,
                    traversal.scale,
                );
                super::plane_path::append(
                    &mut output,
                    fill,
                    pattern,
                    object_bounding_box,
                    traversal.resolved.views.full.process.print.resolution.0,
                    object_options.calib_flowrate_topinfill_special_order.0,
                    traversal.scale,
                )?;
            }
            SurfaceFillPattern::Configured(
                ProcessInfillPattern::HilbertCurve
                | ProcessInfillPattern::ArchimedeanChords
                | ProcessInfillPattern::OctagramSpiral,
            ) => {}
            SurfaceFillPattern::ConcentricInternal => {
                super::concentric::append(
                    &mut output,
                    fill,
                    layer.height,
                    minimum_nozzle_diameter,
                    traversal.scale,
                )?;
            }
            SurfaceFillPattern::Configured(ProcessInfillPattern::Lightning) => {
                if let Some(generator) = lightning_generator {
                    super::lightning::append(
                        &mut output,
                        generator,
                        &fill,
                        layer.id,
                        traversal.scale,
                    )?;
                }
            }
            SurfaceFillPattern::Configured(ProcessInfillPattern::Concentric) => {
                super::concentric::append_configured(
                    &mut output,
                    fill,
                    layer.height,
                    minimum_nozzle_diameter,
                    traversal.scale,
                )?;
            }
        }
        output
            .collection_ranges
            .push((region_id, collection_start..output.collections.len()));
    }
    super::ironing::append_spans(super::ironing::SpanGeneration {
        output: &mut output,
        prepared,
        object_index,
        layer_index,
        prelude,
        layer,
        layers: &plan.layers,
        layer_slices: &object_slices[layer_index],
        object: object_options,
        default_region: &traversal.resolved.views.full.process.region,
        nozzles: &traversal.resolved.views.full.project.print.nozzle_diameter,
        filament_map: &traversal.resolved.applied_filament_map,
        is_bbl: crate::print::validate::project_is_bbl(
            &traversal.resolved.views.full,
            &traversal.project.documents().project_settings_raw,
            None,
        )?,
        scale: traversal.scale,
    })?;
    let process = &traversal.resolved.views.full.process;
    super::simplify::apply(
        &mut output,
        process.print.resolution.0.max(1.0e-4),
        traversal.scale,
        process.gcode.enable_arc_fitting,
        process.print.spiral_mode.0,
    );
    Ok(output)
}

pub(in crate::project_slice) fn layer_options<'a>(
    prepared: &'a PreparedPostExternalSurfaces,
    object_index: usize,
    lightning_generator: Option<&'a crate::fill::lightning::generator::Generator>,
    grid_bounds: Option<crate::geometry::BoundingBox>,
) -> LayerGenerationOptions<'a> {
    LayerGenerationOptions {
        lightning_generator,
        grid_bounds,
        object_spacing: &prepared.object_spacings[object_index],
    }
}
