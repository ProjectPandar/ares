pub(in crate::project_slice) mod adaptive;
mod concentric;
mod crosshatch;
mod cubic;
mod flow;
mod gap_residual;
mod grid;
mod gyroid;
mod honeycomb;
mod ironing;
mod lateral_honeycomb;
mod lateral_lattice;
mod lightning;
mod line;
mod locked_zag;
mod monotonic;
mod plane_path;
mod quartercubic;
mod simplify;
mod stars;
mod tpms_d;
mod tpms_fk;

pub(in crate::project_slice) use flow::{materialized_flow, object_center};

#[cfg(test)]
mod tests;
mod three_d_honeycomb;
mod triangles;
mod types;

pub(in crate::project_slice) use types::{
    FillExtrusionCollection, FillExtrusionEntity, FillExtrusionPath, LayerFillEntities,
};

use crate::{
    ExtrusionRole, ProcessInfillPattern, SliceError,
    geometry::{ExPolygon, Point},
    project_slice::{
        group_fills::{SurfaceFillPattern, group_fills},
        prepare_infill::{
            combine_infill::{self, PreparedPostInfillCombination},
            external_surfaces::PreparedPostExternalSurfaces,
        },
    },
};

#[cfg(test)]
thread_local! {
    static INVOCATIONS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    static DISPOSALS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

pub(in crate::project_slice) struct PreparedPostFillEntities {
    pub(in crate::project_slice) predecessor: PreparedPostInfillCombination,
    pub(in crate::project_slice) objects: Vec<Vec<LayerFillEntities>>,
}

pub(in crate::project_slice) fn prepare(
    mut predecessor: PreparedPostInfillCombination,
) -> Result<PreparedPostFillEntities, SliceError> {
    #[cfg(test)]
    INVOCATIONS.with(|count| count.set(count.get() + 1));
    let result = {
        let external = &predecessor.predecessor.predecessor;
        let traversal = &external.predecessor.predecessor;
        traversal
            .objects
            .iter()
            .enumerate()
            .map(|(object_index, object)| {
                let prelude = &object
                    .predecessor
                    .predecessor
                    .predecessor
                    .predecessor
                    .object;
                let (compensated, _) = prelude.as_parts();
                let (post_regions, _) = compensated.as_parts();
                let (plan, _, regions) = post_regions.as_parts();
                // `FillLightning` needs a per-object `Generator` built from ALL
                // layers' internal surfaces (built once, like upstream
                // `prepare_lightning_infill_data`, `PrintObject.cpp:1010`).
                let resolved_object = traversal
                    .resolved
                    .objects
                    .iter()
                    .find(|o| o.source_object_index == prelude.identity().0);
                let nozzle = traversal
                    .resolved
                    .views
                    .full
                    .project
                    .print
                    .nozzle_diameter
                    .0
                    .iter()
                    .map(|v| v.0)
                    .reduce(f64::min)
                    .unwrap_or(0.4);
                let (layer_height, sparse_width) = resolved_object
                    .map(|o| {
                        let h = plan.layers.first().map(|l| l.height).unwrap_or(0.2);
                        let region = regions.iter().find(|r| {
                            r.as_parts().1.sparse_infill_pattern == ProcessInfillPattern::Lightning
                        });
                        let w = region
                            .map(|r| {
                                crate::project_slice::gcode_emit::resolved_width(
                                    r.as_parts().1.sparse_infill_line_width,
                                    o.object.line_width,
                                    nozzle,
                                    1.125,
                                ) as f64
                            })
                            .unwrap_or(nozzle);
                        (h, w)
                    })
                    .unwrap_or((0.2, nozzle));
                let lightning_generator = build_lightning_generator(
                    regions,
                    plan.layers.len(),
                    layer_height,
                    sparse_width,
                    traversal.scale,
                )?;
                (0..plan.layers.len())
                    .map(|layer_index| {
                        generate_layer(
                            external,
                            object_index,
                            layer_index,
                            lightning_generator.as_ref(),
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()
            })
            .collect::<Result<Vec<_>, _>>()
    };
    match result {
        Ok(mut objects) => {
            move_thin_fills(&mut objects, &mut predecessor.predecessor.predecessor);
            Ok(PreparedPostFillEntities {
                predecessor,
                objects,
            })
        }
        Err(error) => {
            combine_infill::dispose(predecessor);
            Err(error)
        }
    }
}

fn move_thin_fills(
    objects: &mut [Vec<LayerFillEntities>],
    external: &mut PreparedPostExternalSurfaces,
) {
    for (output, source) in objects.iter_mut().zip(&mut external.predecessor.objects) {
        for (layer, record) in output.iter_mut().zip(&mut source.records) {
            if let Some(record) = record {
                layer.perimeters = std::mem::take(&mut record.perimeters);
                layer
                    .thin_fills
                    .extend(std::mem::take(&mut record.thin_fills));
            }
        }
    }
}

pub(in crate::project_slice) fn dispose(prepared: PreparedPostFillEntities) {
    #[cfg(test)]
    DISPOSALS.with(|count| count.set(count.get() + 1));
    combine_infill::dispose(prepared.predecessor);
}

#[cfg(test)]
pub(in crate::project_slice) fn reset_hooks() {
    INVOCATIONS.with(|count| count.set(0));
    DISPOSALS.with(|count| count.set(0));
}

#[cfg(test)]
pub(in crate::project_slice) fn invocations() -> usize {
    INVOCATIONS.with(std::cell::Cell::get)
}

#[cfg(test)]
pub(in crate::project_slice) fn disposals() -> usize {
    DISPOSALS.with(std::cell::Cell::get)
}

/// `prepare_lightning_infill_data` (`PrintObject.cpp:1010`) — build the
/// per-object `FillLightning::Generator` when any region uses the `lightning`
/// sparse pattern. Returns `None` when no region uses lightning. The
/// `layer_internal_outlines` are the `Internal`/`InternalVoid` region surfaces
/// (`stInternal`/`stInternalVoid` fill surfaces, `Generator.cpp:117`).
fn build_lightning_generator(
    regions: &[crate::project_slice::region_slices::PostRegion],
    layer_count: usize,
    layer_height: f64,
    sparse_infill_line_width: f64,
    scale: crate::geometry::CoordinateScale,
) -> Result<Option<crate::fill::lightning::generator::Generator>, SliceError> {
    use crate::project_slice::region_slices::RegionSurfaceKind;
    // Any region that uses `lightning` as its sparse_infill_pattern supplies
    // the generator params (upstream uses a single per-object generator — the
    // first lightning region's params apply to the whole object).
    let Some(lightning_region) = regions
        .iter()
        .find(|r| r.as_parts().1.sparse_infill_pattern == ProcessInfillPattern::Lightning)
    else {
        return Ok(None);
    };
    let options = lightning_region.as_parts().1;
    let layer_internal_outlines: Vec<Vec<crate::geometry::Polygon>> = (0..layer_count)
        .map(|layer_id| {
            regions
                .iter()
                .flat_map(|region| {
                    region
                        .as_parts()
                        .2
                        .get(layer_id)
                        .map(|layer| layer.surfaces())
                        .unwrap_or(&[])
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
                })
                .collect()
        })
        .collect();
    crate::fill::lightning::generator::Generator::new(
        layer_internal_outlines,
        layer_height,
        sparse_infill_line_width,
        options.fill_multiline.0,
        options.sparse_infill_density.0,
        options.lightning_overhang_angle.0,
        options.lightning_prune_angle.0,
        options.lightning_straightening_angle.0,
        scale,
    )
    .map(Some)
    .map_err(|e| SliceError::InvalidInput(format!("lightning generator: {e:?}")))
}

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
    let object_reference = object_center(object_slices);
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
                crosshatch::append(&mut output, fill, layer.print_z, traversal.scale)?;
            }
            SurfaceFillPattern::Configured(ProcessInfillPattern::Grid) => {
                grid::append(
                    &mut output,
                    fill,
                    layer.id,
                    traversal.scale,
                    object_reference,
                )?;
            }
            SurfaceFillPattern::Configured(ProcessInfillPattern::Cubic) => {
                cubic::append(
                    &mut output,
                    fill,
                    layer.print_z,
                    traversal.scale,
                    object_reference,
                )?;
            }
            SurfaceFillPattern::Configured(ProcessInfillPattern::Triangles) => {
                triangles::append(&mut output, fill, traversal.scale, object_reference)?;
            }
            SurfaceFillPattern::Configured(ProcessInfillPattern::Gyroid) => {
                gyroid::append(&mut output, fill, layer.print_z, traversal.scale)?;
            }
            SurfaceFillPattern::Configured(
                pattern
                @ (ProcessInfillPattern::AdaptiveCubic | ProcessInfillPattern::SupportCubic),
            ) => {
                let _ = pattern;
                adaptive::append(
                    &mut output,
                    fill,
                    layer.print_z,
                    traversal,
                    object_index,
                    traversal.scale,
                )?;
            }
            SurfaceFillPattern::Configured(ProcessInfillPattern::ThreeDHoneycomb) => {
                three_d_honeycomb::append(&mut output, fill, layer.print_z, traversal.scale)?;
            }
            SurfaceFillPattern::Configured(ProcessInfillPattern::Honeycomb) => {
                honeycomb::append(&mut output, fill, layer.id, traversal.scale)?;
            }
            SurfaceFillPattern::Configured(ProcessInfillPattern::Line) => {
                line::append(
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
                monotonic::append(&mut output, fill, pattern, layer.id, traversal.scale)?;
            }
            SurfaceFillPattern::Configured(ProcessInfillPattern::TriHexagon) => {
                // `tri-hexagon` maps to `ipStars` (`PrintConfig.cpp:238`).
                stars::append(&mut output, fill, traversal.scale, object_reference)?;
            }
            SurfaceFillPattern::Configured(ProcessInfillPattern::TpmsFk) => {
                tpms_fk::append(&mut output, fill, layer.print_z, traversal.scale)?;
            }
            SurfaceFillPattern::Configured(ProcessInfillPattern::TpmsD) => {
                tpms_d::append(&mut output, fill, layer.print_z, traversal.scale)?;
            }
            SurfaceFillPattern::Configured(ProcessInfillPattern::LateralLattice) => {
                lateral_lattice::append(
                    &mut output,
                    fill,
                    layer.print_z,
                    layer.id,
                    traversal.scale,
                    object_reference,
                )?;
            }
            SurfaceFillPattern::Configured(ProcessInfillPattern::LateralHoneycomb) => {
                lateral_honeycomb::append(
                    &mut output,
                    fill,
                    layer.print_z,
                    layer.id,
                    traversal.scale,
                    object_reference,
                )?;
            }
            SurfaceFillPattern::Configured(ProcessInfillPattern::QuarterCubic) => {
                quartercubic::append(
                    &mut output,
                    fill,
                    layer.print_z,
                    traversal.scale,
                    object_reference,
                )?;
            }
            SurfaceFillPattern::Configured(ProcessInfillPattern::LockedZag) => {
                locked_zag::append(
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
                    plane_path::append(
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
                concentric::append(&mut output, fill, minimum_nozzle_diameter, traversal.scale)?;
            }
            SurfaceFillPattern::Configured(ProcessInfillPattern::Lightning) => {
                if let Some(generator) = lightning_generator {
                    lightning::append(&mut output, generator, &fill, layer.id, traversal.scale)?;
                }
            }
            SurfaceFillPattern::Configured(ProcessInfillPattern::Concentric) => {
                concentric::append_configured(
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
        ironing::append(
            &mut output,
            ironing::Input {
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
        simplify::apply(
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

fn simplify_before_ordering(enable_arc_fitting: bool, spiral_mode: bool) -> bool {
    !enable_arc_fitting || spiral_mode
}

fn geometry_error(error: crate::geometry::ClipperError) -> SliceError {
    match error {
        crate::geometry::ClipperError::CoordinateOutOfRange => SliceError::InvalidInput(
            "fill generation coordinate is outside the supported Clipper range".to_owned(),
        ),
        crate::geometry::ClipperError::OpenPathMustBeSubject
        | crate::geometry::ClipperError::OpenPathsRequirePolyTree => {
            unreachable!("fill generators use valid open subjects and closed clips")
        }
    }
}
