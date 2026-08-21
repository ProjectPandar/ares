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
        return 4.0 / 6.0 - square + 3.0 / 6.0 * square * value;
    }
    value -= 1.0;
    let square = value * value;
    1.0 / 6.0 - 3.0 / 6.0 * value + 3.0 / 6.0 * square - square * value / 6.0
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

fn solve_least_squares(matrix: &[Vec<f32>], observed: &[f32]) -> Vec<f32> {
    let row_count = matrix.len();
    let parameter_count = matrix[0].len();
    let diagonal_count = row_count.min(parameter_count);
    let mut coefficients = matrix.to_vec();
    let mut projected = observed.to_vec();
    let mut permutation = (0..parameter_count).collect::<Vec<_>>();
    let mut rank = diagonal_count;
    let mut first_pivot = 0.0_f32;

    for pivot in 0..diagonal_count {
        let (maximum_row, maximum_column, biggest) = largest_corner_entry(&coefficients, pivot);
        if pivot == 0 {
            first_pivot = biggest;
        } else if biggest <= first_pivot * f32::EPSILON * diagonal_count as f32 {
            rank = pivot;
            break;
        }
        if maximum_row != pivot {
            coefficients.swap(pivot, maximum_row);
            projected.swap(pivot, maximum_row);
        }
        if maximum_column != pivot {
            for row in &mut coefficients {
                row.swap(pivot, maximum_column);
            }
            permutation.swap(pivot, maximum_column);
        }

        let norm = coefficients[pivot..]
            .iter()
            .map(|row| row[pivot] * row[pivot])
            .sum::<f32>()
            .sqrt();
        let alpha = coefficients[pivot][pivot];
        let beta = if alpha >= 0.0 { -norm } else { norm };
        let tau = (beta - alpha) / beta;
        let denominator = alpha - beta;
        coefficients[pivot][pivot] = 1.0;
        for row in &mut coefficients[pivot + 1..] {
            row[pivot] /= denominator;
        }

        for column in pivot + 1..parameter_count {
            let projection = coefficients[pivot..]
                .iter()
                .map(|row| row[pivot] * row[column])
                .sum::<f32>()
                * tau;
            for row in &mut coefficients[pivot..] {
                row[column] -= row[pivot] * projection;
            }
        }
        let rhs_projection = coefficients[pivot..]
            .iter()
            .zip(&projected[pivot..])
            .map(|(row, value)| row[pivot] * value)
            .sum::<f32>()
            * tau;
        for (row, value) in coefficients[pivot..].iter().zip(&mut projected[pivot..]) {
            *value -= row[pivot] * rhs_projection;
        }
        coefficients[pivot][pivot] = beta;
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
