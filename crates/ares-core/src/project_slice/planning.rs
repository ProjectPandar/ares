use crate::{
    Project, SliceError,
    project::effective_config::types::{BoundedResolvedProjectConfig, ResolvedProjectObject},
};

use super::{bounds, capabilities, extruders, layers, parameters, profile};

pub(super) fn plan_project(
    project: &Project,
    resolved: &BoundedResolvedProjectConfig,
) -> Result<Vec<layers::PlannedPrintObject>, SliceError> {
    capabilities::validate(
        project.has_painted_layer_height_profile(),
        project.objects(),
        &resolved.objects,
        resolved.views.full.process.print.spiral_mode.0,
    )?;
    let object_extruders = extruders::collect_project_object_extruders(
        project.objects(),
        &resolved.objects,
        resolved.logical_filament_count,
    );
    plan_resolved_objects(&resolved.objects, |object_index, resolved_object| {
        let object_height = bounds::participating_object_heights(
            project.objects(),
            std::slice::from_ref(resolved_object),
        )?[0];
        parameters::slicing_parameters(
            &resolved.views.full,
            &resolved_object.object,
            object_height,
            &object_extruders[object_index],
        )
    })
}

pub(super) fn plan_resolved_objects(
    resolved_objects: &[ResolvedProjectObject],
    mut prepare: impl FnMut(
        usize,
        &ResolvedProjectObject,
    ) -> Result<parameters::SlicingParameters, SliceError>,
) -> Result<Vec<layers::PlannedPrintObject>, SliceError> {
    let mut budget = layers::LayerBudget::default();
    let mut planned_objects = Vec::new();
    for (object_index, resolved_object) in resolved_objects.iter().enumerate() {
        if resolved_object.print_objects.is_empty() {
            continue;
        }
        let parameters = prepare(object_index, resolved_object)?;
        let profile = profile::fixed_layer_height_profile(&parameters);
        for (transform_index, _) in resolved_object.print_objects.iter().enumerate() {
            planned_objects.push(layers::plan_print_object(
                resolved_object.source_object_index,
                transform_index,
                &parameters,
                &profile,
                &mut budget,
            )?);
        }
        let parameters::SlicingParameters {
            base_raft_layers,
            interface_raft_layers,
            base_raft_layer_height,
            interface_raft_layer_height,
            contact_raft_layer_height,
            layer_height,
            min_layer_height,
            max_layer_height,
            first_print_layer_height,
            first_object_layer_height,
            first_object_layer_bridging,
            gap_raft_object,
            gap_object_support,
            gap_support_object,
            raft_base_top_z,
            raft_interface_top_z,
            raft_contact_top_z,
            object_print_z_min,
            object_print_z_max,
            object_print_z_uncompensated_max,
            object_shrinkage_compensation_z,
        } = parameters;
        let _ = (
            base_raft_layers,
            interface_raft_layers,
            base_raft_layer_height,
            interface_raft_layer_height,
            contact_raft_layer_height,
            layer_height,
            min_layer_height,
            max_layer_height,
            first_print_layer_height,
            first_object_layer_height,
            first_object_layer_bridging,
            gap_raft_object,
            gap_object_support,
            gap_support_object,
            raft_base_top_z,
            raft_interface_top_z,
            raft_contact_top_z,
            object_print_z_min,
            object_print_z_max,
            object_print_z_uncompensated_max,
            object_shrinkage_compensation_z,
        );
    }
    Ok(planned_objects)
}
