#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct StagedTransform3d {
    rows: [[f64; 4]; 4],
}

impl StagedTransform3d {
    pub(super) fn identity() -> Self {
        Self::from_rows([
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    pub(super) fn from_rows(rows: [[f64; 4]; 4]) -> Self {
        Self { rows }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct StagedTransform3f {
    rows: [[f32; 4]; 4],
}

impl StagedTransform3f {
    pub(super) fn from_rows(rows: [[f32; 4]; 4]) -> Self {
        Self { rows }
    }

    pub(super) fn rows(&self) -> &[[f32; 4]; 4] {
        &self.rows
    }
}

pub(super) fn staged_trafo_for_bbox(
    object_trafo: &StagedTransform3d,
    volume_trafo: &StagedTransform3d,
) -> StagedTransform3f {
    let mut rows = multiply_rows(object_trafo.rows, volume_trafo.rows);
    rows[0][3] = 0.0;
    rows[1][3] = 0.0;
    StagedTransform3f {
        rows: rows.map(|row| row.map(|value| value as f32)),
    }
}

fn multiply_rows(left: [[f64; 4]; 4], right: [[f64; 4]; 4]) -> [[f64; 4]; 4] {
    std::array::from_fn(|row| {
        std::array::from_fn(|col| {
            (0..4)
                .map(|index| left[row][index] * right[index][col])
                .sum()
        })
    })
}

const ORCA_EPSILON: f64 = 1e-4;

type Matrix3 = [[f64; 3]; 3];
type Vector3 = [f64; 3];

pub(super) fn staged_trafos_differ_in_rotation_by_z_and_mirroring_by_xy_only(
    t1: &StagedTransform3d,
    t2: &StagedTransform3d,
) -> bool {
    if (t1.translation_z() - t2.translation_z()).abs() > ORCA_EPSILON {
        return false;
    }

    let m1 = t1.linear_3x3();
    let m2 = t2.linear_3x3();
    let m = multiply_3x3(inverse_3x3(m2), m1);
    let z = column(m, 2);
    if z[0].abs() > ORCA_EPSILON || z[1].abs() > ORCA_EPSILON || (z[2] - 1.0).abs() > ORCA_EPSILON {
        return false;
    }

    let x = column(m, 0);
    let y = column(m, 1);
    if x[2].abs() > ORCA_EPSILON || y[2].abs() > ORCA_EPSILON {
        return false;
    }

    let lx2 = squared_norm(x);
    let ly2 = squared_norm(y);
    if lx2 - 1.0 > ORCA_EPSILON * ORCA_EPSILON || ly2 - 1.0 > ORCA_EPSILON * ORCA_EPSILON {
        return false;
    }

    let d = dot(x, y);
    (d * d).abs() < ORCA_EPSILON * lx2 * ly2
}

impl StagedTransform3d {
    fn translation_z(&self) -> f64 {
        self.rows[2][3]
    }

    fn linear_3x3(&self) -> Matrix3 {
        std::array::from_fn(|row| std::array::from_fn(|col| self.rows[row][col]))
    }
}

fn inverse_3x3(m: Matrix3) -> Matrix3 {
    let det = m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
        - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
        + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0]);

    [
        [
            (m[1][1] * m[2][2] - m[1][2] * m[2][1]) / det,
            (m[0][2] * m[2][1] - m[0][1] * m[2][2]) / det,
            (m[0][1] * m[1][2] - m[0][2] * m[1][1]) / det,
        ],
        [
            (m[1][2] * m[2][0] - m[1][0] * m[2][2]) / det,
            (m[0][0] * m[2][2] - m[0][2] * m[2][0]) / det,
            (m[0][2] * m[1][0] - m[0][0] * m[1][2]) / det,
        ],
        [
            (m[1][0] * m[2][1] - m[1][1] * m[2][0]) / det,
            (m[0][1] * m[2][0] - m[0][0] * m[2][1]) / det,
            (m[0][0] * m[1][1] - m[0][1] * m[1][0]) / det,
        ],
    ]
}

fn multiply_3x3(left: Matrix3, right: Matrix3) -> Matrix3 {
    std::array::from_fn(|row| {
        std::array::from_fn(|col| {
            (0..3)
                .map(|index| left[row][index] * right[index][col])
                .sum()
        })
    })
}

fn column(m: Matrix3, col: usize) -> Vector3 {
    [m[0][col], m[1][col], m[2][col]]
}

fn squared_norm(v: Vector3) -> f64 {
    dot(v, v)
}

fn dot(left: Vector3, right: Vector3) -> f64 {
    left.into_iter()
        .zip(right)
        .map(|(left, right)| left * right)
        .sum()
}
