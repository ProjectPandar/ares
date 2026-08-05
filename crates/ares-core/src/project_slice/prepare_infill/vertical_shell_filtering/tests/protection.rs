use crate::{
    geometry::{CoordinateScale, ExPolygon, Point, Polygon},
    project_slice::{
        prepare_infill::{
            vertical_shell_filtering::filter,
            vertical_shell_regularization::types::VerticalShellRegularization,
            vertical_shell_trimming::types::VerticalShellTrim,
        },
        region_slices::RegionSurface,
    },
};

use super::{empty_record, rectangle};

#[test]
fn task22o23_multi_component_hole_protection_uses_flat_path_count_only() {
    let first = ExPolygon::new(
        rectangle(0, 0, 1_000, 1_000),
        vec![clockwise_rectangle(300, 300, 700, 700)],
    );
    let second = ExPolygon::new(rectangle(2_000, 0, 3_000, 1_000), Vec::new());
    let covering = ExPolygon::new(rectangle(-100, -100, 1_100, 1_100), Vec::new());
    let partial = ExPolygon::new(rectangle(50, 50, 100, 100), Vec::new());
    let splitter = ExPolygon::new(rectangle(480, -100, 520, 1_100), Vec::new());
    let in_hole = ExPolygon::new(rectangle(400, 400, 600, 600), Vec::new());
    let mut current = empty_record();
    current.fill_surfaces.extend([
        RegionSurface::internal(first),
        RegionSurface::internal(second),
    ]);

    let output = filter::filter_record(
        filter::RecordOperands {
            trim: &VerticalShellTrim {
                shell: vec![rectangle(0, 0, 1, 1)],
            },
            regularization: &VerticalShellRegularization {
                regularized_shell: vec![covering.clone(), partial, splitter, in_hole],
            },
            current: &current,
            previous_lslices: None,
            next_lslices: None,
        },
        20,
        CoordinateScale::Normal,
    )
    .unwrap();
    assert_eq!(output.filtered_shell, vec![covering]);
}

fn clockwise_rectangle(min_x: i64, min_y: i64, max_x: i64, max_y: i64) -> Polygon {
    Polygon::new(vec![
        Point::new(min_x, min_y),
        Point::new(min_x, max_y),
        Point::new(max_x, max_y),
        Point::new(max_x, min_y),
    ])
}
