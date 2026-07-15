use crate::{Point3d, SliceError};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transform3d([[f64; 4]; 4]);

impl Transform3d {
    pub const IDENTITY: Self = Self([
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]);

    pub fn parse_3mf(value: &str) -> Result<Self, SliceError> {
        let values = parse_values::<12>(value, "3MF transform")?;
        Ok(Self([
            [values[0], values[3], values[6], values[9]],
            [values[1], values[4], values[7], values[10]],
            [values[2], values[5], values[8], values[11]],
            [0.0, 0.0, 0.0, 1.0],
        ]))
    }

    pub(crate) fn parse_row_major(value: &str) -> Result<Self, SliceError> {
        let values = parse_values::<16>(value, "row-major transform")?;
        Ok(Self([
            [values[0], values[1], values[2], values[3]],
            [values[4], values[5], values[6], values[7]],
            [values[8], values[9], values[10], values[11]],
            [values[12], values[13], values[14], values[15]],
        ]))
    }

    pub(crate) fn without_xy_translation(self) -> Self {
        let mut transform = self;
        transform.0[0][3] = 0.0;
        transform.0[1][3] = 0.0;
        transform
    }

    pub(crate) fn fixed_order_less_than(self, rhs: Self) -> bool {
        for index in 0..16 {
            let row = index % 4;
            let column = index / 4;
            if self.0[row][column] < rhs.0[row][column] {
                return true;
            }
            if self.0[row][column] > rhs.0[row][column] {
                return false;
            }
        }
        false
    }

    pub(crate) fn fixed_order_equal(self, rhs: Self) -> bool {
        for index in 0..16 {
            let row = index % 4;
            let column = index / 4;
            if self.0[row][column] != rhs.0[row][column] {
                return false;
            }
        }
        true
    }

    pub(crate) fn transform_z_f32(self, point: Point3d) -> f32 {
        self.0[2][0] as f32 * point.x as f32
            + self.0[2][1] as f32 * point.y as f32
            + self.0[2][2] as f32 * point.z as f32
            + self.0[2][3] as f32
    }

    pub fn then(self, rhs: Self) -> Self {
        let mut product = [[0.0; 4]; 4];
        for (row, output_row) in product.iter_mut().enumerate() {
            for (column, output) in output_row.iter_mut().enumerate() {
                *output = (0..4)
                    .map(|index| self.0[row][index] * rhs.0[index][column])
                    .sum();
            }
        }
        Self(product)
    }

    pub fn transform_point(self, point: Point3d) -> Point3d {
        Point3d {
            x: self.0[0][0] * point.x
                + self.0[0][1] * point.y
                + self.0[0][2] * point.z
                + self.0[0][3],
            y: self.0[1][0] * point.x
                + self.0[1][1] * point.y
                + self.0[1][2] * point.z
                + self.0[1][3],
            z: self.0[2][0] * point.x
                + self.0[2][1] * point.y
                + self.0[2][2] * point.z
                + self.0[2][3],
        }
    }
}

impl Default for Transform3d {
    fn default() -> Self {
        Self::IDENTITY
    }
}

fn parse_values<const N: usize>(value: &str, name: &str) -> Result<[f64; N], SliceError> {
    let tokens = value.split_ascii_whitespace().collect::<Vec<_>>();
    if tokens.len() != N {
        return Err(SliceError::InvalidInput(format!(
            "invalid {name}: expected {N} numbers, found {}",
            tokens.len()
        )));
    }
    let mut values = [0.0; N];
    for (output, token) in values.iter_mut().zip(tokens) {
        *output = token.parse::<f64>().map_err(|_| {
            SliceError::InvalidInput(format!("invalid {name}: {token:?} is not a number"))
        })?;
        if !output.is_finite() {
            return Err(SliceError::InvalidInput(format!(
                "invalid {name}: values must be finite"
            )));
        }
    }
    Ok(values)
}
