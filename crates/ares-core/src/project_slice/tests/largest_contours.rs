use crate::{
    ProjectVolumeType,
    geometry::{ExPolygon, Point, Polygon},
    mesh_slicer::SlicingMode,
};

use super::super::{
    closing::{PostClosingLayer, PostClosingPrintObject, PostClosingVolume},
    largest_contours::apply_project_largest_contours,
    layers::{PlannedLayer, PlannedPrintObject},
};

#[derive(Debug, PartialEq)]
struct ObjectRecords {
    plan: PlannedPrintObject,
    volumes: Vec<VolumeRecords>,
}

#[derive(Debug, PartialEq)]
struct VolumeRecords {
    source_volume_index: usize,
    ordinal: u32,
    volume_type: ProjectVolumeType,
    modes: Vec<SlicingMode>,
}

#[test]
fn task22h_project_stage_preserves_non_multiple_and_non_positive_largest_contour_layers_exactly() {
    let expected = [
        vec![shape(0, 4, 4, &[]), shape(20, 9, 9, &[])],
        vec![shape(40, 5, 7, &[(41, 41)]), shape(50, 4, 4, &[])],
        vec![shape(60, 3, 8, &[]), shape(80, 8, 8, &[])],
        Vec::new(),
        vec![shape(100, 12, 6, &[(101, 101), (104, 101)])],
    ];
    let mut objects = vec![PostClosingPrintObject::new(
        plan(
            17,
            23,
            &[
                (101, 0.11, 0.11, 0.055),
                (103, 0.13, 0.24, 0.175),
                (107, 0.17, 0.41, 0.325),
                (109, 0.19, 0.60, 0.505),
                (113, 0.23, 0.83, 0.715),
            ],
        ),
        vec![PostClosingVolume::new(
            29,
            31,
            ProjectVolumeType::SupportEnforcer,
            vec![
                layer(SlicingMode::Regular, expected[0].clone()),
                layer(SlicingMode::EvenOdd, expected[1].clone()),
                layer(SlicingMode::Positive, expected[2].clone()),
                layer(SlicingMode::PositiveLargestContour, expected[3].clone()),
                layer(SlicingMode::PositiveLargestContour, expected[4].clone()),
            ],
        )],
    )];
    let records = record_facts(&objects);

    apply_project_largest_contours(&mut objects);

    assert_eq!(record_facts(&objects), records);
    for (layer_index, expected) in expected.iter().enumerate() {
        assert_eq!(layer_expolygons(&objects, 0, 0, layer_index), expected);
    }
}

