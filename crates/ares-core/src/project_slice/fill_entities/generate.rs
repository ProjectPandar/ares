//! `fill_entities::generate_layer` — per-layer fill entity generation, split
//! from `fill_entities.rs` for the <400 LOC rule.

use crate::{
    ExtrusionRole, ProcessInfillPattern, SliceError,
    project_slice::{
        group_fills::{SurfaceFillPattern, group_fills},
        prepare_infill::external_surfaces::PreparedPostExternalSurfaces,
    },
};

use super::simplify_before_ordering;
use super::types::LayerFillEntities;

pub(in crate::project_slice) fn generate_layer(
    prepared: &PreparedPostExternalSurfaces,
    object_index: usize,
    layer_index: usize,
    lightning_generator: Option<&crate::fill::lightning::generator::Generator>,
    stars_bounds: Option<crate::geometry::BoundingBox>,
) -> Result<LayerFillEntities, SliceError> {
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
    // `Fill::extended_object_bounding_box()` (`FillBase.cpp:1569-1578`) — the
    // object bbox transpose-merged + sqrt(2)-scaled so the consistent-pattern
    // vertical-line grid is anchored object-wide, independent of rotation.
    // Computed lazily: only consistent-pattern fills consume it, and a fully
    // compensated-away object has no contours to bound (`Fill.cpp:1224`
    // derives the bbox from the mesh `m_size`, never requiring contours).
    let mut extended_object_bounds_x: Option<(i64, i64)> = None;
    let mut extended_object_bounds = || {
        let (extended_object_min_x, extended_object_max_x) = *extended_object_bounds_x
            .get_or_insert_with(|| {
                // `extended_object_bounding_box` (`FillBase.cpp:1569-1578`) reads
                // `PrintObject::bounding_box()` (`Print.hpp:347`), the centered mesh
                // `m_size`. Empirically the ORACLE's grid anchor matches the
                // slice-contour extent, not the raw mesh `m_size` — on the
                // uncompensated cube the raw-mesh `m_size` (10mm) produced a
                // narrower grid (-7.07mm) than the oracle's actual anchor
                // (-8.14mm, = contour extent). So `bounding_box` for a fill is
                // anchored on the compensated slice contours in practice (the
                // perimeter/slice that was actually generated), which we reproduce
                // here via `plane_path_object_bounding_box` centered on the
                // origin. See the residual map for the divergence analysis.
                plane_path_object_bounding_box(object_slices)
                    .map(super::extended_object_bounding_box_x)
                    .unwrap_or((0, 0))
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
                        bounds: stars_bounds,
                    },
                )?;
            }
            SurfaceFillPattern::Configured(ProcessInfillPattern::Cubic) => {
                super::cubic::append(
                    &mut output,
                    fill,
                    layer.print_z,
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
                    super::triangles::LayerGeometry {
                        layer_id: layer.id,
                        bounds: stars_bounds,
                    },
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
                let _ = pattern;
                super::adaptive::append(
                    &mut output,
                    fill,
                    layer.print_z,
                    super::adaptive::AdaptiveObject {
                        traversal,
                        object_index,
                    },
                    traversal.scale,
                )?;
            }
            SurfaceFillPattern::Configured(ProcessInfillPattern::ThreeDHoneycomb) => {
                super::three_d_honeycomb::append(
                    &mut output,
                    fill,
                    layer.print_z,
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
                    super::stars::LayerGeometry {
                        layer_id: layer.id,
                        bounds: stars_bounds,
                    },
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
                let Some(object_bounding_box) = plane_path_object_bounding_box(object_slices)
                else {
                    continue;
                };
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
                    minimum_nozzle_diameter,
                    traversal.scale,
                )?;
            }
        }
        output
            .collection_ranges
            .push((region_id, collection_start..output.collections.len()));
    }
    let region_count = prelude.records.len() / plan.layers.len();
    let mut spans = Vec::with_capacity(region_count);
    for region_index in 0..region_count {
        let record_index = region_index * plan.layers.len() + layer_index;
        if let (Some(record), Some(input)) = (
            prepared.predecessor.objects[object_index].records[record_index].as_ref(),
            prelude.records[record_index].as_ref(),
        ) {
            spans.push((record_index, record, input));
        }
    }
    spans.sort_by(|(_, _, left), (_, _, right)| {
        let left = prelude.region_options(left);
        let right = prelude.region_options(right);
        left.top_surface_filament_id
            .0
            .cmp(&right.top_surface_filament_id.0)
            .then_with(|| {
                left.filament_ironing_spacing
                    .0
                    .total_cmp(&right.filament_ironing_spacing.0)
            })
            .then_with(|| {
                (object_options.layer_height.0 * left.filament_ironing_flow.0)
                    .total_cmp(&(object_options.layer_height.0 * right.filament_ironing_flow.0))
            })
            .then_with(|| {
                left.filament_ironing_speed
                    .0
                    .total_cmp(&right.filament_ironing_speed.0)
            })
            .then_with(|| left.ironing_angle.0.total_cmp(&right.ironing_angle.0))
            .then_with(|| left.ironing_angle_fixed.0.cmp(&right.ironing_angle_fixed.0))
            .then_with(|| {
                left.filament_ironing_inset
                    .0
                    .total_cmp(&right.filament_ironing_inset.0)
            })
    });
    let mut previous_ironing_pattern = None;
    for (_, record, input) in spans {
        let region = prelude.region_options(input);
        let collection_start = output.collections.len();
        let pattern_replaced =
            previous_ironing_pattern.is_some_and(|previous| previous != region.ironing_pattern);
        previous_ironing_pattern = Some(region.ironing_pattern);
        super::ironing::append(
            &mut output,
            super::ironing::Input {
                record,
                layer_slices: &object_slices[layer_index],
                region,
                dont_alternate_fill_direction: region.zaa_enabled.0
                    && region.zaa_dont_alternate_fill_direction.0
                    && !pattern_replaced,
                object: object_options,
                default_region: &traversal.resolved.views.full.process.region,
                nozzles: &traversal.resolved.views.full.project.print.nozzle_diameter,
                layer,
                layers: &plan.layers,
                last_layer: input.upper_layer_index.is_none(),
                spiral_mode: input.spiral_mode,
                model_rotation_offset: if region.align_infill_direction_to_model.0 {
                    input.model_rotation_rad as f32
                } else {
                    0.0
                },
                scale: traversal.scale,
            },
        )?;
        output.collection_ranges.push((
            input.current.region_index,
            collection_start..output.collections.len(),
        ));
    }
    let process = &traversal.resolved.views.full.process;
    if simplify_before_ordering(
        process.gcode.enable_arc_fitting.0,
        process.print.spiral_mode.0,
    ) {
        super::simplify::apply(
            &mut output,
            process.print.resolution.0.max(1.0e-4),
            traversal.scale,
        );
    }
    Ok(output)
}

fn plane_path_object_bounding_box(
    slices: &[Vec<crate::geometry::ExPolygon>],
) -> Option<crate::geometry::BoundingBox> {
    let mut points = slices
        .iter()
        .flatten()
        .flat_map(|expolygon| expolygon.contour().points());
    let first = *points.next()?;
    let (minimum, maximum) = points.fold((first, first), |(minimum, maximum), point| {
        (
            crate::geometry::Point::new(minimum.x().min(point.x()), minimum.y().min(point.y())),
            crate::geometry::Point::new(maximum.x().max(point.x()), maximum.y().max(point.y())),
        )
    });
    Some(crate::geometry::BoundingBox::new(minimum, maximum))
}
