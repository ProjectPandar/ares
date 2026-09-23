//! `fill_entities::generate_layer` — per-layer fill entity generation, split
//! from `fill_entities.rs` for the <400 LOC rule.

use crate::{
    ExtrusionRole, ProcessInfillPattern, SliceError,
    geometry::{ExPolygon, Point},
    project_slice::{
        group_fills::{SurfaceFillPattern, group_fills},
        prepare_infill::external_surfaces::PreparedPostExternalSurfaces,
    },
};

use super::types::{FillExtrusionCollection, LayerFillEntities};
use super::{geometry_error, simplify_before_ordering};

pub(in crate::project_slice) fn generate_layer(
    prepared: &PreparedPostExternalSurfaces,
    object_index: usize,
    layer_index: usize,
    lightning_generator: Option<&crate::fill::lightning::generator::Generator>,
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
    let mut plane_path_bounding_box = None;
    // `Fill::_infill_direction` anchors the infill grid on the PRINT OBJECT
    // bounding box center (FillBase.cpp:297-301), not the surface center.
    let object_reference = super::object_center(object_slices);
    // `FillLockedZag::set_lock_region_param` (`Fill.cpp:1309`) — the skin/
    // skeleton density+flow maps, built per-layer in `group_fills`.
    let crate::project_slice::group_fills::GroupedFills {
        surface_fills,
        lock_region_param,
    } = grouped;

    for fill in surface_fills {
        if let Ok(path) = std::env::var("ARES_DUMP_FLOW") {
            use std::io::Write;
            if let Ok(mut file) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
            {
                let _ = writeln!(
                    file,
                    "FLOW w={:016b} h={:016b} mm3={:016b}",
                    fill.params.flow.width.to_bits(),
                    fill.params.flow.height.to_bits(),
                    fill.params.flow.mm3_per_mm.to_bits()
                );
            }
        }
        if let Ok(path) = std::env::var("ARES_DUMP_SURF") {
            use std::io::Write;
            if let Ok(mut file) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
            {
                for expolygon in &fill.expolygons {
                    let points = expolygon.contour().points();
                    let head = points
                        .iter()
                        .take(3)
                        .map(|point| format!(",{},{}", point.x(), point.y()))
                        .collect::<String>();
                    let _ = writeln!(
                        file,
                        "SURF pattern={:?} n={} {}",
                        fill.params.pattern,
                        points.len(),
                        head
                    );
                }
            }
        }
        match fill.params.pattern {
            SurfaceFillPattern::Configured(ProcessInfillPattern::CrossHatch) => {
                super::crosshatch::append(&mut output, fill, layer.print_z, traversal.scale)?;
            }
            SurfaceFillPattern::Configured(ProcessInfillPattern::Grid) => {
                super::grid::append(
                    &mut output,
                    fill,
                    layer.id,
                    traversal.scale,
                    object_reference,
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
                super::triangles::append(&mut output, fill, traversal.scale, object_reference)?;
            }
            SurfaceFillPattern::Configured(ProcessInfillPattern::Gyroid) => {
                super::gyroid::append(&mut output, fill, layer.print_z, traversal.scale)?;
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
                    traversal,
                    object_index,
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
                super::honeycomb::append(&mut output, fill, layer.id, traversal.scale)?;
            }
            SurfaceFillPattern::Configured(ProcessInfillPattern::Line) => {
                super::line::append(
                    &mut output,
                    fill,
                    layer.id,
                    object_reference,
                    traversal.scale,
                )?;
            }
            SurfaceFillPattern::Configured(
                pattern @ (ProcessInfillPattern::Rectilinear
                | ProcessInfillPattern::AlignedRectilinear
                | ProcessInfillPattern::ZigZag
                | ProcessInfillPattern::CrossZag
                | ProcessInfillPattern::Monotonic
                | ProcessInfillPattern::MonotonicLine),
            ) => {
                super::monotonic::append(&mut output, fill, pattern, layer.id, traversal.scale)?;
            }
            SurfaceFillPattern::Configured(ProcessInfillPattern::TriHexagon) => {
                // `tri-hexagon` maps to `ipStars` (`PrintConfig.cpp:238`).
                super::stars::append(&mut output, fill, traversal.scale, object_reference)?;
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
                    layer.print_z,
                    layer.id,
                    traversal.scale,
                    object_reference,
                )?;
            }
            SurfaceFillPattern::Configured(ProcessInfillPattern::LateralHoneycomb) => {
                super::lateral_honeycomb::append(
                    &mut output,
                    fill,
                    layer.print_z,
                    layer.id,
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
                    layer.id,
                    traversal.scale,
                )?;
            }
            SurfaceFillPattern::Configured(
                pattern @ (ProcessInfillPattern::HilbertCurve
                | ProcessInfillPattern::ArchimedeanChords
                | ProcessInfillPattern::OctagramSpiral),
            ) => {
                if matches!(
                    fill.params.extrusion_role,
                    ExtrusionRole::TopSolidInfill
                        | ExtrusionRole::BottomSurface
                        | ExtrusionRole::SolidInfill
                        // Sparse infill (sparse_infill_pattern=hilbertcurve/
                        // archimedeanchords/octagramspiral) is InternalInfill
                        // and must still emit the plane-path fill.
                        | ExtrusionRole::InternalInfill
                ) {
                    let object_bounding_box = *plane_path_bounding_box
                        .get_or_insert_with(|| plane_path_object_bounding_box(object_slices));
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
            }
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
            SurfaceFillPattern::Configured(_) => {}
        }
    }
    if let (Some(record), Some(input)) = (
        prepared.predecessor.objects[object_index].records[layer_index].as_ref(),
        prelude.records[layer_index].as_ref(),
    ) {
        let region = prelude.region_options(input);
        super::ironing::append(
            &mut output,
            super::ironing::Input {
                record,
                layer_slices: &object_slices[layer_index],
                region,
                object: object_options,
                nozzles: &traversal.resolved.views.full.project.print.nozzle_diameter,
                layer,
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
) -> crate::geometry::BoundingBox {
    let mut points = slices
        .iter()
        .flatten()
        .flat_map(|expolygon| expolygon.contour().points());
    let first = *points
        .next()
        .expect("a sliced print object has a nonempty contour");
    let (minimum, maximum) = points.fold((first, first), |(minimum, maximum), point| {
        (
            crate::geometry::Point::new(minimum.x().min(point.x()), minimum.y().min(point.y())),
            crate::geometry::Point::new(maximum.x().max(point.x()), maximum.y().max(point.y())),
        )
    });
    crate::geometry::BoundingBox::new(minimum, maximum)
}
