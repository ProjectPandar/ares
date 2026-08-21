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
        let qr = FullPivQr::factorize(&matrix);
        Self {
            x: qr.solve(&observed_x),
            y: qr.solve(&observed_y),
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

// Source boundary: Eigen 3.4.1 `FullPivHouseholderQR` (rows >= cols path used by
// OrcaSlicer `Curves.hpp:170`): corner max pivoting with column-major scan order,
// Householder elimination with Eigen's operation order, rank threshold
// `eps * size * max_pivot`, and back-substitution over the permuted system.
struct FullPivQr {
    qr: Vec<Vec<f32>>,
    h_coeffs: Vec<f32>,
    row_transpositions: Vec<usize>,
    cols_permutation: Vec<usize>,
    nonzero_pivots: usize,
    max_pivot: f32,
}

impl FullPivQr {
    fn factorize(matrix: &[Vec<f32>]) -> Self {
        let rows = matrix.len();
        let cols = matrix[0].len();
        let size = rows.min(cols);
        let mut state = FactorizeState::new(matrix, size);
        for k in 0..size {
            if !state.step(k, rows, cols) {
                break;
            }
        }
        state.finish(cols)
    }

    fn solve(&self, observed: &[f32]) -> Vec<f32> {
        let rows = self.qr.len();
        let cols = self.qr[0].len();
        let threshold = self.max_pivot * f32::EPSILON * rows.min(cols) as f32;
        let rank = (0..self.nonzero_pivots)
            .filter(|&index| self.qr[index][index].abs() > threshold)
            .count();
        let mut result = vec![0.0; cols];
        if rank == 0 {
            return result;
        }
        let mut c = observed.to_vec();
        for (k, &tau) in self.h_coeffs.iter().enumerate().take(rank) {
            c.swap(k, self.row_transpositions[k]);
            self.apply_householder_to_vector(k, tau, rows, &mut c);
        }
        for row in (0..rank).rev() {
            let remainder = sse_dot((row + 1..rank).map(|j| self.qr[row][j] * c[j]));
            c[row] = (c[row] - remainder) / self.qr[row][row];
        }
        for (index, &target) in self.cols_permutation.iter().enumerate().take(rank) {
            result[target] = c[index];
        }
        result
    }

    // applyHouseholderOnTheLeft on c: dot essential·tail, then += c[k].
    fn apply_householder_to_vector(&self, k: usize, tau: f32, rows: usize, c: &mut [f32]) {
        if tau == 0.0 {
            return;
        }
        let mut tail = sse_dot((k + 1..rows).map(|i| self.qr[i][k] * c[i]));
        tail += c[k];
        c[k] -= tau * tail;
        for (i, value) in c.iter_mut().enumerate().take(rows).skip(k + 1) {
            *value -= (tau * self.qr[i][k]) * tail;
        }
    }
}

struct FactorizeState {
    qr: Vec<Vec<f32>>,
    h_coeffs: Vec<f32>,
    row_transpositions: Vec<usize>,
    cols_transpositions: Vec<usize>,
    nonzero_pivots: usize,
    max_pivot: f32,
    biggest: f32,
    size: usize,
}

impl FactorizeState {
    fn new(matrix: &[Vec<f32>], size: usize) -> Self {
        Self {
            qr: matrix.to_vec(),
            h_coeffs: vec![0.0; size],
            row_transpositions: vec![0; size],
            cols_transpositions: vec![0; size],
            nonzero_pivots: size,
            max_pivot: 0.0,
            biggest: 0.0,
            size,
        }
    }

    // Returns false when the remaining corner is negligible (early exit).
    fn step(&mut self, k: usize, rows: usize, cols: usize) -> bool {
        let precision = f32::EPSILON * self.size as f32;
        let (biggest_in_corner, pivot_row, pivot_col) = find_pivot(&self.qr, k, rows, cols);
        if k == 0 {
            self.biggest = biggest_in_corner;
        }
        if biggest_in_corner.abs() <= self.biggest.abs() * precision {
            self.nonzero_pivots = k;
            for index in k..self.size {
                self.row_transpositions[index] = index;
                self.cols_transpositions[index] = index;
                self.h_coeffs[index] = 0.0;
            }
            return false;
        }
        self.row_transpositions[k] = pivot_row;
        self.cols_transpositions[k] = pivot_col;
        swap_row_tail(&mut self.qr, k, pivot_row, cols);
        if k != pivot_col {
            for row in self.qr.iter_mut() {
                row.swap(k, pivot_col);
            }
        }
        let tau = eliminate_column(&mut self.qr, k, &mut self.max_pivot);
        self.h_coeffs[k] = tau;
        apply_householder(&mut self.qr, k, tau);
        true
    }

    fn finish(self, cols: usize) -> FullPivQr {
        // cols_permutation: identity with transpositions applied on the right.
        let mut cols_permutation = (0..cols).collect::<Vec<_>>();
        for (k, &target) in self.cols_transpositions.iter().enumerate() {
            cols_permutation.swap(k, target);
        }
        FullPivQr {
            qr: self.qr,
            h_coeffs: self.h_coeffs,
            row_transpositions: self.row_transpositions,
            cols_permutation,
            nonzero_pivots: self.nonzero_pivots,
            max_pivot: self.max_pivot,
        }
    }
}

// Eigen maxCoeff over the bottom-right corner: column-major scan keeping the first
// strictly-greater coefficient.
fn find_pivot(qr: &[Vec<f32>], k: usize, rows: usize, cols: usize) -> (f32, usize, usize) {
    let mut best = (0.0_f32, k, k);
    for (column_offset, column) in (k..cols).enumerate() {
        for (row_offset, row) in (k..rows).enumerate() {
            let score = qr[row][column].abs();
            if column_offset == 0 && row_offset == 0 {
                best = (score, row, column);
                continue;
            }
            if score > best.0 {
                best = (score, row, column);
            }
        }
    }
    best
}

// Eigen swaps row(k).tail(cols-k): only columns >= k move, so earlier Householder
// essential entries keep their row addresses.
#[expect(clippy::needless_range_loop, reason = "two rows swap in place")]
fn swap_row_tail(qr: &mut [Vec<f32>], k: usize, pivot_row: usize, cols: usize) {
    if k == pivot_row {
        return;
    }
    for column in k..cols {
        let value = qr[pivot_row][column];
        qr[pivot_row][column] = qr[k][column];
        qr[k][column] = value;
    }
}

// makeHouseholderInPlace on column k: beta = sqrt(c0*c0 + tailSqNorm) with the two
// products rounded separately, matching Eigen's expression order. Returns tau.
fn eliminate_column(qr: &mut [Vec<f32>], k: usize, max_pivot: &mut f32) -> f32 {
    // Eigen's squaredNorm on the column segment reduces sequentially (no packets
    // for this expression); verified bit-exact against the oracle trace.
    let tail_squared_norm = qr[k + 1..]
        .iter()
        .map(|row| row[k] * row[k])
        .fold(0.0_f32, |sum, square| sum + square);
    let leading = qr[k][k];
    let mut beta = (leading * leading + tail_squared_norm).sqrt();
    if leading >= 0.0 {
        beta = -beta;
    }
    let tau = if tail_squared_norm <= f32::MIN_POSITIVE {
        for row in &mut qr[k + 1..] {
            row[k] = 0.0;
        }
        0.0
    } else {
        let denominator = leading - beta;
        for row in &mut qr[k + 1..] {
            row[k] /= denominator;
        }
        (beta - leading) / beta
    };
    qr[k][k] = beta;
    if beta.abs() > *max_pivot {
        *max_pivot = beta.abs();
    }
    tau
}

// Eigen SSE redux for float sums over dynamic-size ranges: lane-mod-4 accumulation
// followed by `predux` (((l0+l2)+(l1+l3))) and a sequential scalar tail.
fn sse_dot(products: impl ExactSizeIterator<Item = f32>) -> f32 {
    let len = products.len();
    let packeted = len / 4 * 4;
    let mut lanes = [0.0_f32; 4];
    let mut scalar = 0.0_f32;
    for (index, product) in products.enumerate() {
        if index < packeted {
            lanes[index % 4] += product;
        } else {
            scalar += product;
        }
    }
    (lanes[0] + lanes[2]) + (lanes[1] + lanes[3]) + scalar
}

// Eigen applyHouseholderOnTheLeft: tmp = essentialᵀ·bottom (dot first), then
// tmp += row(pivot); row(pivot) -= tau*tmp; bottom -= (tau*essential)·tmp.
#[expect(
    clippy::needless_range_loop,
    reason = "column and row index the same cell"
)]
fn apply_householder(matrix: &mut [Vec<f32>], pivot: usize, tau: f32) {
    if tau == 0.0 {
        return;
    }
    let rows = matrix.len();
    let cols = matrix[0].len();
    for column in pivot + 1..cols {
        let mut tail =
            sse_dot((pivot + 1..rows).map(|row| matrix[row][pivot] * matrix[row][column]));
        tail += matrix[pivot][column];
        matrix[pivot][column] -= tau * tail;
        for row in pivot + 1..rows {
            matrix[row][column] -= (tau * matrix[row][pivot]) * tail;
        }
    }
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
