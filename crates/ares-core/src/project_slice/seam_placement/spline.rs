pub(super) struct CubicSpline {
    x: Vec<f32>,
    y: Vec<f32>,
    start: f32,
    segment_size: f32,
}

impl CubicSpline {
    pub(super) fn fit(
        observations: &[(f32, f32)],
        points: &[f32],
        weights: &[f32],
        segment_count: usize,
    ) -> Self {
        assert!(!observations.is_empty());
        assert_eq!(observations.len(), points.len());
        assert_eq!(observations.len(), weights.len());
        assert!(segment_count <= observations.len());
        let start = points[0];
        let segment_size = (points[points.len() - 1] - start) / segment_count as f32;
        let parameter_count = segment_count + 1;
        let mut matrix = vec![vec![0.0; parameter_count]; observations.len()];
        for (row, (&point, &weight)) in points.iter().zip(weights).enumerate() {
            let root_weight = weight.sqrt();
            let middle_right = ((point - start) / segment_size).floor() as i32;
            let first_segment = middle_right - 1;
            for segment in first_segment..first_segment + 4 {
                let segment_start = start + segment as f32 * segment_size;
                let distance = (segment_start - point) / segment_size;
                let parameter = segment.clamp(0, parameter_count as i32 - 1) as usize;
                matrix[row][parameter] += cubic_kernel(distance) * root_weight;
            }
        }
        let observed_x = observations
            .iter()
            .zip(weights)
            .map(|(&(x, _), &weight)| x * weight.sqrt())
            .collect::<Vec<_>>();
        let observed_y = observations
            .iter()
            .zip(weights)
            .map(|(&(_, y), &weight)| y * weight.sqrt())
            .collect::<Vec<_>>();
        Self {
            x: solve_least_squares(&matrix, &observed_x),
            y: solve_least_squares(&matrix, &observed_y),
            start,
            segment_size,
        }
    }

    pub(super) fn value(&self, point: f32) -> (f32, f32) {
        let middle_right = ((point - self.start) / self.segment_size).floor() as i32;
        let first_segment = middle_right - 1;
        let mut result = (0.0, 0.0);
        for segment in first_segment..first_segment + 4 {
            let segment_start = self.start + segment as f32 * self.segment_size;
            let distance = (segment_start - point) / self.segment_size;
            let parameter = segment.clamp(0, self.x.len() as i32 - 1) as usize;
            let weight = cubic_kernel(distance);
            result.0 += weight * self.x[parameter];
            result.1 += weight * self.y[parameter];
        }
        result
    }
}

fn cubic_kernel(mut value: f32) -> f32 {
    value = value.abs();
    if value >= 2.0 {
        return 0.0;
    }
    if value <= 1.0 {
        let square = value * value;
        let cube = square * value;
        return 4.0 / 6.0 - square + (3.0 / 6.0) * cube;
    }
    value -= 1.0;
    let square = value * value;
    let cube = square * value;
    1.0 / 6.0 - (3.0 / 6.0) * value + (3.0 / 6.0) * square - (1.0 / 6.0) * cube
}

fn largest_corner_entry(matrix: &[Vec<f32>], pivot: usize) -> (usize, usize, f32) {
    let mut largest = (pivot, pivot, matrix[pivot][pivot].abs());
    for column in pivot..matrix[0].len() {
        for (row, values) in matrix.iter().enumerate().skip(pivot) {
            let value = values[column].abs();
            if value > largest.2 {
                largest = (row, column, value);
            }
        }
    }
    largest
}

fn swap_row_tails(matrix: &mut [Vec<f32>], pivot: usize, other: usize) {
    let (before_other, from_other) = matrix.split_at_mut(other);
    for (left, right) in before_other[pivot][pivot..]
        .iter_mut()
        .zip(&mut from_other[0][pivot..])
    {
        std::mem::swap(left, right);
    }
}

fn apply_householder_left(matrix: &mut [Vec<f32>], pivot: usize, tau: f32) {
    if tau == 0.0 {
        return;
    }
    let parameter_count = matrix[0].len();
    for column in pivot + 1..parameter_count {
        let tail_projection = matrix[pivot + 1..]
            .iter()
            .map(|row| row[pivot] * row[column])
            .sum::<f32>();
        let projection = (tail_projection + matrix[pivot][column]) * tau;
        matrix[pivot][column] -= projection;
        for row in &mut matrix[pivot + 1..] {
            row[column] -= row[pivot] * projection;
        }
    }
}

