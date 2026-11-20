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
        project_settings: settings.clone(),
    })
}
