use serde_json::json;

use super::super::{
    ScaledPoint, append_all_extruder_intersection_first_result,
    collect_extruder_diff_first_results, find_intersection_ids, printable_area_polygons,
    printable_filament_intersection_ids_changed, scale_printable_area_polygons,
};

fn map(value: serde_json::Value) -> serde_json::Map<String, serde_json::Value> {
    value
        .as_object()
        .expect("test value must be object")
        .clone()
}

#[test]
fn scaled_printable_area_polygons_scale_and_round_printable_points() {
    let polygons = printable_area_polygons(&map(json!({
        "printable_area": [[0, 0], [1.2345674, 2.0000005]]
    })))
    .expect("printable area should parse");

    let scaled = scale_printable_area_polygons(&polygons);

    assert_eq!(scaled.printable[0].x, 0);
    assert_eq!(scaled.printable[0].y, 0);
    assert_eq!(scaled.printable[1].x, 1_234_567);
    assert_eq!(scaled.printable[1].y, 2_000_001);
}

#[test]
fn scaled_printable_area_polygons_preserve_extruder_group_order() {
    let polygons = printable_area_polygons(&map(json!({
        "printable_area": [[0, 0], [1, 0], [1, 1]],
        "extruder_printable_area": [
            [[0.25, 0.5], [0.75, 1.25]],
            [[2.5, 3.5]]
        ]
    })))
    .expect("extruder areas should parse");

    let scaled = scale_printable_area_polygons(&polygons);

    assert_eq!(scaled.extruders.len(), 2);
    assert_eq!(scaled.extruders[0][0].x, 250_000);
    assert_eq!(scaled.extruders[0][0].y, 500_000);
    assert_eq!(scaled.extruders[0][1].x, 750_000);
    assert_eq!(scaled.extruders[0][1].y, 1_250_000);
    assert_eq!(scaled.extruders[1][0].x, 2_500_000);
    assert_eq!(scaled.extruders[1][0].y, 3_500_000);
}

#[test]
fn scaled_printable_area_polygons_scale_negative_fractional_points() {
    let polygons = printable_area_polygons(&map(json!({
        "printable_area": [[-0.0000014, -0.0000015], [0.0000015, 0.0000016]]
    })))
    .expect("printable area should parse");

    let scaled = scale_printable_area_polygons(&polygons);

    assert_eq!(scaled.printable[0].x, -1);
    assert_eq!(scaled.printable[0].y, -2);
    assert_eq!(scaled.printable[1].x, 2);
    assert_eq!(scaled.printable[1].y, 2);
}

#[test]
fn extruder_diff_calls_callback_once_per_extruder_in_order() {
    let polygons = scale_printable_area_polygons(
        &printable_area_polygons(&map(json!({
            "printable_area": [[0, 0], [1, 0], [1, 1]],
            "extruder_printable_area": [
                [[0.1, 0.1]],
                [[0.2, 0.2]]
            ]
        })))
        .expect("areas should parse"),
    );
    let mut calls = Vec::new();

    let _ = collect_extruder_diff_first_results(&polygons, |subject, clip| {
        calls.push((subject[0], clip[0]));
        Vec::new()
    });

    assert_eq!(
        calls,
        vec![
            (polygons.printable[0], polygons.extruders[0][0]),
            (polygons.printable[0], polygons.extruders[1][0]),
        ]
    );
}

#[test]
fn extruder_diff_skips_empty_results_and_appends_first_result_only() {
    let polygons = scale_printable_area_polygons(
        &printable_area_polygons(&map(json!({
            "printable_area": [[0, 0], [1, 0], [1, 1]],
            "extruder_printable_area": [
                [[0.1, 0.1]],
                [[0.2, 0.2]],
                [[0.3, 0.3]]
            ]
        })))
        .expect("areas should parse"),
    );
    let mut index = 0;

    let split_polys = collect_extruder_diff_first_results(&polygons, |_, _| {
        index += 1;
        match index {
            1 => Vec::new(),
            2 => vec![
                vec![ScaledPoint { x: 20, y: 20 }],
                vec![ScaledPoint { x: 21, y: 21 }],
            ],
            3 => vec![vec![ScaledPoint { x: 30, y: 30 }]],
            _ => unreachable!("one callback per extruder"),
        }
    });

    assert_eq!(
        split_polys,
        vec![
            vec![ScaledPoint { x: 20, y: 20 }],
            vec![ScaledPoint { x: 30, y: 30 }],
        ]
    );
}

#[test]
fn all_extruder_intersection_calls_callback_once_with_printable_subject_and_all_extruders() {
    let polygons = scale_printable_area_polygons(
        &printable_area_polygons(&map(json!({
            "printable_area": [[0, 0], [1, 0], [1, 1]],
            "extruder_printable_area": [
                [[0.1, 0.1]],
                [[0.2, 0.2]]
            ]
        })))
        .expect("areas should parse"),
    );
    let mut calls = Vec::new();
    let mut split_polys = Vec::new();

    append_all_extruder_intersection_first_result(&polygons, &mut split_polys, |subject, clips| {
        calls.push((subject.to_vec(), clips.to_vec()));
        Vec::new()
    });

    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].0, vec![polygons.printable.clone()]);
    assert_eq!(calls[0].1, polygons.extruders);
}

