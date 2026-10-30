//! Arrangement invariants through the public `arrangement::arrange`: whatever the squares, each
//! lands on a bed inside its outline, no two on one bed share area, and the same input gives the
//! same placement (Arrange.cpp has no random stream; the nester's evaluation order is fixed).

use crate::support::{MM, assert_disjoint, bed, inside, params, placed, square};
use ares_core::arrangement::arrange;
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(24))]

    #[test]
    fn arranged_squares_fit_their_bed_and_never_overlap(sides in proptest::collection::vec(5i64..90, 1..10)) {
        let mut items: Vec<_> = sides.iter().map(|side| square(side * MM)).collect();
        arrange(&mut items, &[], &bed(200, 200), &params(2 * MM));
        let mut again: Vec<_> = sides.iter().map(|side| square(side * MM)).collect();
        arrange(&mut again, &[], &bed(200, 200), &params(2 * MM));
        prop_assert_eq!(&items, &again);
        let outline = ares_core::arrangement::BoundingBox::new(
            ares_core::arrangement::Point::new(0, 0),
            ares_core::arrangement::Point::new(200 * MM, 200 * MM),
        );
        for item in &items {
            prop_assert!(item.bed_idx >= 0);
            prop_assert!(inside(outline, placed(item)));
        }
        let beds = items.iter().map(|item| item.bed_idx).max().unwrap();
        for bed_idx in 0..=beds {
            let on_bed: Vec<_> = items.iter().filter(|item| item.bed_idx == bed_idx).cloned().collect();
            assert_disjoint(&on_bed);
        }
    }
}
