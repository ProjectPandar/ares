use super::super::{Point, chain_points};

mod kd_tree;
mod priority_queue;

#[test]
fn task22m_chain_points_handles_empty_and_singleton() {
    assert!(chain_points(&[]).is_empty());
    assert_eq!(chain_points(&[Point::new(7, -3)]), [0]);
}

#[test]
fn task22m_chain_points_matches_fixed_multifragment_orders() {
    for (points, expected) in [
        (
            vec![(10, 0), (0, 0), (20, 0), (0, 10), (10, 10)],
            vec![1, 3, 4, 0, 2],
        ),
        (vec![(0, 0), (100, 0), (10, 0), (20, 0)], vec![1, 3, 2, 0]),
        (vec![(0, 0), (10, 0), (0, 10), (10, 10)], vec![0, 1, 3, 2]),
        (vec![(0, 0), (0, 0), (10, 0), (0, 10)], vec![2, 0, 1, 3]),
        (
            vec![(-20, -5), (-5, -5), (-10, -10), (0, -5)],
            vec![0, 2, 1, 3],
        ),
        (
            vec![
                (i64::MIN, 0),
                (i64::MAX, 0),
                (0, 0),
                (9_223_372_036_854_770_000, 1),
            ],
            vec![0, 2, 3, 1],
        ),
    ] {
        let points = points
            .into_iter()
            .map(|(x, y)| Point::new(x, y))
            .collect::<Vec<_>>();
        assert_eq!(chain_points(&points), expected);
    }
}
