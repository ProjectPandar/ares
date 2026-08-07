use super::*;
use crate::geometry::Point;
use crate::geometry::clipper::{ClipOperation, FillRule};

fn xyz(points: &[KernelPoint]) -> Vec<(i64, i64, i64)> {
    points
        .iter()
        .map(|point| (point.x(), point.y(), point.z))
        .collect()
}

#[test]
fn polygon_fixup_removes_duplicate_and_collinear_nodes_without_transferring_z() {
    let survivors = Clipper::fixup_survivors_for_test(&[
        KernelPoint::new(0, 0, 10),
        KernelPoint::new(5, 0, 20),
        KernelPoint::new(10, 0, 30),
        KernelPoint::new(10, 0, 99),
        KernelPoint::new(10, 10, 40),
        KernelPoint::new(0, 10, 50),
    ]);
    assert_eq!(
        xyz(&survivors),
        vec![(0, 0, 10), (10, 0, 99), (10, 10, 40), (0, 10, 50)]
    );
}

#[test]
fn duplicate_out_point_copies_complete_xyz() {
    let point = KernelPoint::new(-7, 13, -29);
    let (original, duplicate) = Clipper::duplicate_xyz_for_test(point);
    assert!(original.full_eq(point));
    assert!(duplicate.full_eq(point));
}

#[test]
fn horizontal_join_overwrite_and_followup_copy_use_replacement_xyz() {
    let replacement = KernelPoint::new(5, 0, 77);
    let (original, overwritten, duplicate) = Clipper::join_copy_overwrite_for_test(
        &[
            KernelPoint::new(0, 0, 11),
            KernelPoint::new(10, 0, 22),
            KernelPoint::new(10, 10, 33),
            KernelPoint::new(0, 10, 44),
        ],
        replacement,
    );
    assert!(original.full_eq(KernelPoint::new(0, 0, 11)));
    assert!(overwritten.full_eq(replacement));
    assert!(duplicate.full_eq(replacement));
}

#[test]
fn scanbeam_top_bottom_interior_and_promotion_keep_exact_z_rules() {
    let [top, bottom, interior, promoted] = crate::geometry::clipper::top_updates_for_test();
    assert_eq!((top.x(), top.y(), top.z), (10, 10, 20));
    assert_eq!((bottom.x(), bottom.y(), bottom.z), (0, 0, 10));
    assert_eq!((interior.x(), interior.y(), interior.z), (5, 5, 0));
    assert_eq!((promoted.x(), promoted.y(), promoted.z), (10, 10, 30));
}

#[test]
fn immediate_xy_dedup_retains_the_stored_complete_point() {
    let stored = KernelPoint::new(4, 9, 17);
    let duplicate = KernelPoint::new(4, 9, 81);
    let (survivor, same_node) = Clipper::immediate_dedup_for_test(stored, duplicate);
    assert!(same_node);
    assert!(survivor.full_eq(stored));
}

#[test]
fn open_fixup_removes_the_duplicate_without_transferring_z() {
    let survivors = Clipper::open_fixup_survivors_for_test(&[
        KernelPoint::new(0, 0, 10),
        KernelPoint::new(0, 0, 90),
        KernelPoint::new(10, 0, 20),
    ]);
    assert_eq!(xyz(&survivors), vec![(0, 0, 10), (10, 0, 20)]);
}

#[test]
fn polytree_flattening_retains_ordered_xy_and_z() {
    let (paths, pairs) = crossing_clipper().execute_z_paths(
        ClipOperation::Intersection,
        FillRule::NonZero,
        FillRule::NonZero,
    );
    assert_eq!(pairs, vec![(1, 2), (1, 2)]);
    let coordinates: Vec<_> = paths[0]
        .iter()
        .map(|point| (point.x(), point.y(), point.z))
        .collect();
    assert_eq!(coordinates, vec![(10, 5, -2), (0, 5, -1)]);
}

#[test]
fn ordinary_and_z_execution_have_identical_complete_xy_output() {
    let points = [(0, 0), (20, 0), (20, 20), (0, 20)];
    let mut ordinary = Clipper::new(ClipperOptions::default());
    ordinary
        .add_closed_path(
            &crate::geometry::Polygon::new(points.iter().map(|&(x, y)| Point::new(x, y)).collect()),
            PathRole::Subject,
        )
        .unwrap();
    let ordinary = ordinary
        .execute_paths(ClipOperation::Union, FillRule::NonZero, FillRule::NonZero)
        .unwrap();

    let mut with_z = Clipper::new(ClipperOptions::default());
    with_z
        .add_z_closed_path(&path(&points, 8), PathRole::Subject)
        .unwrap();
    let (with_z, table) =
        with_z.execute_z_paths(ClipOperation::Union, FillRule::NonZero, FillRule::NonZero);
    assert!(table.is_empty());
    assert_eq!(
        with_z
            .iter()
            .map(|path| path.iter().map(|point| point.xy).collect::<Vec<_>>())
            .collect::<Vec<_>>(),
        ordinary
            .iter()
            .map(|polygon| polygon.points().to_vec())
            .collect::<Vec<_>>()
    );
}

