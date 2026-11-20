use super::{brim, footprint, layers};
use crate::SliceError;
use crate::project_slice::ArchivePlate;

pub(super) fn facts(
    traversal: &crate::project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal,
    schedule: &layers::Schedule,
    brim: &[Option<brim::BrimPlan>],
) -> Result<ArchivePlate, SliceError> {
    let footprint = footprint::export_object_footprint(
        traversal,
        schedule,
        brim.first().and_then(Option::as_ref),
    )
    .ok_or_else(|| {
        SliceError::InvalidInput("sliced 3MF requires one printable object instance".into())
    })?;
    let settings = &traversal.project.documents().project_settings_raw;
    let setting = |key: &str| {
        settings
            .get(key)
            .and_then(|value| value.element(0))
            .and_then(crate::project::raw_settings::RawConfigValue::token)
            .unwrap_or_default()
    };
    let nozzle_diameter = setting("nozzle_diameter").parse().unwrap_or(0.0);
    Ok(ArchivePlate {
        object_name: traversal.project.objects()[0].name().to_owned(),
        plate_id: traversal.project.objects()[0].id(),
        identify_id: schedule.instances[0][0].label_id,
        bounds: footprint.bounds,
        area: footprint.area,
        layer_height: footprint.layer_height,
        layer_count: schedule.layer_count(),
        filament_color: setting("filament_colour"),
        filament_type: setting("filament_type"),
        filament_id: setting("filament_ids"),
        nozzle_diameter,
        project_settings: saved_settings(settings),
    })
}

fn saved_settings(
    settings: &crate::project::raw_settings::ProjectSettingsRaw,
) -> crate::project::raw_settings::ProjectSettingsRaw {
    let mut settings = settings.clone();
    // OrcaSlicer.cpp:3723–3728 fills the per-filament compatibility vector
    // from the loaded filament preset; its unset condition is an empty string.
    if !settings.contains_key("compatible_process_expression_group") {
        settings.insert_strings("compatible_process_expression_group", &[String::new()]);
    }
    // PrintConfig.cpp:3275–3281, OrcaSlicer.cpp:4250–4256: the default
    // material-flow vector is present even for an ordinary filament printer.
    if !settings.contains_key("pellet_flow_coefficient") {
        let key = "pellet_flow_coefficient";
        let default = crate::option_definition(key).unwrap();
        settings.insert_strings(key, &[default.default_wire_value.to_owned()]);
    }
    settings
}
