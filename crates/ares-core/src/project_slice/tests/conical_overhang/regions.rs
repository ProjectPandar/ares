use crate::geometry::CoordinateScale;

use super::{
    apply_objects, expolygon, layer_geometry, marked_surface, object_options, output_rectangle,
    post_region, print_object, rectangle, region_options, square, surface_metadata,
};

const DEFAULT_METADATA: (f64, u16, f64, u16) = (-1.0, 1, -1.0, 0);
const MARKED_METADATA: (f64, u16, f64, u16) = (0.37, 7, 1.25, 9);

#[test]
fn task22l_stage_pair_wide_gate_projects_into_enabled_empty_region() {
    let regions = vec![
        post_region(
            10,
            region_options(true, 1, 0, 0.0, 0),
            vec![vec![square(0, 1_000_000)], vec![]],
        ),
        post_region(
            20,
            region_options(true, 1, 0, 0.0, 0),
            vec![
                vec![],
                vec![rectangle(800_000, 200_000, 1_800_000, 800_000)],
            ],
        ),
    ];
    let mut object = print_object(0, 393, &[0.2, 0.2], regions);
    apply(&mut object, CoordinateScale::Normal);

    assert_eq!(
        layer_geometry(&object, 0, 0),
        vec![expolygon(&[
            (1_000_000, 199_990),
            (799_990, 199_990),
            (799_990, 800_010),
            (1_000_000, 800_010),
            (1_000_000, 1_000_000),
            (0, 1_000_000),
            (0, 0),
            (1_000_000, 0),
        ])]
    );
    assert_eq!(
        layer_geometry(&object, 1, 0),
        vec![output_rectangle(800_000, 200_000, 1_800_000, 800_000)]
    );
    assert!(layer_geometry(&object, 0, 1).is_empty());
    assert_eq!(
        layer_geometry(&object, 1, 1),
        vec![rectangle(800_000, 200_000, 1_800_000, 800_000)]
    );
}

#[test]
fn task22l_stage_filters_fully_covered_islands_but_keeps_partial_island_whole() {
    let region = post_region(
        0,
        region_options(true, 1, 0, 0.0, 0),
        vec![
            vec![rectangle(0, 0, 3_000_000, 2_000_000)],
            vec![
                rectangle(500_000, 500_000, 1_000_000, 1_000_000),
                rectangle(2_500_000, 500_000, 3_500_000, 1_500_000),
            ],
        ],
    );
    let mut object = print_object(0, 183, &[0.2, 0.2], vec![region]);
    apply(&mut object, CoordinateScale::Normal);

    assert_eq!(
        layer_geometry(&object, 0, 0),
        vec![expolygon(&[
            (3_000_000, 500_000),
            (3_500_000, 500_000),
            (3_500_000, 1_500_000),
            (3_000_000, 1_500_000),
            (3_000_000, 2_000_000),
            (0, 2_000_000),
            (0, 0),
            (3_000_000, 0),
        ])]
    );

    let eroded = post_region(
        0,
        region_options(true, 1, 0, 0.0, 0),
        vec![
            vec![rectangle(0, 0, 100_000, 100_000)],
            vec![rectangle(500_000, 500_000, 900_000, 900_000)],
        ],
    );
    let mut object = print_object(0, 233, &[0.2, 0.2], vec![eroded]);
    apply_objects(
        std::slice::from_mut(&mut object),
        vec![object_options(55.0, 0.0, 0.2)],
        CoordinateScale::Normal,
    )
    .unwrap();
    assert_eq!(
        layer_geometry(&object, 0, 0),
        vec![output_rectangle(0, 0, 100_000, 100_000)]
    );
}

#[test]
fn task22l_stage_region_order_transfers_overlapping_ownership() {
    let mut object = two_enabled_object(&[0.2, 0.2], 293, 1_000_000);
    apply(&mut object, CoordinateScale::Normal);
    let (first, second) = expected_owned_layers();

    assert_eq!(layer_geometry(&object, 0, 0), first);
    assert_eq!(layer_geometry(&object, 1, 0), second);
}

#[test]
fn task22l_stage_keeps_pair_start_current_poly_fixed_across_region_passes() {
    let options = region_options(true, 1, 0, 0.0, 0);
    let candidate = rectangle(1_200_000, 200_000, 1_800_000, 800_000);
    let regions = vec![
        post_region(
            10,
            options.clone(),
            vec![vec![square(0, 1_000_000)], vec![candidate.clone()]],
        ),
        post_region(20, options, vec![vec![], vec![candidate]]),
    ];
    let mut object = print_object(0, 294, &[0.2, 0.2], regions);
    apply(&mut object, CoordinateScale::Normal);

    assert_eq!(
        layer_geometry(&object, 0, 0),
        vec![output_rectangle(0, 0, 1_000_000, 1_000_000)]
    );
    assert_eq!(
        layer_geometry(&object, 1, 0),
        vec![output_rectangle(1_200_000, 200_000, 1_800_000, 800_000)]
    );
}

