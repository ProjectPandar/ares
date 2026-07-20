use crate::geometry::Point;

pub(crate) fn map_distances_to_compensation(
    distances: &mut [f32],
    minimum_width: f64,
    scaled_compensation: f64,
) {
    let compensated_width = minimum_width + 2.0 * scaled_compensation;
    for distance in distances {
        if f64::from(*distance) < minimum_width {
            *distance = 0.0;
        } else if f64::from(*distance) > compensated_width {
            *distance = -(scaled_compensation as f32);
        } else {
            *distance = -(*distance - minimum_width as f32) / 2.0;
        }
    }
}

pub(crate) fn smooth_compensation_banded(
    contour: &[Point],
    values: &mut [f32],
    band: f32,
    strength: f32,
    passes: usize,
) {
    let mut current = values.to_vec();
    let mut output = current.clone();
    for _ in 0..passes {
        for index in 0..current.len() {
            let previous = value_at_band(contour, &current, index, band, previous_index);
            let next = value_at_band(contour, &current, index, band, next_index);
            let laplacian = current[index] * (1.0 - strength) + 0.5 * strength * (previous + next);
            output[index] = if laplacian < current[index] {
                current[index]
            } else {
                laplacian
            };
        }
        std::mem::swap(&mut current, &mut output);
    }
    values.copy_from_slice(&current);
}

fn value_at_band(
    contour: &[Point],
    values: &[f32],
    index: usize,
    band: f32,
    step: fn(usize, usize) -> usize,
) -> f32 {
    let point = contour[index];
    let mut sample_index = step(index, contour.len());
    let mut previous_point = contour[sample_index];
    let mut value = values[sample_index];
    let length_squared = squared_distance(point, previous_point);
    if length_squared < band * band {
        let mut length = length_squared.sqrt();
        let mut previous_index = sample_index;
        sample_index = step(sample_index, contour.len());
        while sample_index != index {
            let sample_point = contour[sample_index];
            let sample_length = squared_distance(sample_point, previous_point).sqrt();
            let next_length = length + sample_length;
            if next_length > band {
                value = lerp(
                    values[previous_index],
                    values[sample_index],
                    (band - length) / sample_length,
                );
                break;
            }
            value = values[sample_index];
            previous_point = sample_point;
            length = next_length;
            previous_index = sample_index;
            sample_index = step(sample_index, contour.len());
        }
    }
    value
}

fn squared_distance(left: Point, right: Point) -> f32 {
    let x = left.x() as f32 - right.x() as f32;
    let y = left.y() as f32 - right.y() as f32;
    x * x + y * y
}

fn lerp(start: f32, end: f32, parameter: f32) -> f32 {
    (1.0 - parameter) * start + parameter * end
}

fn previous_index(index: usize, len: usize) -> usize {
    (index + len - 1) % len
}

fn next_index(index: usize, len: usize) -> usize {
    (index + 1) % len
}
