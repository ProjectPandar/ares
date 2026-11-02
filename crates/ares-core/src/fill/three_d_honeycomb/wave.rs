//! `Fill3DHoneycomb.cpp:41–87` wave phase and critical points.

pub(super) fn sign(value: f64) -> f64 {
    f64::from(i32::from(0.0 < value) - i32::from(value < 0.0))
}

pub(super) fn tri(position: f64, grid: f64) -> f64 {
    let mut t = (position / (grid * 2.0) + 0.25) as f32;
    t -= t as i32 as f32;
    (1.0 - (f64::from(t) * 8.0 - 4.0).abs()) * (grid / 4.0) + grid / 4.0
}

pub(super) fn troct(position: f64, grid: f64, z: f64) -> f64 {
    let offset = tri(z, grid) / 2.0;
    let y = tri(position, grid);
    if y.abs() > offset.abs() {
        sign(y) * offset
    } else {
        y * sign(offset)
    }
}

pub(super) fn critical(z: f64, grid: f64) -> Vec<f64> {
    let normalized = (tri(z, grid) / 2.0).abs() / grid;
    if normalized > 0.0 {
        vec![
            grid * normalized,
            grid * (1.0 - normalized),
            grid * (1.0 + normalized),
            grid * (2.0 - normalized),
        ]
    } else {
        Vec::new()
    }
}
