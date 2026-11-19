use super::{ArrangeScope, ArrangeSettings, PlaterSession, prepare};
use crate::{Project, SliceError, options::ProcessPrintSequence};

/// `OrcaSlicer.cpp:4938–4959,5299–5327,6095–6268`: repeat only the selected
/// plate, keeping the original model if the arrangement cannot add any copies.
pub(crate) fn repeat_cli_plate(
    project: Project,
    plate: Option<u32>,
    count: i32,
) -> Result<(Project, Option<usize>), SliceError> {
    if count <= 1 {
        return Ok((project, None));
    }
    let mut project = crate::plater::initialize_slice_plates(project);
    let id = plate
        .filter(|&id| id != 0)
        .ok_or(SliceError::InvalidRepetitionPlate)?;
    let index = project
        .plates()
        .iter()
        .position(|p| p.id() == id)
        .ok_or(SliceError::InvalidRepetitionPlate)?;
    project.prepare_import_scope(plate);
    let original = PlaterSession::with_project(Vec::new(), project);
    if original.project.settings().process.print.spiral_mode.0
        && prepare::real_print_sequence(&original.project, &original.plates, index)
            != ProcessPrintSequence::ByObject
    {
        return Ok((original.project, None));
    }
    let settings = ArrangeSettings {
        avoid_extrusion_cali_region: false,
        ..original.default_arrange_settings()
    };
    let mut low = 0;
    let mut high = (count - 1) as usize;
    let mut additional = high;
    while additional > 0 {
        let expanded = original.project.repeated_model(index, additional)?;
        let mut trial = PlaterSession::with_project(Vec::new(), expanded);
        trial.plates.plates[index].locked = false;
        if let Some(placed_count) = trial.arrange_job(
            ArrangeScope::CurrentPlate(index),
            settings,
            Some(additional),
        )? {
            if additional == high || additional + 1 == high {
                let edits = trial.edits();
                trial.project.apply_plater_edits(&edits);
                return Ok((trial.project, Some(placed_count)));
            }
            low = additional;
        } else {
            if additional == 1 {
                break;
            }
            high = additional;
        }
        additional = (low + high) / 2;
    }
    Ok((original.project, None))
}
