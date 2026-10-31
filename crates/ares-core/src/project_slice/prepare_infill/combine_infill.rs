use crate::{SliceError, project_slice::prepare_infill::bridge_over_infill::transaction};

mod process;

pub(in crate::project_slice) struct PreparedPostInfillCombination {
    pub(in crate::project_slice) predecessor: transaction::PreparedPostBridgeOverInfill,
}

pub(in crate::project_slice) fn prepare(
    mut predecessor: transaction::PreparedPostBridgeOverInfill,
) -> Result<PreparedPostInfillCombination, SliceError> {
    let horizontal = &mut predecessor.predecessor.predecessor;
    let traversal = &horizontal.predecessor;
    let nozzles = &traversal.resolved.views.full.project.print.nozzle_diameter;
    let scale = traversal.scale;
    let result = horizontal
        .objects
        .iter_mut()
        .zip(&traversal.objects)
        .try_for_each(|(object, traversal)| process::apply(object, traversal, nozzles, scale));
    if let Err(error) = result {
        transaction::dispose(predecessor);
        return Err(error);
    }
    Ok(PreparedPostInfillCombination { predecessor })
}

pub(in crate::project_slice) fn dispose(prepared: PreparedPostInfillCombination) {
    transaction::dispose(prepared.predecessor);
}
