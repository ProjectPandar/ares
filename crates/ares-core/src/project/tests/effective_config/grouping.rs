use crate::{
    ProjectInstance, ProjectObject,
    project::{effective_config::grouping::group_print_object_transforms, transform::Transform3d},
};

#[test]
fn xy_translation_collapses_while_z_rotation_and_scale_remain_distinct() {
    let xy_a = transform_3mf("1 0 0 0 1 0 0 0 1 10 20 0");
    let xy_b = transform_3mf("1 0 0 0 1 0 0 0 1 -4 7 0");
    let z_translation = transform_3mf("1 0 0 0 1 0 0 0 1 2 3 1");
    let rotation = transform_3mf("0 1 0 -1 0 0 0 0 1 0 0 0");
    let scale = transform_3mf("2 0 0 0 1 0 0 0 1 0 0 0");
    let objects = [project_object(
        7,
        vec![
            (true, xy_a),
            (true, xy_b),
            (true, z_translation),
            (true, rotation),
            (true, scale),
        ],
    )];

    let grouped = group_print_object_transforms(&objects);

    assert_eq!(grouped.by_object.len(), 1);
    assert_eq!(grouped.by_object[0].transforms.len(), 4);
    assert_eq!(grouped.effective_print_object_count, 4);
    for expected in [
        Transform3d::IDENTITY,
        z_translation.without_xy_translation(),
        rotation,
        scale,
    ] {
        assert_eq!(exact_count(&grouped.by_object[0].transforms, expected), 1);
    }
}

#[test]
fn reversed_input_is_sorted_in_eigen_column_major_order() {
    let column_major_first = row_major([
        1.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    ]);
    let row_major_first = row_major([
        1.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    ]);
    let forward = [project_object(
        8,
        vec![(true, row_major_first), (true, column_major_first)],
    )];
    let reversed = [project_object(
        8,
        vec![(true, column_major_first), (true, row_major_first)],
    )];

    let forward = group_print_object_transforms(&forward);
    let reversed = group_print_object_transforms(&reversed);

    assert_eq!(forward, reversed);
    assert_eq!(forward.by_object[0].transforms.len(), 2);
    assert!(forward.by_object[0].transforms[0].fixed_order_equal(column_major_first));
    assert!(forward.by_object[0].transforms[1].fixed_order_equal(row_major_first));
}

#[test]
fn signed_zero_duplicates_group_and_non_printable_is_excluded() {
    let positive_zero = transform_3mf("1 0 0 0 1 0 0 0 1 0 0 0");
    let negative_zero = transform_3mf("1 -0 -0 -0 1 -0 -0 -0 1 -0 -0 -0");
    let total_order_interloper = transform_3mf("1 -0 1 -0 1 -0 -0 -0 1 -0 -0 -0");
    let non_printable = transform_3mf("1 0 0 0 1 0 0 0 1 0 0 9");
    let objects = [project_object(
        9,
        vec![
            (true, negative_zero),
            (true, total_order_interloper),
            (true, positive_zero),
            (false, non_printable),
        ],
    )];

    let grouped = group_print_object_transforms(&objects);

    assert_eq!(grouped.by_object[0].transforms.len(), 2);
    assert_eq!(grouped.effective_print_object_count, 2);
    assert_eq!(
        exact_count(&grouped.by_object[0].transforms, positive_zero),
        1
    );
    assert_eq!(
        exact_count(&grouped.by_object[0].transforms, total_order_interloper),
        1
    );
}

#[test]
fn equal_transforms_on_different_objects_stay_separate_and_preserve_order() {
    let objects = [
        project_object(20, vec![(true, Transform3d::IDENTITY)]),
        project_object(10, vec![(true, Transform3d::IDENTITY)]),
    ];

    let grouped = group_print_object_transforms(&objects);

    assert_eq!(grouped.by_object.len(), 2);
    assert_eq!(grouped.effective_print_object_count, 2);
    assert_eq!(
        grouped
            .by_object
            .iter()
            .map(|entry| objects[entry.source_object_index].id())
            .collect::<Vec<_>>(),
        vec![20, 10]
    );
    assert_eq!(
        grouped
            .by_object
            .iter()
            .map(|entry| entry.transforms.len())
            .collect::<Vec<_>>(),
        vec![1, 1]
    );
}

#[test]
fn effective_count_sums_per_object_unique_groups() {
    let identity_xy_a = transform_3mf("1 0 0 0 1 0 0 0 1 1 2 0");
    let identity_xy_b = transform_3mf("1 0 0 0 1 0 0 0 1 3 4 0");
    let z_translation = transform_3mf("1 0 0 0 1 0 0 0 1 0 0 1");
    let scale = transform_3mf("2 0 0 0 1 0 0 0 1 0 0 0");
    let objects = [
        project_object(
            1,
            vec![
                (true, identity_xy_a),
                (true, identity_xy_b),
                (true, z_translation),
            ],
        ),
        project_object(2, vec![(true, Transform3d::IDENTITY), (true, scale)]),
        project_object(3, vec![(false, Transform3d::IDENTITY)]),
    ];

    let grouped = group_print_object_transforms(&objects);

    assert_eq!(objects.len(), 3);
    assert_eq!(
        objects
            .iter()
            .map(|object| {
                object
                    .instances()
                    .iter()
                    .filter(|instance| instance.printable())
                    .count()
            })
            .sum::<usize>(),
        5
    );
    assert_eq!(
        grouped
            .by_object
            .iter()
            .map(|entry| entry.transforms.len())
            .collect::<Vec<_>>(),
        vec![2, 2, 0]
    );
    assert_eq!(grouped.effective_print_object_count, 4);
}

fn project_object(id: u32, instances: Vec<(bool, Transform3d)>) -> ProjectObject {
    let instances = instances
        .into_iter()
        .enumerate()
        .map(|(index, (printable, transform))| {
            let instance_id = u32::try_from(index).unwrap();
            ProjectInstance::new(
                [id, instance_id, 1_000 + instance_id],
                printable,
                false,
                transform,
            )
        })
        .collect();
    ProjectObject::new(
        format!("synthetic-{id}.model"),
        id,
        (
            format!("object-{id}"),
            String::new(),
            Default::default(),
            Default::default(),
        ),
        Vec::new(),
        instances,
    )
}

fn transform_3mf(value: &str) -> Transform3d {
    Transform3d::parse_3mf(value).unwrap()
}

fn row_major(values: [f64; 16]) -> Transform3d {
    let value = values
        .into_iter()
        .map(|value| value.to_string())
        .collect::<Vec<_>>()
        .join(" ");
    Transform3d::parse_row_major(&value).unwrap()
}

fn exact_count(transforms: &[Transform3d], expected: Transform3d) -> usize {
    transforms
        .iter()
        .filter(|transform| transform.fixed_order_equal(expected))
        .count()
}
