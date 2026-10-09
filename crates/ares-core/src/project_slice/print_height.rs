//! Print.cpp:1354-1396: validate generated object-layer upper bounds.

use crate::{Project, SliceError, project::effective_config::types::BoundedResolvedProjectConfig};

pub(crate) fn validate(
    project: &Project,
    resolved: &BoundedResolvedProjectConfig,
    plate_index: u32,
) -> Result<(), SliceError> {
    let settings = &resolved.views.full;
    let height = settings.printer.remaining.printable_height.0;
    // libslic3r.h:52; unlike BuildVolume's sentinel, zero stays literal here.
    const EPSILON: f64 = 1e-4;
    for object in super::planning::plan_project(project, resolved)? {
        let Some(last) = object.layers.last() else {
            continue;
        };
        let upper = last.print_z - object.slicing_parameters.object_print_z_min;
        let shrinkage = object.slicing_parameters.object_shrinkage_compensation_z;
        let name = project.objects()[object.source_object_index].name();
        let message = if shrinkage != 1.0 && upper > height / shrinkage + EPSILON {
            format!(
                "While the object {name} itself fits the build volume, it exceeds the maximum build volume height because of material shrinkage compensation."
            )
        } else if upper > height + EPSILON {
            if upper - 0.5 * last.height > height + EPSILON {
                format!("The object {name} exceeds the maximum build volume height.")
            } else {
                format!(
                    "While the object {name} itself fits the build volume, its last layer exceeds the maximum build volume height. You might want to reduce the size of your model or change current print settings and retry."
                )
            }
        } else {
            continue;
        };
        return Err(SliceError::Validate {
            message,
            plate_index,
            layer_height: f64::from(settings.process.object.layer_height.0 as f32),
            wall_loops: settings.process.region.wall_loops.0,
            sparse_infill_density: f64::from(
                settings.process.region.sparse_infill_density.0 as f32,
            ),
        });
    }
    Ok(())
}
