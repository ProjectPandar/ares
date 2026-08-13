use crate::{
    SliceError,
    geometry::ClipperError,
    project_slice::prepare_infill::horizontal_shell_propagation::{
        self, PreparedPostHorizontalShellPropagation,
    },
};

use super::{parameters::ProcessExternalSurfacesConfig, process::process_external_surfaces};

pub(in crate::project_slice) struct PreparedPostExternalSurfaces {
    pub(in crate::project_slice) predecessor: PreparedPostHorizontalShellPropagation,
}

pub(in crate::project_slice) fn prepare(
    mut predecessor: PreparedPostHorizontalShellPropagation,
) -> Result<PreparedPostExternalSurfaces, SliceError> {
    if let Err(error) = process_records(&mut predecessor) {
        horizontal_shell_propagation::dispose(predecessor);
        return Err(geometry_error(error));
    }
    Ok(PreparedPostExternalSurfaces { predecessor })
}

pub(in crate::project_slice) fn dispose(prepared: PreparedPostExternalSurfaces) {
    horizontal_shell_propagation::dispose(prepared.predecessor);
}

fn process_records(
    predecessor: &mut PreparedPostHorizontalShellPropagation,
) -> Result<(), ClipperError> {
    let scale = predecessor.predecessor.scale;
    let spiral_mode = predecessor
        .predecessor
        .resolved
        .views
        .full
        .process
        .print
        .spiral_mode
        .0;

    for (object, traversal) in predecessor
        .objects
        .iter_mut()
        .zip(&predecessor.predecessor.objects)
    {
        let prelude = &traversal.predecessor.predecessor.predecessor.predecessor;
        let (_, inputs) = prelude.object.as_parts();
        for ((record, classic), input) in
            object.records.iter_mut().zip(&prelude.records).zip(inputs)
        {
            match (record, classic, input) {
                (Some(record), Some(classic), Some(input)) => {
                    let options = prelude.object.region_options(input);
                    process_external_surfaces(
                        &mut record.fill_surfaces,
                        ProcessExternalSurfacesConfig {
                            wall_loops: options.wall_loops.0,
                            perimeter_spacing: classic.perimeter_spacing,
                            external_width: classic.external_width,
                            external_spacing: classic.external_spacing,
                            solid_infill_spacing: classic.solid_infill_spacing,
                            bridge_angle_degrees: options.bridge_angle.0,
                            relative_bridge_angle: options.relative_bridge_angle.0,
                            model_rotation_radians: input.model_rotation_rad,
                            sparse_infill_density_percent: options.sparse_infill_density.0,
                            minimum_sparse_infill_area_mm2: options.minimum_sparse_infill_area.0,
                            spiral_mode,
                            scale,
                        },
                    )?;
                }
                (None, None, None) => {}
                _ => unreachable!("validated O26 record slots remain aligned"),
            }
        }
    }
    Ok(())
}

fn geometry_error(error: ClipperError) -> SliceError {
    match error {
        ClipperError::CoordinateOutOfRange => SliceError::InvalidInput(
            "external-surface polygon coordinate is outside the supported Clipper range".to_owned(),
        ),
        ClipperError::OpenPathMustBeSubject | ClipperError::OpenPathsRequirePolyTree => {
            unreachable!("external-surface open paths use subject input and PolyTree output")
        }
    }
}
