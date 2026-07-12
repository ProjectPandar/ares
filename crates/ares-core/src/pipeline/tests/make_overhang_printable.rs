use super::*;
use crate::{Point2, PrintPathRole, ToolpathMoveKind};
use serde_json::json;

#[test]
fn default_make_overhang_printable_preserves_two_layer_fixture() {
    let disabled = options(json!({ "make_overhang_printable": false }));
    let default = options(json!({}));
    let disabled_pipeline =
        crate::pipeline::test_support::unsupported_second_layer_pipeline(&disabled);
    let default_pipeline =
        crate::pipeline::test_support::unsupported_second_layer_pipeline(&default);
    let disabled_gcode =
        String::from_utf8(crate::gcode::format_gcode(&disabled_pipeline, &disabled).unwrap())
            .unwrap();
    let default_gcode =
        String::from_utf8(crate::gcode::format_gcode(&default_pipeline, &default).unwrap())
            .unwrap();

    assert_eq!(
        lower_contours(&default_pipeline),
        lower_contours(&disabled_pipeline)
    );
    assert_eq!(
        lower_perimeter_count(&default_pipeline),
        lower_perimeter_count(&disabled_pipeline)
    );
    assert_eq!(
        lower_print_path_count(&default_pipeline),
        lower_print_path_count(&disabled_pipeline)
    );
    assert_eq!(
        lower_print_move_count(&default_pipeline),
        lower_print_move_count(&disabled_pipeline)
    );
    assert_eq!(
        lower_extrusion_print_move_count(&default_pipeline),
        lower_extrusion_print_move_count(&disabled_pipeline)
    );
    assert_eq!(
        without_option_count(default_gcode),
        without_option_count(disabled_gcode)
    );
}

#[test]
fn enabled_make_overhang_printable_projects_upper_rectangle_into_lower_layer() {
    let off = options(json!({ "make_overhang_printable": false }));
    let on = options(json!({
        "make_overhang_printable": true,
        "make_overhang_printable_angle": 45,
        "gcode_comments": true
    }));
    let off_pipeline = crate::pipeline::test_support::unsupported_second_layer_pipeline(&off);
    let on_pipeline = crate::pipeline::test_support::unsupported_second_layer_pipeline(&on);
    let gcode = String::from_utf8(crate::gcode::format_gcode(&on_pipeline, &on).unwrap()).unwrap();

    assert_eq!(
        lower_contours(&on_pipeline).len(),
        lower_contours(&off_pipeline).len() + 1
    );
    assert_eq!(
        lower_contours(&on_pipeline)[1],
        vec![
            Point2::new(10.2, 0.2),
            Point2::new(13.8, 0.2),
            Point2::new(13.8, 3.8),
            Point2::new(10.2, 3.8),
        ]
    );
    assert!(lower_perimeter_count(&on_pipeline) > lower_perimeter_count(&off_pipeline));
    assert!(lower_print_path_count(&on_pipeline) > lower_print_path_count(&off_pipeline));
    assert!(lower_print_move_count(&on_pipeline) > lower_print_move_count(&off_pipeline));
    assert!(
        lower_extrusion_print_move_count(&on_pipeline)
            > lower_extrusion_print_move_count(&off_pipeline)
    );
    assert!(gcode.contains(";PERIMETER:external:10.2,0.2 -> 13.8,0.2 -> 13.8,3.8 -> 10.2,3.8"));
    assert!(
        gcode.contains(
            ";PRINT_PATH:external_perimeter:10.2,0.2 -> 13.8,0.2 -> 13.8,3.8 -> 10.2,3.8"
        )
    );
}

#[test]
fn make_overhang_printable_angle_ninety_preserves_geometry() {
    let off = options(json!({ "make_overhang_printable": false }));
    let on = options(json!({
        "make_overhang_printable": true,
        "make_overhang_printable_angle": 90
    }));
    let off_pipeline = crate::pipeline::test_support::unsupported_second_layer_pipeline(&off);
    let on_pipeline = crate::pipeline::test_support::unsupported_second_layer_pipeline(&on);

    assert_eq!(lower_contours(&on_pipeline), lower_contours(&off_pipeline));
    assert_eq!(
        lower_perimeter_count(&on_pipeline),
        lower_perimeter_count(&off_pipeline)
    );
    assert_eq!(
        lower_print_path_count(&on_pipeline),
        lower_print_path_count(&off_pipeline)
    );
}

#[test]
fn make_overhang_printable_angle_zero_projects_unshrunk_upper_rectangle() {
    let on = options(json!({
        "make_overhang_printable": true,
        "make_overhang_printable_angle": 0
    }));
    let pipeline = crate::pipeline::test_support::unsupported_second_layer_pipeline(&on);

    assert_eq!(
        lower_contours(&pipeline)[1],
        vec![
            Point2::new(10.0, 0.0),
            Point2::new(14.0, 0.0),
            Point2::new(14.0, 4.0),
            Point2::new(10.0, 4.0),
        ]
    );
}

