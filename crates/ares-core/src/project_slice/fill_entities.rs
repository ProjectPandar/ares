mod concentric;
mod crosshatch;
mod cubic;
mod gap_residual;
mod grid;
mod gyroid;
mod monotonic;
mod simplify;
mod triangles;
mod types;

pub(in crate::project_slice) use types::{
    FillExtrusionCollection, FillExtrusionEntity, FillExtrusionPath, LayerFillEntities,
};

use crate::{
    ExtrusionRole, ProcessInfillPattern, SliceError,
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
                let (plan, _, _) = post_regions.as_parts();
                (0..plan.layers.len())
                    .map(|layer_index| generate_layer(external, object_index, layer_index))
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

fn materialized_flow(
    params: crate::project_slice::group_fills::SurfaceFillParams,
    spacing: f32,
) -> crate::project_slice::perimeters::types::Flow {
    if params.extrusion_role == ExtrusionRole::InternalInfill && !params.bridge {
        params.flow
    } else {
        crate::project_slice::perimeters::flow::with_spacing(params.flow, spacing)
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

pub(in crate::project_slice) fn generate_layer(
    prepared: &PreparedPostExternalSurfaces,
    object_index: usize,
    layer_index: usize,
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
    let (post_regions, _) = compensated.as_parts();
    let (plan, _, _) = post_regions.as_parts();
    let layer = &plan.layers[layer_index];
    let mut output = LayerFillEntities::default();

    for fill in grouped.surface_fills {
        match fill.params.pattern {
            SurfaceFillPattern::Configured(ProcessInfillPattern::CrossHatch) => {
                crosshatch::append(&mut output, fill, layer.print_z, traversal.scale)?;
            }
            SurfaceFillPattern::Configured(ProcessInfillPattern::Grid) => {
                grid::append(&mut output, fill, layer.id, traversal.scale)?;
            }
            SurfaceFillPattern::Configured(ProcessInfillPattern::Cubic) => {
                cubic::append(&mut output, fill, layer.print_z, traversal.scale)?;
            }
            SurfaceFillPattern::Configured(ProcessInfillPattern::Triangles) => {
                triangles::append(&mut output, fill, traversal.scale)?;
            }
            SurfaceFillPattern::Configured(ProcessInfillPattern::Gyroid) => {
                gyroid::append(&mut output, fill, layer.print_z, traversal.scale)?;
            }
            SurfaceFillPattern::Configured(
                pattern @ (ProcessInfillPattern::Rectilinear
                | ProcessInfillPattern::ZigZag
                | ProcessInfillPattern::Monotonic
                | ProcessInfillPattern::MonotonicLine),
            ) => {
                monotonic::append(&mut output, fill, pattern, layer.id, traversal.scale)?;
            }
            SurfaceFillPattern::ConcentricInternal => {
                concentric::append(&mut output, fill, minimum_nozzle_diameter, traversal.scale)?;
            }
            SurfaceFillPattern::Configured(_) => {}
        }
    }
    let process = &traversal.resolved.views.full.process;
    simplify::apply(
        &mut output,
        process.print.resolution.0.max(1.0e-4),
        process.gcode.enable_arc_fitting.0,
        traversal.scale,
    );
    Ok(output)
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
