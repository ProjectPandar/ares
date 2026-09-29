use crate::{SliceError, project_slice::prepare_infill::bridge_over_infill::transaction};

mod process;

#[cfg(test)]
thread_local! {
    static INVOCATIONS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    static DISPOSALS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

pub(in crate::project_slice) struct PreparedPostInfillCombination {
    pub(in crate::project_slice) predecessor: transaction::PreparedPostBridgeOverInfill,
}

pub(in crate::project_slice) fn prepare(
    mut predecessor: transaction::PreparedPostBridgeOverInfill,
) -> Result<PreparedPostInfillCombination, SliceError> {
    #[cfg(test)]
    INVOCATIONS.with(|count| count.set(count.get() + 1));

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
    #[cfg(test)]
    DISPOSALS.with(|count| count.set(count.get() + 1));
    transaction::dispose(prepared.predecessor);
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