#[test]
fn task22l_stage_cascades_top_down_across_two_enabled_regions() {
    let mut object = two_enabled_object(&[0.08, 0.32, 0.11], 383, 600_000);
    apply(&mut object, CoordinateScale::Normal);
    let (middle_first, middle_second) = expected_owned_layers();

    assert_eq!(
        layer_geometry(&object, 0, 0),
        vec![expolygon(&[
            (1_000_000, 200_000),
            (2_000_000, 200_000),
            (2_000_000, 599_990),
            (1_399_990, 599_990),
            (1_399_990, 1_000_000),
            (800_000, 1_000_000),
            (800_000, 999_990),
            (0, 999_990),
            (0, 0),
            (1_000_000, 0),
        ])]
    );
    assert_eq!(layer_geometry(&object, 1, 0), middle_second.clone());
    assert_eq!(layer_geometry(&object, 0, 1), middle_first);
    assert_eq!(layer_geometry(&object, 1, 1), middle_second);
    assert_eq!(
        layer_geometry(&object, 0, 2),
        vec![rectangle(800_000, 200_000, 2_000_000, 1_000_000)]
    );
    assert_eq!(
        layer_geometry(&object, 1, 2),
        vec![rectangle(1_400_000, 600_000, 2_600_000, 1_400_000)]
    );
}

#[test]
fn task22l_stage_interior_empty_layer_is_a_cascade_barrier() {
    let region = post_region(
        0,
        region_options(true, 1, 0, 0.0, 0),
        vec![
            vec![square(0, 1_000_000)],
            vec![],
            vec![rectangle(500_000, 0, 1_500_000, 1_000_000)],
        ],
    );
    let mut object = print_object(0, 203, &[0.2, 0.2, 0.2], vec![region]);
    apply(&mut object, CoordinateScale::Normal);

    assert_eq!(layer_geometry(&object, 0, 0), vec![square(0, 1_000_000)]);
    assert!(layer_geometry(&object, 0, 1).is_empty());
    assert_eq!(
        layer_geometry(&object, 0, 2),
        vec![rectangle(500_000, 0, 1_500_000, 1_000_000)]
    );
}

#[test]
fn task22l_stage_nonempty_candidate_resets_same_and_other_region_metadata() {
    let regions = vec![
        post_region(
            10,
            region_options(true, 1, 0, 0.0, 0),
            vec![
                vec![square(0, 1_000_000)],
                vec![rectangle(800_000, 0, 1_800_000, 1_000_000)],
            ],
        ),
        post_region(
            20,
            region_options(false, 1, 0, 0.0, 0),
            vec![vec![rectangle(3_000_000, 0, 4_000_000, 1_000_000)], vec![]],
        ),
    ];
    let mut object = print_object(0, 213, &[0.2, 0.2], regions);
    object.regions[0].layers[0].surfaces = vec![marked_surface(square(0, 1_000_000))];
    object.regions[0].layers[1].surfaces =
        vec![marked_surface(rectangle(800_000, 0, 1_800_000, 1_000_000))];
    object.regions[1].layers[0].surfaces = vec![marked_surface(rectangle(
        3_000_000, 0, 4_000_000, 1_000_000,
    ))];
    apply(&mut object, CoordinateScale::Normal);

    assert_eq!(surface_metadata(&object, 0, 0), vec![DEFAULT_METADATA]);
    assert_eq!(surface_metadata(&object, 1, 0), vec![DEFAULT_METADATA]);
    assert_eq!(surface_metadata(&object, 0, 1), vec![MARKED_METADATA]);
}

#[test]
fn task22l_stage_empty_candidate_resets_same_and_other_region_metadata() {
    let regions = vec![
        post_region(
            10,
            region_options(true, 1, 0, 0.0, 0),
            vec![vec![square(0, 1_000_000)], vec![]],
        ),
        post_region(
            20,
            region_options(false, 1, 0, 0.0, 0),
            vec![
                vec![rectangle(2_000_000, 0, 3_000_000, 1_000_000)],
                vec![rectangle(2_100_000, 200_000, 2_900_000, 800_000)],
            ],
        ),
    ];
    let mut object = print_object(0, 193, &[0.2, 0.2], regions);
    object.regions[0].layers[0].surfaces = vec![marked_surface(square(0, 1_000_000))];
    object.regions[1].layers[0].surfaces = vec![marked_surface(rectangle(
        2_000_000, 0, 3_000_000, 1_000_000,
    ))];
    object.regions[1].layers[1].surfaces = vec![marked_surface(rectangle(
        2_100_000, 200_000, 2_900_000, 800_000,
    ))];
    apply(&mut object, CoordinateScale::Normal);

    assert_eq!(surface_metadata(&object, 0, 0), vec![DEFAULT_METADATA]);
    assert_eq!(surface_metadata(&object, 1, 0), vec![DEFAULT_METADATA]);
    assert_eq!(surface_metadata(&object, 1, 1), vec![MARKED_METADATA]);
}