#[test]
fn task22h_project_stage_selects_each_multiple_positive_largest_contour_layer_locally() {
    let object0_volume0_layer0 = shape(200, 20, 20, &[(203, 203), (211, 203)]);
    let object0_volume0_layer1 = shape(300, 15, 15, &[(303, 303)]);
    let object0_volume1_layer0 = shape(400, 30, 30, &[(403, 403), (411, 403)]);
    let object0_volume1_layer1 = shape(500, 11, 11, &[]);
    let object1_volume0_layer0 = shape(600, 8, 8, &[(602, 602)]);
    let object1_volume0_layer1 = shape(700, 12, 12, &[(702, 702), (706, 702)]);
    let object1_volume1_layer0 = shape(800, 50, 50, &[(802, 802)]);
    let object1_volume1_layer1 = shape(900, 20, 20, &[(902, 902), (906, 902)]);

    let mut objects = vec![
        PostClosingPrintObject::new(
            plan(37, 41, &[(127, 0.18, 1.18, 1.09), (131, 0.22, 1.40, 1.29)]),
            vec![
                PostClosingVolume::new(
                    43,
                    47,
                    ProjectVolumeType::ModelPart,
                    vec![
                        positive_largest_contour_layer(vec![
                            shape(220, 10, 10, &[]),
                            object0_volume0_layer0.clone(),
                            shape(240, 15, 15, &[]),
                        ]),
                        positive_largest_contour_layer(vec![
                            object0_volume0_layer1.clone(),
                            shape(330, 15, 15, &[(332, 332), (336, 332)]),
                        ]),
                    ],
                ),
                PostClosingVolume::new(
                    53,
                    59,
                    ProjectVolumeType::NegativeVolume,
                    vec![
                        positive_largest_contour_layer(vec![
                            object0_volume1_layer0.clone(),
                            shape(430, 7, 7, &[]),
                        ]),
                        positive_largest_contour_layer(vec![
                            shape(520, 9, 9, &[]),
                            object0_volume1_layer1.clone(),
                        ]),
                    ],
                ),
            ],
        ),
        PostClosingPrintObject::new(
            plan(
                61,
                67,
                &[(137, 0.27, 2.27, 2.135), (139, 0.31, 2.58, 2.425)],
            ),
            vec![
                PostClosingVolume::new(
                    71,
                    73,
                    ProjectVolumeType::ParameterModifier,
                    vec![
                        positive_largest_contour_layer(vec![
                            shape(620, 4, 4, &[]),
                            object1_volume0_layer0.clone(),
                        ]),
                        positive_largest_contour_layer(vec![
                            shape(720, 10, 10, &[]),
                            object1_volume0_layer1.clone(),
                            shape(740, 11, 11, &[]),
                        ]),
                    ],
                ),
                PostClosingVolume::new(
                    79,
                    83,
                    ProjectVolumeType::SupportBlocker,
                    vec![
                        positive_largest_contour_layer(vec![
                            shape(820, 40, 40, &[]),
                            object1_volume1_layer0.clone(),
                        ]),
                        positive_largest_contour_layer(vec![
                            object1_volume1_layer1.clone(),
                            shape(930, 20, 20, &[(932, 932)]),
                        ]),
                    ],
                ),
            ],
        ),
    ];
    let records = record_facts(&objects);

    apply_project_largest_contours(&mut objects);

    assert_eq!(record_facts(&objects), records);
    for (object, volume, expected_layers) in [
        (0, 0, [object0_volume0_layer0, object0_volume0_layer1]),
        (0, 1, [object0_volume1_layer0, object0_volume1_layer1]),
        (1, 0, [object1_volume0_layer0, object1_volume0_layer1]),
        (1, 1, [object1_volume1_layer0, object1_volume1_layer1]),
    ] {
        for (layer, expected) in expected_layers.into_iter().enumerate() {
            assert_eq!(
                layer_expolygons(&objects, object, volume, layer),
                &[expected]
            );
        }
    }
}

fn record_facts(objects: &[PostClosingPrintObject]) -> Vec<ObjectRecords> {
    objects
        .iter()
        .map(|object| ObjectRecords {
            plan: object.plan().clone(),
            volumes: object
                .volumes()
                .iter()
                .map(|volume| VolumeRecords {
                    source_volume_index: volume.source_volume_index(),
                    ordinal: volume.ordinal(),
                    volume_type: volume.volume_type(),
                    modes: volume.layers().iter().map(PostClosingLayer::mode).collect(),
                })
                .collect(),
        })
        .collect()
}

fn layer_expolygons(
    objects: &[PostClosingPrintObject],
    object: usize,
    volume: usize,
    layer: usize,
) -> &[ExPolygon] {
    objects[object].volumes()[volume].layers()[layer].expolygons()
}

fn plan(
    source_object_index: usize,
    transform_index: usize,
    layers: &[(usize, f64, f64, f64)],
) -> PlannedPrintObject {
    PlannedPrintObject {
        source_object_index,
        transform_index,
        layers: layers
            .iter()
            .map(|&(id, height, print_z, slice_z)| PlannedLayer {
                id,
                height,
                print_z,
                slice_z,
            })
            .collect(),
    }
}

fn positive_largest_contour_layer(expolygons: Vec<ExPolygon>) -> PostClosingLayer {
    layer(SlicingMode::PositiveLargestContour, expolygons)
}

fn layer(mode: SlicingMode, expolygons: Vec<ExPolygon>) -> PostClosingLayer {
    PostClosingLayer::new(mode, expolygons)
}

fn shape(x: i64, width: i64, height: i64, holes: &[(i64, i64)]) -> ExPolygon {
    ExPolygon::new(
        rectangle(x, x, width, height),
        holes
            .iter()
            .map(|&(hole_x, hole_y)| rectangle(hole_x, hole_y, 2, 2))
            .collect(),
    )
}

fn rectangle(x: i64, y: i64, width: i64, height: i64) -> Polygon {
    Polygon::new(vec![
        Point::new(x, y),
        Point::new(x + width, y),
        Point::new(x + width, y + height),
        Point::new(x, y + height),
    ])
}
