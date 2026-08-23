// Source boundary: Eigen 3.4.1 `FullPivHouseholderQR` (rows >= cols path used by
// OrcaSlicer `Curves.hpp:170`): corner max pivoting with column-major scan order,
// Householder elimination with Eigen's operation order, rank threshold
// `eps * size * max_pivot`, and back-substitution over the permuted system.
pub(super) struct FullPivQr {
    qr: Vec<Vec<f32>>,
    h_coeffs: Vec<f32>,
    row_transpositions: Vec<usize>,
    cols_permutation: Vec<usize>,
    nonzero_pivots: usize,
    max_pivot: f32,
}

impl FullPivQr {
    pub(super) fn factorize(matrix: &[Vec<f32>]) -> Self {
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

    pub(super) fn solve(&self, observed: &[f32]) -> Vec<f32> {
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
        solve_upper_triangular(&self.qr, rank, &mut c);
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
        let dot = sse_dot((k + 1..rows).map(|i| self.qr[i][k] * c[i]));
        let tail = dot + c[k];
        c[k] -= tau * tail;

        for (i, value) in c.iter_mut().enumerate().take(rows).skip(k + 1) {
            *value -= (tau * self.qr[i][k]) * tail;
        }
    }
}

// Eigen's column-major vector solver handles eight diagonal columns per panel:
// backward substitution within the panel, then one matrix-vector update above it.
pub(super) fn solve_upper_triangular(qr: &[Vec<f32>], rank: usize, right_hand_side: &mut [f32]) {
    const PANEL_WIDTH: usize = 8;
    let mut panel_end = rank;
    while panel_end > 0 {
        let panel_start = panel_end.saturating_sub(PANEL_WIDTH);
        for column in (panel_start..panel_end).rev() {
            if right_hand_side[column] == 0.0 {
                continue;
            }
            right_hand_side[column] /= qr[column][column];
            for row in panel_start..column {
                right_hand_side[row] -= right_hand_side[column] * qr[row][column];
            }
        }
        for row in 0..panel_start {
            let mut update = 0.0;
            for column in panel_start..panel_end {
                update += qr[row][column] * right_hand_side[column];
            }
            right_hand_side[row] += -update;
        }
        panel_end = panel_start;
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

// Eigen's dynamic float squaredNorm uses its two-accumulator SSE redux, aligned
// to the source column address. Returns tau after the in-place elimination.
fn eliminate_column(qr: &mut [Vec<f32>], k: usize, max_pivot: &mut f32) -> f32 {
    let rows = qr.len();
    let tail_len = rows - k - 1;
    let source_offset = k * rows + k + 1;
    let aligned_start = (4 - source_offset % 4) % 4;
    let tail_squared_norm = eigen_sse_redux_sum(tail_len, aligned_start, |index| {
        let value = qr[k + 1 + index][k];
        value * value
    });
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

fn eigen_sse_redux_sum(
    len: usize,
    aligned_start: usize,
    mut value: impl FnMut(usize) -> f32,
) -> f32 {
    if len == 0 {
        return 0.0;
    }
    let aligned_start = aligned_start.min(len);
    let aligned_size_two = (len - aligned_start) / 8 * 8;
    let aligned_size = (len - aligned_start) / 4 * 4;
    if aligned_size == 0 {
        let mut result = value(0);
        for index in 1..len {
            result += value(index);
        }
        return result;
    }

    let aligned_end_two = aligned_start + aligned_size_two;
    let aligned_end = aligned_start + aligned_size;
    let mut first = sse_packet(aligned_start, &mut value);
    if aligned_size > 4 {
        let mut second = sse_packet(aligned_start + 4, &mut value);
        let mut index = aligned_start + 8;
        while index < aligned_end_two {
            first = add_sse_packets(first, sse_packet(index, &mut value));
            second = add_sse_packets(second, sse_packet(index + 4, &mut value));
            index += 8;
        }
        first = add_sse_packets(first, second);
        if aligned_end > aligned_end_two {
            first = add_sse_packets(first, sse_packet(aligned_end_two, &mut value));
        }
    }
    let mut result = (first[0] + first[2]) + (first[1] + first[3]);
    for index in 0..aligned_start {
        result += value(index);
    }
    for index in aligned_end..len {
        result += value(index);
    }
    result
}

fn sse_packet(start: usize, value: &mut impl FnMut(usize) -> f32) -> [f32; 4] {
    [
        value(start),
        value(start + 1),
        value(start + 2),
        value(start + 3),
    ]
}

fn add_sse_packets(left: [f32; 4], right: [f32; 4]) -> [f32; 4] {
    [
        left[0] + right[0],
        left[1] + right[1],
        left[2] + right[2],
        left[3] + right[3],
    ]
}

// Eigen's runtime vector redux uses two interleaved packet accumulators before
// `predux`; cwise products have no direct-access alignment prefix.
fn sse_dot(mut products: impl ExactSizeIterator<Item = f32>) -> f32 {
    let len = products.len();
    let packeted_two = len / 8 * 8;
    let packeted = len / 4 * 4;
    if packeted == 0 {
        let mut result = products.next().unwrap_or(0.0);
        for product in products {
            result += product;
        }
        return result;
    }

    let mut first = next_sse_packet(&mut products);
    if packeted > 4 {
        let mut second = next_sse_packet(&mut products);
        let mut consumed = 8;
        while consumed < packeted_two {
            first = add_sse_packets(first, next_sse_packet(&mut products));
            second = add_sse_packets(second, next_sse_packet(&mut products));
            consumed += 8;
        }
        first = add_sse_packets(first, second);
        if packeted > packeted_two {
            first = add_sse_packets(first, next_sse_packet(&mut products));
        }
    }
    let mut result = (first[0] + first[2]) + (first[1] + first[3]);
    for product in products {
        result += product;
    }
    result
}

fn next_sse_packet(products: &mut impl Iterator<Item = f32>) -> [f32; 4] {
    [
        products.next().expect("packet has four products"),
        products.next().expect("packet has four products"),
        products.next().expect("packet has four products"),
        products.next().expect("packet has four products"),
    ]
}

fn sse_dot_single(products: impl ExactSizeIterator<Item = f32>) -> f32 {
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

// Eigen evaluates a one-column remainder as a vector redux with two packet
// accumulators; wider remainders use its matrix-product path with one per column.
// The update order remains tmp += top, top -= tau*tmp, bottom -= tau*essential*tmp.
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
        let products = || (pivot + 1..rows).map(|row| matrix[row][pivot] * matrix[row][column]);
        let mut tail = if cols - pivot - 1 == 1 {
            sse_dot(products())
        } else {
            sse_dot_single(products())
        };
        tail += matrix[pivot][column];
        matrix[pivot][column] -= tau * tail;
        for row in pivot + 1..rows {
            matrix[row][column] -= (tau * matrix[row][pivot]) * tail;
        }
    }
}
