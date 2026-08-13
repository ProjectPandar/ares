use crate::{SliceError, project_slice::prepare_infill::bridge_over_infill::transaction};

#[cfg(test)]
thread_local! {
    static INVOCATIONS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    static DISPOSALS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

pub(in crate::project_slice) struct PreparedPostInfillCombination {
    pub(in crate::project_slice) predecessor: transaction::PreparedPostBridgeOverInfill,
}

pub(in crate::project_slice) fn prepare(
    predecessor: transaction::PreparedPostBridgeOverInfill,
) -> Result<PreparedPostInfillCombination, SliceError> {
    #[cfg(test)]
    INVOCATIONS.with(|count| count.set(count.get() + 1));

    let has_active_combination = predecessor
        .predecessor
        .predecessor
        .predecessor
        .objects
        .iter()
        .any(|object| {
            let prelude = &object
                .predecessor
                .predecessor
                .predecessor
                .predecessor
                .object;
            let (compensated, _) = prelude.as_parts();
            let (post_regions, _) = compensated.as_parts();
            let (_, _, regions) = post_regions.as_parts();
            regions.iter().any(|region| {
                let options = region.as_parts().1;
                options.infill_combination.0 && options.sparse_infill_density.0 != 0.0
            })
        });
    if has_active_combination {
        transaction::dispose(predecessor);
        return Err(SliceError::UnsupportedProjectFeature(
            "infill_combination".to_owned(),
        ));
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
