use crate::SliceError;
use crate::project_slice::{
    ArchivePlate, perimeters::classic::traversal::PreparedPostClassicTraversal,
};

use super::{brim, footprint, layers};

pub(super) fn archive_facts(
    traversal: &PreparedPostClassicTraversal,
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
    Ok(ArchivePlate {
        object_name: traversal.project.objects()[0].name().to_owned(),
        identify_id: schedule.instances[0][0].label_id,
        bounds: footprint.bounds,
        area: footprint.area,
        layer_height: footprint.layer_height,
        layer_count: schedule.layer_count(),
        project_settings: traversal.project.documents().project_settings_raw.clone(),
    })
}
