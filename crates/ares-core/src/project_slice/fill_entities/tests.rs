use super::simplify_before_ordering;

#[test]
fn arc_fitting_owns_the_single_non_spiral_simplification_pass() {
    assert!(!simplify_before_ordering(true, false));
    assert!(simplify_before_ordering(false, false));
    assert!(simplify_before_ordering(true, true));
}
