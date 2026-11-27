use crate::{
    Project, SliceError,
    geometry::CoordinateScale,
    project::effective_config::types::{BoundedResolvedProjectConfig, ResolvedProjectObject},
};

use super::{
    belt::{BeltRotation, BeltSlice},
    bounds, capabilities, extruders, layers, parameters, profile,
};

pub(super) fn plan_project(
    project: &Project,
    resolved: &BoundedResolvedProjectConfig,
    scale: CoordinateScale,
) -> Result<Vec<layers::PlannedPrintObject>, SliceError> {
    capabilities::validate(project.objects(), &resolved.objects)?;
    let object_extruders = extruders::collect_project_object_extruders(
        project.objects(),
        &resolved.objects,
        resolved.logical_filament_count,
    );
    let rotation = BeltRotation::from_printer(&resolved.views.full.printer.remaining);
    let mut budget = layers::LayerBudget::default();
    let mut planned_objects = Vec::new();
    for (object_index, resolved_object) in resolved.objects.iter().enumerate() {
        if resolved_object.print_objects.is_empty() {
            continue;
        }
        let source = &project.objects()[resolved_object.source_object_index];
        let prepare = |height| -> Result<_, SliceError> {
            let parameters = parameters::slicing_parameters(
                &resolved.views.full,
                &resolved_object.object,
                height,
                &object_extruders[object_index],
            )?;
            let profile = profile::update_layer_height_profile(
                &parameters,
                source.layer_height_profile(),
                source.layer_config_ranges(),
            )?;
            Ok((parameters, profile))
        };
        let flat = rotation
            .is_none()
            .then(|| prepare(bounds::object_height(source)?))
            .transpose()?;
        for (transform_index, group) in resolved_object.print_objects.iter().enumerate() {
            let belt_slice = rotation
                .map(|rotation| BeltSlice::new(rotation, source, group, scale))
                .transpose()?;
            let belt = belt_slice.map(|slice| prepare(slice.height)).transpose()?;
            let (parameters, profile) = belt
                .as_ref()
                .or(flat.as_ref())
                .expect("group retains a layer profile");
            planned_objects.push(layers::plan_print_object(
                (resolved_object.source_object_index, transform_index),
                parameters,
                profile,
                layers::LayerPlanOptions {
                    precise_z_height: resolved_object.object.precise_z_height.0,
                    zaa_min_z: zaa_min_z(resolved_object),
                    belt_slice,
                },
                &mut budget,
            )?);
        }
    }
    Ok(planned_objects)
}

fn zaa_min_z(object: &ResolvedProjectObject) -> Option<f64> {
    // PrintObjectSlice.cpp:24-47 and Print.hpp:423-424 include all printing regions.
    object
        .all_regions
        .iter()
        .filter(|region| region.zaa_enabled.0)
        .map(|region| region.zaa_min_z.0)
        .reduce(f64::min)
}
