use crate::{LayerContours, PerimeterOptions, Point2, SliceError};

use super::{GapFillPath, LayerGapFills};

pub(super) fn generate(
    layers: &[LayerContours],
    options: PerimeterOptions,
    gap_infill_speed_mm_s: f64,
) -> Result<Vec<LayerGapFills>, SliceError> {
    let topmost_layer_id = layers.last().map(LayerContours::layer_id);
    layers
        .iter()
        .map(|layer| {
            let paths = if gap_infill_speed_mm_s <= 0.0 || options.detect_thin_wall() {
                Vec::new()
            } else {
                gap_fills_for_layer(layer, options, topmost_layer_id)?
            };
            Ok(LayerGapFills::new(layer.layer_id(), layer.print_z(), paths))
        })
        .collect()
}

fn gap_fills_for_layer(
    layer: &LayerContours,
    options: PerimeterOptions,
    topmost_layer_id: Option<usize>,
) -> Result<Vec<GapFillPath>, SliceError> {
    let effective_wall_loops = resolve_wall_loops(options, layer.layer_id(), topmost_layer_id);
    if effective_wall_loops == 0 {
        return Ok(Vec::new());
    }

    let mut paths = Vec::new();
    for contour in layer.contours() {
        if let Some(path) = gap_fill_for_contour(contour.points(), options, effective_wall_loops)? {
            paths.push(path);
        }
    }
    Ok(paths)
}

fn gap_fill_for_contour(
    points: &[Point2],
    options: PerimeterOptions,
    effective_wall_loops: u32,
) -> Result<Option<GapFillPath>, SliceError> {
    let Some((min_x, min_y, max_x, max_y)) = crate::contours::axis_aligned_rectangle_bounds(points)
    else {
        return Ok(None);
    };

    let first_internal_shrink =
        (options.external_line_width() + options.internal_line_width()) / 2.0;
    let mut last_generated_offset = 0.0;
    for loop_index in 1..effective_wall_loops {
        let shrink =
            first_internal_shrink + f64::from(loop_index - 1) * options.internal_line_width();
        if min_x + shrink < max_x - shrink && min_y + shrink < max_y - shrink {
            last_generated_offset = shrink;
        }
    }

    let next_loop_offset = if last_generated_offset == 0.0 {
        first_internal_shrink
    } else {
        last_generated_offset + options.internal_line_width()
    };
    let next_width = max_x - min_x - 2.0 * next_loop_offset;
    let next_height = max_y - min_y - 2.0 * next_loop_offset;

    match (next_width > 0.0, next_height > 0.0) {
        (true, false) => {
            let center_y = (min_y + max_y) / 2.0;
            GapFillPath::new(vec![
                Point2::new(min_x + next_loop_offset, center_y),
                Point2::new(max_x - next_loop_offset, center_y),
            ])
            .map(Some)
        }
        (false, true) => {
            let center_x = (min_x + max_x) / 2.0;
            GapFillPath::new(vec![
                Point2::new(center_x, min_y + next_loop_offset),
                Point2::new(center_x, max_y - next_loop_offset),
            ])
            .map(Some)
        }
        (true, true) | (false, false) => Ok(None),
    }
}

fn resolve_wall_loops(
    options: PerimeterOptions,
    layer_id: usize,
    topmost_layer_id: Option<usize>,
) -> u32 {
    if options.wall_loops() == 0 {
        return 0;
    }
    if layer_id == 0 && options.only_one_wall_first_layer() {
        return 1;
    }
    let mut wall_loops = options.wall_loops();
    if options.alternate_extra_wall()
        && layer_id % 2 == 1
        && options.sparse_infill_density_percent() > 0.0
    {
        wall_loops += 1;
    }
    if topmost_layer_id == Some(layer_id) && options.only_one_wall_top() && wall_loops > 1 {
        return 1;
    }
    wall_loops
}

#[cfg(test)]
mod tests {
    use crate::{
        Contour, LayerContours, PerimeterOptions, Point2, WallDirection, WallSequence,
        generate_gap_fills,
    };

    #[test]
    fn generates_x_axis_rectangular_gap_fill_when_next_loop_collapses_vertically() {
        let layers = [LayerContours::new(
            0,
            0.2,
            vec![rectangle(0.0, 0.0, 3.0, 0.7)],
        )];
        let options = PerimeterOptions::new(
            4,
            0.4,
            0.4,
            WallDirection::CounterClockwise,
            WallSequence::InnerOuter,
        );

        let gap_fills = generate_gap_fills(&layers, options, 30.0).unwrap();

        assert_eq!(gap_fills[0].paths().len(), 1);
        assert_eq!(
            gap_fills[0].paths()[0].points(),
            &[Point2::new(0.4, 0.35), Point2::new(2.6, 0.35)]
        );
    }

    #[test]
    fn generates_y_axis_rectangular_gap_fill_when_next_loop_collapses_horizontally() {
        let layers = [LayerContours::new(
            0,
            0.2,
            vec![rectangle(0.0, 0.0, 0.7, 3.0)],
        )];
        let options = PerimeterOptions::new(
            4,
            0.4,
            0.4,
            WallDirection::CounterClockwise,
            WallSequence::InnerOuter,
        );

        let gap_fills = generate_gap_fills(&layers, options, 30.0).unwrap();

        assert_eq!(
            gap_fills[0].paths()[0].points(),
            &[Point2::new(0.35, 0.4), Point2::new(0.35, 2.6)]
        );
    }

    #[test]
    fn gap_infill_speed_zero_disables_generation() {
        let layers = [LayerContours::new(
            0,
            0.2,
            vec![rectangle(0.0, 0.0, 3.0, 0.7)],
        )];
        let options = PerimeterOptions::new(
            4,
            0.4,
            0.4,
            WallDirection::CounterClockwise,
            WallSequence::InnerOuter,
        );

        let gap_fills = generate_gap_fills(&layers, options, 0.0).unwrap();

        assert!(gap_fills[0].paths().is_empty());
    }

    #[test]
    fn non_rectangular_contours_generate_no_gap_fill() {
        let layers = [LayerContours::new(
            0,
            0.2,
            vec![Contour::new(vec![
                Point2::new(0.0, 0.0),
                Point2::new(3.0, 0.0),
                Point2::new(1.5, 0.7),
            ])],
        )];
        let options = PerimeterOptions::new(
            4,
            0.4,
            0.4,
            WallDirection::CounterClockwise,
            WallSequence::InnerOuter,
        );

        let gap_fills = generate_gap_fills(&layers, options, 30.0).unwrap();

        assert!(gap_fills[0].paths().is_empty());
    }

    #[test]
    fn rectangular_bounds_accepts_rotated_clockwise_rectangle_points() {
        let points = [
            Point2::new(3.0, 0.7),
            Point2::new(3.0, 0.0),
            Point2::new(0.0, 0.0),
            Point2::new(0.0, 0.7),
        ];

        assert_eq!(
            crate::contours::axis_aligned_rectangle_bounds(&points),
            Some((0.0, 0.0, 3.0, 0.7))
        );
    }

    fn rectangle(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Contour {
        Contour::new(vec![
            Point2::new(min_x, min_y),
            Point2::new(max_x, min_y),
            Point2::new(max_x, max_y),
            Point2::new(min_x, max_y),
        ])
    }
}