#[test]
fn make_overhang_printable_preserves_mixed_non_rectangular_layer_pairs() {
    let on = options(json!({
        "make_overhang_printable": true,
        "make_overhang_printable_angle": 45
    }));
    let lower_non_rectangular = crate::Contour::new(vec![
        Point2::new(0.0, 0.0),
        Point2::new(4.0, 0.0),
        Point2::new(4.0, 4.0),
        Point2::new(2.0, 5.0),
        Point2::new(0.0, 4.0),
    ]);
    let lower_rectangular = crate::Contour::new(vec![
        Point2::new(0.0, 0.0),
        Point2::new(4.0, 0.0),
        Point2::new(4.0, 4.0),
        Point2::new(0.0, 4.0),
    ]);
    let upper_rectangular = crate::Contour::new(vec![
        Point2::new(10.0, 0.0),
        Point2::new(14.0, 0.0),
        Point2::new(14.0, 4.0),
        Point2::new(10.0, 4.0),
    ]);
    let upper_non_rectangular = crate::Contour::new(vec![
        Point2::new(10.0, 0.0),
        Point2::new(14.0, 0.0),
        Point2::new(14.0, 4.0),
        Point2::new(12.0, 5.0),
        Point2::new(10.0, 4.0),
    ]);

    for contours_by_layer in [
        vec![
            vec![lower_non_rectangular.clone()],
            vec![upper_rectangular.clone()],
        ],
        vec![
            vec![lower_rectangular.clone()],
            vec![upper_non_rectangular.clone()],
        ],
        vec![
            vec![lower_rectangular.clone()],
            vec![upper_rectangular.clone(), upper_non_rectangular.clone()],
        ],
    ] {
        let expected_lower = contours_by_layer[0]
            .iter()
            .map(|contour| contour.points().to_vec())
            .collect::<Vec<_>>();
        let pipeline = crate::pipeline::test_support::contour_layers_pipeline_from_layers_for_tests(
            &on,
            contours_by_layer,
        );

        assert_eq!(lower_contours(&pipeline), expected_lower);
    }
}

#[test]
fn make_overhang_printable_hole_size_zero_emits_projected_gcode() {
    let (options, pipeline) = overhang_hole_fixture(
        json!({
            "make_overhang_printable": true,
            "make_overhang_printable_angle": 45,
            "make_overhang_printable_hole_size": 0,
            "gcode_comments": true
        }),
        rectangle(9.0, 9.0, 13.0, 13.0),
    );
    let gcode =
        String::from_utf8(crate::gcode::format_gcode(&pipeline, &options).unwrap()).unwrap();

    assert_eq!(lower_contours(&pipeline).len(), 3);
    assert_eq!(
        lower_contours(&pipeline)[2],
        rectangle_points(9.2, 9.2, 12.8, 12.8)
    );
    assert!(gcode.contains(";PERIMETER:external:9.2,9.2 -> 12.8,9.2 -> 12.8,12.8 -> 9.2,12.8"));
    assert!(
        gcode.contains(
            ";PRINT_PATH:external_perimeter:9.2,9.2 -> 12.8,9.2 -> 12.8,12.8 -> 9.2,12.8"
        )
    );
}

#[test]
fn make_overhang_printable_hole_size_skips_projection_covering_small_nested_rectangle() {
    let (options, pipeline) = overhang_hole_fixture(
        json!({
            "make_overhang_printable": true,
            "make_overhang_printable_angle": 45,
            "make_overhang_printable_hole_size": 4.1,
            "gcode_comments": true
        }),
        rectangle(9.0, 9.0, 13.0, 13.0),
    );
    let gcode =
        String::from_utf8(crate::gcode::format_gcode(&pipeline, &options).unwrap()).unwrap();

    assert_eq!(
        lower_contours(&pipeline),
        vec![
            rectangle_points(0.0, 0.0, 20.0, 20.0),
            rectangle_points(10.0, 10.0, 12.0, 12.0),
        ]
    );
    assert!(!gcode.contains(";PERIMETER:external:9.2,9.2 -> 12.8,9.2 -> 12.8,12.8 -> 9.2,12.8"));
    assert!(
        !gcode.contains(
            ";PRINT_PATH:external_perimeter:9.2,9.2 -> 12.8,9.2 -> 12.8,12.8 -> 9.2,12.8"
        )
    );
}

