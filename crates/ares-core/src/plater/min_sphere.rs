//! The rotation pivot of `Selection::set_caches` (`Selection.cpp:2516-2531`):
//! the centre of the smallest sphere enclosing the selected volumes' hull
//! points (`Selection::get_bounding_sphere`, `Selection.cpp:1177-1210`,
//! `CGAL::Min_sphere_of_spheres_d` over `float` points). The sphere is
//! unique, so this exact incremental construction (Welzl's move-to-front
//! bounds on a fixed shuffle) finds the same centre up to float rounding.

type Point = [f64; 3];

#[derive(Clone, Copy)]
struct Ball {
    center: Point,
    squared_radius: f64,
}

impl Ball {
    fn contains(&self, point: &Point) -> bool {
        let tolerance = 1e-12 * (1.0 + self.squared_radius);
        squared_distance(&self.center, point) <= self.squared_radius + tolerance
    }
}

/// The sphere centre as `float` coordinates, as CGAL returns it.
pub(super) fn bounding_sphere_center(points: &[[f32; 3]]) -> [f64; 3] {
    let mut points = points
        .iter()
        .map(|point| point.map(f64::from))
        .collect::<Vec<_>>();
    if points.is_empty() {
        return [0.0; 3];
    }
    shuffle(&mut points);
    let ball = enclose(&points, &mut Vec::with_capacity(4));
    ball.center.map(|value| f64::from(value as f32))
}

/// The smallest ball enclosing `points` with `boundary` on its surface
/// (Welzl's move-to-front recursion; 4 boundary points fix the ball).
fn enclose(points: &[Point], boundary: &mut Vec<Point>) -> Ball {
    let (mut ball, start) = match boundary.len() {
        0 => (ball_of(&points[..1]), 1),
        4 => return ball_of(boundary),
        _ => (ball_of(boundary), 0),
    };
    for index in start..points.len() {
        if ball.contains(&points[index]) {
            continue;
        }
        boundary.push(points[index]);
        ball = enclose(&points[..index], boundary);
        boundary.pop();
    }
    ball
}

/// The smallest ball with all of `boundary` on its surface; a degenerate
/// boundary falls back to the smallest ball of its widest pair.
fn ball_of(boundary: &[Point]) -> Ball {
    match boundary {
        [a] => Ball {
            center: *a,
            squared_radius: 0.0,
        },
        [a, b] => {
            let center = std::array::from_fn(|axis| (a[axis] + b[axis]) * 0.5);
            Ball {
                center,
                squared_radius: squared_distance(&center, a),
            }
        }
        _ => circumscribed(boundary).unwrap_or_else(|| widest_pair(boundary)),
    }
}

/// The sphere through 3 or 4 points with its centre in their affine hull.
fn circumscribed(boundary: &[Point]) -> Option<Ball> {
    let origin = boundary[0];
    let vectors = boundary[1..]
        .iter()
        .map(|point| sub(point, &origin))
        .collect::<Vec<_>>();
    // Centre = origin + Σ λi·vi with vi·(centre - origin) = |vi|²/2.
    let n = vectors.len();
    let mut matrix = vec![vec![0.0; n + 1]; n];
    for row in 0..n {
        for column in 0..n {
            matrix[row][column] = dot(&vectors[row], &vectors[column]);
        }
        matrix[row][n] = 0.5 * dot(&vectors[row], &vectors[row]);
    }
    let lambdas = solve(matrix)?;
    let mut center = origin;
    for (vector, lambda) in vectors.iter().zip(lambdas) {
        for axis in 0..3 {
            center[axis] += lambda * vector[axis];
        }
    }
    let squared_radius = squared_distance(&center, &origin);
    Some(Ball {
        center,
        squared_radius,
    })
}

fn widest_pair(boundary: &[Point]) -> Ball {
    let mut best = ball_of(&boundary[..2]);
    for (index, a) in boundary.iter().enumerate() {
        for b in &boundary[index + 1..] {
            let ball = ball_of(&[*a, *b]);
            if ball.squared_radius > best.squared_radius {
                best = ball;
            }
        }
    }
    best
}

/// Gaussian elimination with partial pivoting; `None` for a singular system.
fn solve(mut matrix: Vec<Vec<f64>>) -> Option<Vec<f64>> {
    let n = matrix.len();
    for column in 0..n {
        let pivot = (column..n).max_by(|a, b| {
            matrix[*a][column]
                .abs()
                .total_cmp(&matrix[*b][column].abs())
        })?;
        if matrix[pivot][column].abs() < 1e-18 {
            return None;
        }
        matrix.swap(column, pivot);
        let (top, bottom) = matrix.split_at_mut(column + 1);
        let pivot_row = &top[column];
        for row in bottom {
            let factor = row[column] / pivot_row[column];
            for (entry, pivot) in row[column..].iter_mut().zip(&pivot_row[column..]) {
                *entry -= factor * pivot;
            }
        }
    }
    let mut solution = vec![0.0; n];
    for row in (0..n).rev() {
        let tail = (row + 1..n)
            .map(|column| matrix[row][column] * solution[column])
            .sum::<f64>();
        solution[row] = (matrix[row][n] - tail) / matrix[row][row];
    }
    Some(solution)
}

/// A fixed-seed Fisher-Yates shuffle: the incremental construction's
/// expected linear time needs a random order, the result does not depend on it.
fn shuffle(points: &mut [Point]) {
    let mut state: u64 = 0x9E37_79B9_7F4A_7C15;
    for index in (1..points.len()).rev() {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        let other = ((state >> 33) % (index as u64 + 1)) as usize;
        points.swap(index, other);
    }
}

fn sub(a: &Point, b: &Point) -> Point {
    std::array::from_fn(|axis| a[axis] - b[axis])
}

fn dot(a: &Point, b: &Point) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn squared_distance(a: &Point, b: &Point) -> f64 {
    let d = sub(a, b);
    dot(&d, &d)
}
