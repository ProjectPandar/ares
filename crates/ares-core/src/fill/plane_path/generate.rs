use super::{PlanePathPattern, output::InfillPolylineOutput};
use crate::geometry::{ClipperError, Point};

#[expect(
    clippy::too_many_arguments,
    reason = "the source generator receives explicit integer bounds, resolution, and output state"
)]
pub(super) fn generate(
    pattern: PlanePathPattern,
    min_x: i64,
    min_y: i64,
    max_x: i64,
    max_y: i64,
    resolution: f64,
    output: &mut InfillPolylineOutput,
) -> Result<(), ClipperError> {
    match pattern {
        PlanePathPattern::HilbertCurve => generate_hilbert(min_x, min_y, max_x, max_y, output),
        PlanePathPattern::ArchimedeanChords => {
            generate_archimedean(max_x, max_y, resolution, output)
        }
        PlanePathPattern::OctagramSpiral => generate_octagram(max_x, max_y, output),
    }
}

// `FillPlanePath.cpp:212-258`, including the source state tables and its
// parity-dependent initial transpose.
fn generate_hilbert(
    min_x: i64,
    min_y: i64,
    max_x: i64,
    max_y: i64,
    output: &mut InfillPolylineOutput,
) -> Result<(), ClipperError> {
    let required =
        (i128::from(max_x) + 1 - i128::from(min_x)).max(i128::from(max_y) + 1 - i128::from(min_y));
    let required = usize::try_from(required).map_err(|_| ClipperError::CoordinateOutOfRange)?;
    let mut size = 2_usize;
    while size < required {
        size = size
            .checked_mul(2)
            .ok_or(ClipperError::CoordinateOutOfRange)?;
    }
    let count = size
        .checked_mul(size)
        .ok_or(ClipperError::CoordinateOutOfRange)?;
    output.reserve(count);
    for index in 0..count {
        let point = hilbert_n_to_xy(index);
        let x = point
            .x()
            .checked_add(min_x)
            .ok_or(ClipperError::CoordinateOutOfRange)?;
        let y = point
            .y()
            .checked_add(min_y)
            .ok_or(ClipperError::CoordinateOutOfRange)?;
        output.add_point(x as f64, y as f64)?;
    }
    Ok(())
}

fn hilbert_n_to_xy(index: usize) -> Point {
    const NEXT_STATE: [usize; 16] = [4, 0, 0, 12, 0, 4, 4, 8, 12, 8, 8, 4, 8, 12, 12, 0];
    const DIGIT_TO_X: [i64; 16] = [0, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 1, 1, 1, 0, 0];
    const DIGIT_TO_Y: [i64; 16] = [0, 0, 1, 1, 0, 1, 1, 0, 1, 1, 0, 0, 1, 0, 0, 1];

    let mut digits = 0;
    let mut remaining = index;
    while remaining > 0 {
        remaining >>= 2;
        digits += 1;
    }
    let mut state = if digits & 1 == 1 { 4 } else { 0 };
    let mut x = 0;
    let mut y = 0;
    for digit_index in (0..digits).rev() {
        let digit = (index >> (digit_index * 2)) & 3;
        let table_index = state + digit;
        x |= DIGIT_TO_X[table_index] << digit_index;
        y |= DIGIT_TO_Y[table_index] << digit_index;
        state = NEXT_STATE[table_index];
    }
    Point::new(x, y)
}

// `FillPlanePath.cpp:168-193`: r = 1 + theta / (2 pi), with the source
// chord-error increment and point insertion order.
fn generate_archimedean(
    max_x: i64,
    max_y: i64,
    resolution: f64,
    output: &mut InfillPolylineOutput,
) -> Result<(), ClipperError> {
    let mut radius = 1.0;
    let maximum_radius = ((max_x as f64) * (max_x as f64) + (max_y as f64) * (max_y as f64)).sqrt()
        * 2.0_f64.sqrt()
        + 1.5;
    let radius_per_radian = 1.0 / (2.0 * std::f64::consts::PI);
    let mut theta = 0.0;
    output.add_point(0.0, 0.0)?;
    output.add_point(1.0, 0.0)?;
    while radius < maximum_radius {
        theta += 2.0 * (1.0 - resolution / radius).acos();
        radius = 1.0 + radius_per_radian * theta;
        output.add_point(radius * theta.cos(), radius * theta.sin())?;
    }
    Ok(())
}

// `FillPlanePath.cpp:267-306`: retain the exact sixteen-point ring order and
// the extended final right-hand chord.
fn generate_octagram(
    max_x: i64,
    max_y: i64,
    output: &mut InfillPolylineOutput,
) -> Result<(), ClipperError> {
    let maximum_radius = ((max_x as f64) * (max_x as f64) + (max_y as f64) * (max_y as f64)).sqrt()
        * 2.0_f64.sqrt()
        + 1.5;
    let radius_increment = 2.0_f64.sqrt();
    let mut radius = 0.0;
    output.add_point(0.0, 0.0)?;
    while radius < maximum_radius {
        radius += radius_increment;
        let diagonal = radius / 2.0_f64.sqrt();
        let outer = radius + diagonal;
        for (x, y) in [
            (radius, 0.0),
            (outer, diagonal),
            (diagonal, diagonal),
            (diagonal, outer),
            (0.0, radius),
            (-diagonal, outer),
            (-diagonal, diagonal),
            (-outer, diagonal),
            (-radius, 0.0),
            (-outer, -diagonal),
            (-diagonal, -diagonal),
            (-diagonal, -outer),
            (0.0, -radius),
            (diagonal, -outer),
            (diagonal, -diagonal),
            (outer + radius_increment, -diagonal),
        ] {
            output.add_point(x, y)?;
        }
    }
    Ok(())
}
