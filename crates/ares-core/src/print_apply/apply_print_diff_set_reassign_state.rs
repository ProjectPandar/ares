use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct StagedPrintDiffSetReassign {
    pub(super) original_print_diff_len: usize,
    pub(super) print_diff_set_len: usize,
    pub(super) reassigned: bool,
    pub(super) resulting_print_diff: Vec<&'static str>,
}

pub(super) fn staged_apply_print_diff_set_reassign(
    print_diff: &[&'static str],
    print_diff_set_keys: &[&'static str],
) -> StagedPrintDiffSetReassign {
    let print_diff_set = print_diff_set_keys.iter().copied().collect::<BTreeSet<_>>();
    let reassigned = print_diff_set.len() != print_diff.len();
    let resulting_print_diff = if reassigned {
        print_diff_set.iter().copied().collect()
    } else {
        print_diff.to_vec()
    };

    StagedPrintDiffSetReassign {
        original_print_diff_len: print_diff.len(),
        print_diff_set_len: print_diff_set.len(),
        reassigned,
        resulting_print_diff,
    }
}
