use crate::{
    arachne::{ExtrusionLine, wall_toolpaths::RawWallToolPathConfig},
    geometry::{Point, Polygon, union_even_odd_polygons_paths},
};

pub(super) fn remove_small_lines(
    toolpaths: &mut [Vec<ExtrusionLine>],
    min_length_factor: f64,
    is_top_or_bottom_layer: bool,
) {
    for inset in toolpaths {
        let mut line_index = 0;
        while line_index < inset.len() {
            let line = &inset[line_index];
            let min_width = line
                .junctions
                .iter()
                .map(|junction| junction.width)
                .min()
                .unwrap();
            let threshold = if is_top_or_bottom_layer {
                min_width / 2
            } else {
                (min_width as f64 * min_length_factor) as i64
            };
            if line.is_odd && !line.is_closed && shorter_than(line, threshold) {
                inset.swap_remove(line_index);
            } else {
                line_index += 1;
            }
        }
    }
}

pub(super) fn simplify_toolpaths(
    toolpaths: &mut [Vec<ExtrusionLine>],
    config: RawWallToolPathConfig,
) {
    let maximum_resolution = config.wall_maximum_resolution;
    let maximum_deviation = config.wall_maximum_deviation;
    let maximum_extrusion_area_deviation = config.coordinate_scale.checked_scale(2.0).unwrap();
    for line in toolpaths.iter_mut().flatten() {
        line.simplify(
            maximum_resolution * maximum_resolution,
            maximum_deviation * maximum_deviation,
            maximum_extrusion_area_deviation,
            config.coordinate_scale,
        );
    }
}

pub(super) fn separate_inner_contour(toolpaths: &mut Vec<Vec<ExtrusionLine>>) -> Vec<Polygon> {
    let mut actual_toolpaths = Vec::with_capacity(toolpaths.len());
    let mut inner_contour = Vec::new();
    for inset in std::mem::take(toolpaths) {
        let Some(last_line) = inset.last() else {
            continue;
        };
        let is_contour = last_line.junctions.first().unwrap().width == 0;
        if is_contour {
            debug_assert!(
                inset
                    .iter()
                    .flat_map(|line| &line.junctions)
                    .all(|junction| junction.width == 0)
            );
            inner_contour.extend(
                inset
                    .iter()
                    .filter(|line| !line.is_odd && line.is_closed)
                    .map(ExtrusionLine::to_polygon),
            );
        } else {
            actual_toolpaths.push(inset);
        }
    }
    *toolpaths = actual_toolpaths;
    union_even_odd_polygons_paths(&inner_contour).unwrap()
}
fn shorter_than(line: &ExtrusionLine, threshold: i64) -> bool {
    let mut previous = line.junctions.last().unwrap().point;
    let mut length = 0;
    for junction in &line.junctions {
        length += distance(previous, junction.point);
        if length >= threshold {
            return false;
        }
        previous = junction.point;
    }
    true
}

fn distance(left: Point, right: Point) -> i64 {
    let dx = i128::from(left.x() - right.x());
    let dy = i128::from(left.y() - right.y());
    ((dx * dx + dy * dy) as f64).sqrt() as i64
}