#[test]
fn nested_polytree_flattens_outer_hole_island_preorder_with_z() {
    let rings = [
        path(&[(0, 0), (100, 0), (100, 100), (0, 100)], 1),
        path(&[(20, 20), (20, 80), (80, 80), (80, 20)], 2),
        path(&[(40, 40), (60, 40), (60, 60), (40, 60)], 3),
    ];
    let mut clipper = Clipper::new(ClipperOptions::default());
    for ring in &rings {
        clipper.add_z_closed_path(ring, PathRole::Subject).unwrap();
    }

    let (paths, table) =
        clipper.execute_z_paths(ClipOperation::Union, FillRule::NonZero, FillRule::NonZero);
    assert!(table.is_empty());
    assert_eq!(paths.len(), 3);
    assert_eq!(
        paths
            .iter()
            .map(|path| {
                let mut labels = path.iter().map(|point| point.z).collect::<Vec<_>>();
                labels.sort_unstable();
                labels.dedup();
                labels
            })
            .collect::<Vec<_>>(),
        vec![vec![1], vec![2], vec![3]]
    );
}

#[test]
fn mixed_open_and_closed_polytree_flattens_root_preorder_with_z() {
    let mut clipper = Clipper::new(ClipperOptions::default());
    clipper
        .add_z_open_path(&path(&[(-5, 20), (15, 20)], 2), PathRole::Subject)
        .unwrap();
    clipper
        .add_z_closed_path(
            &path(&[(0, 0), (10, 0), (10, 10), (0, 10)], 1),
            PathRole::Subject,
        )
        .unwrap();
    let (paths, table) =
        clipper.execute_z_paths(ClipOperation::Union, FillRule::NonZero, FillRule::NonZero);
    assert!(table.is_empty());
    assert_eq!(
        paths.iter().map(|path| xyz(path)).collect::<Vec<_>>(),
        vec![
            vec![(15, 20, 2), (-5, 20, 2)],
            vec![(10, 10, 1), (0, 10, 1), (0, 0, 1), (10, 0, 1)],
        ]
    );
}

#[test]
fn xy_duplicate_cleanup_keeps_one_original_z_without_synthesis() {
    let mut clipper = Clipper::new(ClipperOptions::default());
    let input = vec![
        KernelPoint::new(0, 0, 4),
        KernelPoint::new(20, 0, 4),
        KernelPoint::new(20, 0, 99),
        KernelPoint::new(20, 20, 4),
        KernelPoint::new(0, 20, 4),
        KernelPoint::new(0, 0, 77),
    ];
    clipper
        .add_z_closed_path(&input, PathRole::Subject)
        .unwrap();
    let (paths, table) =
        clipper.execute_z_paths(ClipOperation::Union, FillRule::NonZero, FillRule::NonZero);
    assert!(table.is_empty());
    assert!(paths[0].iter().all(|point| [4, 99, 77].contains(&point.z)));
    assert!(!paths[0].iter().any(|point| point.z == 0));
}

#[test]
fn strict_type3_touch_writes_same_filled_z_to_both_output_records() {
    let mut clipper = Clipper::new(ClipperOptions {
        strictly_simple: true,
        ..ClipperOptions::default()
    });
    clipper
        .add_z_closed_path(&path(&[(0, 20), (10, 0), (20, 20)], 1), PathRole::Subject)
        .unwrap();
    clipper
        .add_z_closed_path(
            &path(&[(-5, 10), (0, 5), (5, 10), (0, 15)], 2),
            PathRole::Subject,
        )
        .unwrap();
    let (paths, _) =
        clipper.execute_z_paths(ClipOperation::Union, FillRule::NonZero, FillRule::NonZero);
    let touch_z = paths
        .iter()
        .flatten()
        .filter(|point| point.xy == Point::new(5, 10))
        .map(|point| point.z)
        .collect::<Vec<_>>();
    assert_eq!(touch_z.len(), 2);
    assert_ne!(touch_z[0], 0);
    assert_eq!(touch_z[0], touch_z[1]);
}

#[test]
fn horizontal_crossings_from_both_scan_directions_keep_filled_labels() {
    let mut clipper = Clipper::new(ClipperOptions::default());
    clipper
        .add_z_closed_path(
            &path(&[(0, 0), (40, 0), (40, 20), (0, 20)], 5),
            PathRole::Subject,
        )
        .unwrap();
    clipper
        .add_z_closed_path(
            &path(&[(10, -10), (30, -10), (30, 30), (10, 30)], 8),
            PathRole::Clip,
        )
        .unwrap();
    let (paths, pairs) = clipper.execute_z_paths(
        ClipOperation::Intersection,
        FillRule::NonZero,
        FillRule::NonZero,
    );
    assert_eq!(pairs, vec![(5, 8); 6]);
    assert_eq!(paths.len(), 1);
    assert_eq!(paths[0].iter().filter(|point| point.z < 0).count(), 4);
    assert!(paths[0].iter().all(|point| point.z != 0));
}
