use std::cmp::Ordering;

use super::source_priority;

#[test]
fn source_priority_applies_overhang_embedding_then_score() {
    assert_eq!(
        source_priority((0.1, 0.0, 0.0), (0.2, -1.0, -1.0)),
        Ordering::Less
    );
    assert_eq!(
        source_priority((0.0, -0.6, 1.0), (0.0, 0.0, 0.0)),
        Ordering::Less
    );
    assert_eq!(
        source_priority((0.0, 0.0, 0.1), (0.0, 0.0, 0.2)),
        Ordering::Less
    );
    assert_eq!(
        source_priority((0.0, 0.0, 0.1), (0.0, 0.0, 0.1)),
        Ordering::Equal
    );
}

#[test]
fn stable_sort_preserves_equal_source_priority_order() {
    let mut scores = [
        (0, (0.0, 0.0, 0.2)),
        (1, (0.0, 0.0, 0.1)),
        (2, (0.0, 0.0, 0.1)),
    ];
    scores.sort_by(|left, right| source_priority(left.1, right.1));

    assert_eq!(scores.map(|entry| entry.0), [1, 2, 0]);
}