#[test]
fn all_extruder_intersection_skips_empty_and_appends_first_result_after_existing_splits() {
    let polygons = scale_printable_area_polygons(
        &printable_area_polygons(&map(json!({
            "printable_area": [[0, 0], [1, 0], [1, 1]],
            "extruder_printable_area": [[[0.1, 0.1]]]
        })))
        .expect("areas should parse"),
    );
    let existing = vec![ScaledPoint { x: 1, y: 1 }];
    let mut split_polys = vec![existing.clone()];

    append_all_extruder_intersection_first_result(&polygons, &mut split_polys, |_, _| Vec::new());

    assert_eq!(split_polys, vec![existing.clone()]);

    append_all_extruder_intersection_first_result(&polygons, &mut split_polys, |_, _| {
        vec![
            vec![ScaledPoint { x: 10, y: 10 }],
            vec![ScaledPoint { x: 11, y: 11 }],
        ]
    });

    assert_eq!(
        split_polys,
        vec![existing, vec![ScaledPoint { x: 10, y: 10 }]]
    );
}

#[test]
fn find_intersection_ids_calls_callback_for_each_contour_in_order() {
    let poly = vec![ScaledPoint { x: 0, y: 0 }];
    let contours = vec![
        vec![ScaledPoint { x: 1, y: 1 }],
        vec![ScaledPoint { x: 2, y: 2 }],
    ];
    let mut calls = Vec::new();

    let ids = find_intersection_ids(&poly, &contours, |subject, contour| {
        calls.push((subject[0], contour[0]));
        Vec::new()
    });

    assert!(ids.is_empty());
    assert_eq!(
        calls,
        vec![(poly[0], contours[0][0]), (poly[0], contours[1][0])]
    );
}

#[test]
fn find_intersection_ids_returns_sorted_non_empty_result_indices() {
    let poly = vec![ScaledPoint { x: 0, y: 0 }];
    let contours = vec![
        vec![ScaledPoint { x: 1, y: 1 }],
        vec![ScaledPoint { x: 2, y: 2 }],
        vec![ScaledPoint { x: 3, y: 3 }],
    ];
    let mut index = 0;

    let ids = find_intersection_ids(&poly, &contours, |_, _| {
        index += 1;
        if index == 1 || index == 3 {
            vec![vec![ScaledPoint { x: index, y: index }]]
        } else {
            Vec::new()
        }
    });

    assert_eq!(ids.into_iter().collect::<Vec<_>>(), vec![0, 2]);
}

#[test]
fn intersection_ids_changed_returns_false_for_equal_old_new_sets() {
    let old_poly = vec![ScaledPoint { x: 0, y: 0 }];
    let new_poly = vec![ScaledPoint { x: 9, y: 9 }];
    let split_polys = vec![
        vec![ScaledPoint { x: 1, y: 1 }],
        vec![ScaledPoint { x: 2, y: 2 }],
    ];

    let changed = printable_filament_intersection_ids_changed(
        &old_poly,
        &new_poly,
        &split_polys,
        |_, contour| {
            if contour[0].x == 1 {
                vec![vec![contour[0]]]
            } else {
                Vec::new()
            }
        },
    );

    assert!(!changed);
}

#[test]
fn intersection_ids_changed_returns_true_for_different_old_new_sets() {
    let old_poly = vec![ScaledPoint { x: 0, y: 0 }];
    let new_poly = vec![ScaledPoint { x: 9, y: 9 }];
    let split_polys = vec![
        vec![ScaledPoint { x: 1, y: 1 }],
        vec![ScaledPoint { x: 2, y: 2 }],
    ];

    let changed = printable_filament_intersection_ids_changed(
        &old_poly,
        &new_poly,
        &split_polys,
        |subject, contour| {
            if (subject[0] == old_poly[0] && contour[0].x == 1)
                || (subject[0] == new_poly[0] && contour[0].x == 2)
            {
                vec![vec![contour[0]]]
            } else {
                Vec::new()
            }
        },
    );

    assert!(changed);
}

#[test]
fn intersection_ids_changed_checks_old_contours_before_new_contours() {
    let old_poly = vec![ScaledPoint { x: 0, y: 0 }];
    let new_poly = vec![ScaledPoint { x: 9, y: 9 }];
    let split_polys = vec![
        vec![ScaledPoint { x: 1, y: 1 }],
        vec![ScaledPoint { x: 2, y: 2 }],
    ];
    let mut calls = Vec::new();

    let _ = printable_filament_intersection_ids_changed(
        &old_poly,
        &new_poly,
        &split_polys,
        |subject, contour| {
            calls.push((subject[0], contour[0]));
            Vec::new()
        },
    );

    assert_eq!(
        calls,
        vec![
            (old_poly[0], split_polys[0][0]),
            (old_poly[0], split_polys[1][0]),
            (new_poly[0], split_polys[0][0]),
            (new_poly[0], split_polys[1][0]),
        ]
    );
}