#[test]
fn task22l_stage_skipped_pairs_preserve_nondefault_metadata() {
    let cases = [
        post_region(
            0,
            region_options(false, 1, 0, 0.0, 0),
            vec![vec![square(0, 1_000_000)], vec![square(100_000, 1_100_000)]],
        ),
        post_region(
            0,
            region_options(true, 1, 0, 0.0, 0),
            vec![vec![square(0, 1_000_000)], vec![]],
        ),
    ];
    for region in cases {
        let mut object = print_object(0, 0, &[0.2, 0.2], vec![region]);
        object.regions[0].layers[0].surfaces = vec![marked_surface(square(0, 1_000_000))];
        if !object.regions[0].layers[1].surfaces.is_empty() {
            object.regions[0].layers[1].surfaces = vec![marked_surface(square(100_000, 1_100_000))];
        }
        apply(&mut object, CoordinateScale::Normal);
        assert_eq!(surface_metadata(&object, 0, 0), vec![MARKED_METADATA]);
        if !object.regions[0].layers[1].surfaces.is_empty() {
            assert_eq!(surface_metadata(&object, 0, 1), vec![MARKED_METADATA]);
        }
    }
}

#[test]
fn task22l_stage_empty_and_exact_ninety_return_before_geometry_or_writes() {
    let region = post_region(
        0,
        region_options(true, 1, 0, 0.0, 0),
        vec![vec![square(0, 1_000_000)], vec![square(100_000, 1_100_000)]],
    );
    let mut ninety = print_object(0, 0, &[0.2, 0.2], vec![region]);
    ninety.regions[0].layers[0].surfaces = vec![marked_surface(square(0, 1_000_000))];
    ninety.regions[0].layers[1].surfaces = vec![marked_surface(square(100_000, 1_100_000))];
    apply_objects(
        std::slice::from_mut(&mut ninety),
        vec![object_options(90.0, 1.0e30, 1.0e34)],
        CoordinateScale::Normal,
    )
    .unwrap();
    assert_eq!(surface_metadata(&ninety, 0, 0), vec![MARKED_METADATA]);
    assert_eq!(surface_metadata(&ninety, 0, 1), vec![MARKED_METADATA]);

    let mut empty = print_object(0, 0, &[], vec![]);
    apply_objects(
        std::slice::from_mut(&mut empty),
        vec![object_options(0.0, 1.0e30, 1.0e34)],
        CoordinateScale::LargeBed,
    )
    .unwrap();
    assert!(empty.plan.layers.is_empty());
}

fn apply(object: &mut super::PostRegionPrintObject, scale: CoordinateScale) {
    apply_objects(
        std::slice::from_mut(object),
        vec![object_options(0.0, 0.0, 0.2)],
        scale,
    )
    .unwrap();
}

fn two_enabled_object(
    heights: &[f64],
    transform_index: usize,
    lower_width: i64,
) -> super::PostRegionPrintObject {
    let mut first = vec![rectangle(0, 0, lower_width, 1_000_000)];
    let mut second = vec![rectangle(0, 1_000_000, lower_width, 2_000_000)];
    if heights.len() == 3 {
        first.push(rectangle(0, 0, 1_000_000, 1_000_000));
        second.push(rectangle(0, 1_000_000, 1_000_000, 2_000_000));
    }
    first.push(rectangle(800_000, 200_000, 2_000_000, 1_000_000));
    second.push(rectangle(1_400_000, 600_000, 2_600_000, 1_400_000));
    print_object(
        0,
        transform_index,
        heights,
        vec![
            post_region(
                10,
                region_options(true, 1, 0, 0.0, 0),
                first.into_iter().map(|shape| vec![shape]).collect(),
            ),
            post_region(
                20,
                region_options(true, 1, 0, 0.0, 0),
                second.into_iter().map(|shape| vec![shape]).collect(),
            ),
        ],
    )
}

fn expected_owned_layers() -> (
    Vec<crate::geometry::ExPolygon>,
    Vec<crate::geometry::ExPolygon>,
) {
    (
        vec![expolygon(&[
            (1_000_000, 200_000),
            (2_000_000, 200_000),
            (2_000_000, 599_990),
            (1_399_990, 599_990),
            (1_399_990, 1_000_000),
            (0, 1_000_000),
            (0, 0),
            (1_000_000, 0),
        ])],
        vec![
            expolygon(&[
                (799_990, 1_000_010),
                (1_000_000, 1_000_010),
                (1_000_000, 2_000_000),
                (0, 2_000_000),
                (0, 1_000_000),
                (799_990, 1_000_000),
            ]),
            output_rectangle(1_400_000, 600_000, 2_600_000, 1_400_000),
        ],
    )
}
