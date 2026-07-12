use super::super::apply_support_used_state::staged_apply_support_used;

#[test]
fn apply_support_used_queries_enable_support_key() {
    let assignment = staged_apply_support_used(Some(true));

    assert_eq!(assignment.queried_key(), "enable_support");
}

#[test]
fn apply_support_used_missing_option_sets_false() {
    let assignment = staged_apply_support_used(None);

    assert!(!assignment.support_used());
}

#[test]
fn apply_support_used_false_option_sets_false() {
    let assignment = staged_apply_support_used(Some(false));

    assert!(!assignment.support_used());
}

#[test]
fn apply_support_used_true_option_sets_true() {
    let assignment = staged_apply_support_used(Some(true));

    assert!(assignment.support_used());
}
