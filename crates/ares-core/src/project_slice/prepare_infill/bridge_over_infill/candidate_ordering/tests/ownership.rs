use std::collections::BTreeMap;

use super::{candidate, ids, rectangle};
use crate::project_slice::prepare_infill::bridge_over_infill::{
    candidate_ordering::order_candidate_surfaces, types::CandidateSurface,
};

#[test]
fn task22o55_moves_every_candidate_with_fields_and_polygon_allocations_intact() {
    let input = fixture();
    let before = input
        .iter()
        .map(|candidate| {
            (
                candidate.source.surface_index,
                (
                    candidate.source,
                    candidate.bridge_angle.to_bits(),
                    candidate.new_polygons.as_ptr() as usize,
                    candidate
                        .new_polygons
                        .iter()
                        .map(|polygon| polygon.points().as_ptr() as usize)
                        .collect::<Vec<_>>(),
                    candidate
                        .new_polygons
                        .iter()
                        .map(|polygon| {
                            polygon
                                .points()
                                .iter()
                                .map(|point| (point.x(), point.y()))
                                .collect::<Vec<_>>()
                        })
                        .collect::<Vec<_>>(),
                ),
            )
        })
        .collect::<BTreeMap<_, _>>();

    let ordered = order_candidate_surfaces(input);
    assert_eq!(ids(&ordered), vec![1, 0, 2, 3]);
    for candidate in &ordered {
        let expected = &before[&candidate.source.surface_index];
        assert_eq!(candidate.source, expected.0);
        assert_eq!(candidate.bridge_angle.to_bits(), expected.1);
        assert_eq!(candidate.new_polygons.as_ptr() as usize, expected.2);
        assert_eq!(
            candidate
                .new_polygons
                .iter()
                .map(|polygon| polygon.points().as_ptr() as usize)
                .collect::<Vec<_>>(),
            expected.3
        );
        assert_eq!(
            candidate
                .new_polygons
                .iter()
                .map(|polygon| {
                    polygon
                        .points()
                        .iter()
                        .map(|point| (point.x(), point.y()))
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>(),
            expected.4
        );
    }
}

#[test]
fn task22o55_independently_identical_owned_inputs_are_repeatable() {
    let first = order_candidate_surfaces(fixture());
    let second = order_candidate_surfaces(fixture());
    assert_eq!(ids(&first), ids(&second));
    assert_eq!(snapshot(&first), snapshot(&second));
}

fn fixture() -> Vec<CandidateSurface> {
    vec![
        candidate(0, vec![rectangle(20, 0, 30, 10), rectangle(21, 1, 29, 9)]),
        candidate(1, vec![rectangle(0, 0, 100, 100)]),
        candidate(2, vec![rectangle(15, 0, 25, 10)]),
        candidate(3, vec![rectangle(5, 0, 15, 10)]),
    ]
}

type CandidateSnapshot = (usize, u64, Vec<Vec<(i64, i64)>>);

fn snapshot(candidates: &[CandidateSurface]) -> Vec<CandidateSnapshot> {
    candidates
        .iter()
        .map(|candidate| {
            (
                candidate.source.surface_index,
                candidate.bridge_angle.to_bits(),
                candidate
                    .new_polygons
                    .iter()
                    .map(|polygon| {
                        polygon
                            .points()
                            .iter()
                            .map(|point| (point.x(), point.y()))
                            .collect()
                    })
                    .collect(),
            )
        })
        .collect()
}