fn solve_least_squares(matrix: &[Vec<f32>], observed: &[f32]) -> Vec<f32> {
    let row_count = matrix.len();
    let parameter_count = matrix[0].len();
    let diagonal_count = row_count.min(parameter_count);
    let mut coefficients = matrix.to_vec();
    let mut row_transpositions = vec![0; diagonal_count];
    let mut permutation = (0..parameter_count).collect::<Vec<_>>();
    let mut householder = vec![0.0_f32; diagonal_count];
    let mut nonzero_pivots = diagonal_count;
    let mut biggest = 0.0_f32;

    for pivot in 0..diagonal_count {
        let (maximum_row, maximum_column, biggest_in_corner) =
            largest_corner_entry(&coefficients, pivot);
        if pivot == 0 {
            biggest = biggest_in_corner;
        } else if biggest_in_corner <= biggest * f32::EPSILON * diagonal_count as f32 {
            nonzero_pivots = pivot;
            break;
        }
        row_transpositions[pivot] = maximum_row;
        if maximum_row != pivot {
            swap_row_tails(&mut coefficients, pivot, maximum_row);
        }
        if maximum_column != pivot {
            for row in &mut coefficients {
                row.swap(pivot, maximum_column);
            }
            permutation.swap(pivot, maximum_column);
        }

        let tail_squared_norm = coefficients[pivot + 1..]
            .iter()
            .map(|row| row[pivot] * row[pivot])
            .sum::<f32>();
        let leading = coefficients[pivot][pivot];
        let mut beta = leading.mul_add(leading, tail_squared_norm).sqrt();
        if leading >= 0.0 {
            beta = -beta;
        }
        let tau = if tail_squared_norm <= f32::MIN_POSITIVE {
            for row in &mut coefficients[pivot + 1..] {
                row[pivot] = 0.0;
            }
            0.0
        } else {
            let denominator = leading - beta;
            for row in &mut coefficients[pivot + 1..] {
                row[pivot] /= denominator;
            }
            (beta - leading) / beta
        };
        householder[pivot] = tau;
        coefficients[pivot][pivot] = beta;

        apply_householder_left(&mut coefficients, pivot, tau);
    }

    let threshold = coefficients
        .iter()
        .enumerate()
        .take(nonzero_pivots)
        .map(|(index, row)| row[index].abs())
        .fold(0.0_f32, f32::max)
        * f32::EPSILON
        * row_count.max(parameter_count) as f32;
    let rank = (0..nonzero_pivots)
        .filter(|&index| coefficients[index][index].abs() > threshold)
        .count();
    let mut projected = observed.to_vec();
    for pivot in 0..rank {
        projected.swap(pivot, row_transpositions[pivot]);
        let tail_projection = coefficients[pivot + 1..]
            .iter()
            .zip(&projected[pivot + 1..])
            .map(|(row, value)| row[pivot] * value)
            .sum::<f32>();
        let projection = (tail_projection + projected[pivot]) * householder[pivot];
        projected[pivot] -= projection;
        for (row, value) in coefficients[pivot + 1..]
            .iter()
            .zip(&mut projected[pivot + 1..])
        {
            *value -= row[pivot] * projection;
        }
    }

    let mut solution = vec![0.0_f32; parameter_count];
    for row in (0..rank).rev() {
        let remainder = (row + 1..rank)
            .map(|column| coefficients[row][column] * solution[column])
            .sum::<f32>();
        solution[row] = (projected[row] - remainder) / coefficients[row][row];
    }
    let mut unpermuted = vec![0.0; parameter_count];
    for (column, source) in permutation.into_iter().enumerate() {
        unpermuted[source] = solution[column];
    }
    unpermuted
}

#[cfg(test)]
mod tests {
    use super::CubicSpline;

    #[test]
    fn task22o129_constant_observations_remain_constant() {
        let points = [0.2, 0.4, 0.6, 0.8, 1.0, 1.2];
        let curve = CubicSpline::fit(&[(3.0, -2.0); 6], &points, &[1.0; 6], 1);

        for point in points {
            let fitted = curve.value(point);
            assert!((fitted.0 - 3.0).abs() < 1.0e-4, "{fitted:?}");
            assert!((fitted.1 + 2.0).abs() < 1.0e-4, "{fitted:?}");
        }
    }
}