#[test]
fn make_overhang_printable_hole_size_zero_and_strict_area_threshold_keep_projection() {
    for hole_size in [0.0, 3.9, 4.0] {
        let (_, pipeline) = overhang_hole_fixture(
            json!({
                "make_overhang_printable": true,
                "make_overhang_printable_angle": 0,
                "make_overhang_printable_hole_size": hole_size
            }),
            rectangle(9.0, 9.0, 13.0, 13.0),
        );

        assert_eq!(lower_contours(&pipeline).len(), 3);
        assert_eq!(
            lower_contours(&pipeline)[2],
            rectangle_points(9.0, 9.0, 13.0, 13.0)
        );
    }
}

#[test]
fn make_overhang_printable_hole_size_keeps_projection_when_upper_does_not_cover_hole() {
    let (_, pipeline) = overhang_hole_fixture(
        json!({
            "make_overhang_printable": true,
            "make_overhang_printable_angle": 0,
            "make_overhang_printable_hole_size": 4.1
        }),
        rectangle(9.0, 9.0, 11.0, 13.0),
    );

    assert_eq!(lower_contours(&pipeline).len(), 3);
    assert_eq!(
        lower_contours(&pipeline)[2],
        rectangle_points(9.0, 9.0, 11.0, 13.0)
    );
}

#[test]
fn make_overhang_printable_hole_size_ignores_non_nested_lower_rectangles() {
    let options = options(json!({
        "make_overhang_printable": true,
        "make_overhang_printable_angle": 0,
        "make_overhang_printable_hole_size": 4.1
    }));
    let pipeline = crate::pipeline::test_support::contour_layers_pipeline_from_layers_for_tests(
        &options,
        vec![
            vec![
                rectangle(0.0, 0.0, 20.0, 20.0),
                rectangle(22.0, 22.0, 24.0, 24.0),
            ],
            vec![rectangle(21.0, 21.0, 25.0, 25.0)],
        ],
    );

    assert_eq!(lower_contours(&pipeline).len(), 3);
    assert_eq!(
        lower_contours(&pipeline)[2],
        rectangle_points(21.0, 21.0, 25.0, 25.0)
    );
}

fn options(extra: serde_json::Value) -> SliceOptions {
    let mut value = json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "wall_loops": 1,
        "line_width": 0.4,
        "sparse_infill_density": 0,
        "skirt_loops": 0,
        "brim_width": 0.0,
        "filament_max_volumetric_speed": 0.0,
        "slow_down_for_layer_cooling": false
    });
    for (key, value_extra) in extra.as_object().unwrap() {
        value[key] = value_extra.clone();
    }
    serde_json::from_value(value).unwrap()
}

fn overhang_hole_fixture(
    extra: serde_json::Value,
    upper: crate::Contour,
) -> (SliceOptions, SlicingPipeline) {
    let options = options(extra);
    let pipeline = crate::pipeline::test_support::contour_layers_pipeline_from_layers_for_tests(
        &options,
        vec![
            vec![
                rectangle(0.0, 0.0, 20.0, 20.0),
                rectangle(10.0, 10.0, 12.0, 12.0),
            ],
            vec![upper],
        ],
    );

    (options, pipeline)
}

fn rectangle(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> crate::Contour {
    crate::Contour::new(rectangle_points(min_x, min_y, max_x, max_y))
}

fn rectangle_points(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Vec<Point2> {
    vec![
        Point2::new(min_x, min_y),
        Point2::new(max_x, min_y),
        Point2::new(max_x, max_y),
        Point2::new(min_x, max_y),
    ]
}

fn lower_contours(pipeline: &SlicingPipeline) -> Vec<Vec<Point2>> {
    pipeline.layer_contours()[0]
        .contours()
        .iter()
        .map(|contour| contour.points().to_vec())
        .collect()
}

fn lower_perimeter_count(pipeline: &SlicingPipeline) -> usize {
    pipeline.layer_perimeters()[0].paths().len()
}

fn lower_print_path_count(pipeline: &SlicingPipeline) -> usize {
    pipeline.layer_print_paths()[0]
        .paths()
        .iter()
        .filter(|path| path.role() == PrintPathRole::ExternalPerimeter)
        .count()
}

fn lower_print_move_count(pipeline: &SlicingPipeline) -> usize {
    pipeline.layer_toolpath_moves()[0]
        .moves()
        .iter()
        .filter(|mov| mov.kind() == ToolpathMoveKind::Print)
        .count()
}

fn lower_extrusion_print_move_count(pipeline: &SlicingPipeline) -> usize {
    pipeline.layer_extrusion_moves()[0]
        .moves()
        .iter()
        .filter(|mov| mov.kind() == ToolpathMoveKind::Print)
        .count()
}

fn without_option_count(gcode: String) -> Vec<String> {
    gcode
        .lines()
        .filter(|line| !line.starts_with("; option_count = "))
        .map(str::to_owned)
        .collect()
}
