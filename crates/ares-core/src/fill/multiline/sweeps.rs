//! `multiline` per-pattern sweep lists (`FillRectilinear.cpp` sweep angles),
//! split from `multiline.rs` for the <400 LOC rule. Shared by `fill_entities`
//! emit and `sparse_anchoring`.

use crate::geometry::CoordinateScale;

use super::Sweep;

/// `FillStars`/`ipStars` (`FillRectilinear.cpp:3487`): three 60° families, the
/// third shifted by `(3/2)·spacing·multiline/density`.
pub(crate) fn stars_sweeps(spacing: f64, multiline: i32, density: f32) -> [Sweep; 3] {
    let third_shift = (1.5 * spacing * f64::from(multiline)) as f32 / density;
    [
        Sweep {
            angle: 0.0,
            shift: 0.0,
        },
        Sweep {
            angle: std::f32::consts::FRAC_PI_3,
            shift: 0.0,
        },
        Sweep {
            angle: 2.0 * std::f32::consts::FRAC_PI_3,
            shift: third_shift,
        },
    ]
}

/// `FillQuarterCubic` (`FillRectilinear.cpp:3518`): four families `{0,±dx1},
/// {π/2,±dx2}` with the z-dependent `dx` wrapped into the tetrahedral period.
pub(crate) fn quartercubic_sweeps(
    spacing: f64,
    multiline: i32,
    density: f32,
    print_z: f64,
    scale: CoordinateScale,
) -> [Sweep; 4] {
    // Upstream promotes Boost's float constant, but scale_ remains double
    // until the explicit coord_t casts below.
    const ONE_DIV_ROOT_TWO: f64 = std::f32::consts::FRAC_1_SQRT_2 as f64;
    let z_scaled = scale.scaled_delta(print_z);
    let spacing_scaled = scale.scaled_delta(spacing);
    let line_width = spacing_scaled as i64 * i64::from(multiline);
    let period = (spacing_scaled * f64::from(multiline) / f64::from(density)) as i64 * 4;
    let shift_for = |pattern_z_shift: f64| -> f32 {
        let mut shift =
            (ONE_DIV_ROOT_TWO * (z_scaled + pattern_z_shift * period as f64 * 2.0)) as i64 % period;
        shift = shift.min(period - shift);
        shift = shift.min(period / 2 - line_width / 2);
        shift = shift.max(line_width / 2);
        scale.unscale(shift) as f32
    };
    let (dx1, dx2) = (shift_for(0.0), shift_for(0.5));
    [
        Sweep {
            angle: 0.0,
            shift: dx1,
        },
        Sweep {
            angle: 0.0,
            shift: -dx1,
        },
        Sweep {
            angle: std::f32::consts::FRAC_PI_2,
            shift: dx2,
        },
        Sweep {
            angle: std::f32::consts::FRAC_PI_2,
            shift: -dx2,
        },
    ]
}

/// `FillLateralLattice` (`FillRectilinear.cpp:3449`): two 90° families shifted
/// by `tan(lateral_lattice_angle_{1,2})·z`.
pub(crate) fn lateral_lattice_sweeps(angle_1: f32, angle_2: f32, print_z: f64) -> [Sweep; 2] {
    let dx1 = (f64::from(angle_1).to_radians().tan() * print_z) as f32;
    let dx2 = (f64::from(angle_2).to_radians().tan() * print_z) as f32;
    [
        Sweep {
            angle: std::f32::consts::FRAC_PI_2,
            shift: dx1,
        },
        Sweep {
            angle: std::f32::consts::FRAC_PI_2,
            shift: dx2,
        },
    ]
}

/// `FillLateralHoneycomb` (`FillRectilinear.cpp:3550`): the inverted-Y
/// honeycomb — a single 90° family on `single_line` layers, a symmetric double
/// family otherwise, each with its density rescaled. Returns `(sweeps,
/// density)`.
pub(crate) fn lateral_honeycomb_sweeps(
    spacing: f64,
    multiline: i32,
    density: f32,
    overhang_angle_deg: f32,
    print_z: f64,
) -> (Vec<Sweep>, f32) {
    const DEGREE: f32 = std::f32::consts::PI / 180.0;
    let half_horizontal_period =
        0.5 * (1.0 * (2.0 / 3.0) + 2.0 * (1.0 / 3.0)) * spacing as f32 * multiline as f32 / density;
    let vertical_period = 3.0 * half_horizontal_period / (DEGREE * overhang_angle_deg).tan();
    let vertical_thirds_float = 3.0 * print_z as f32 / vertical_period;
    let vertical_thirds_int = vertical_thirds_float as i32;
    let single_line = (vertical_thirds_int + 1) % 3 != 0;
    let odd_layer = ((vertical_thirds_int + 1) / 3) % 2 != 0;
    let horizontal_offset = if odd_layer {
        half_horizontal_period
    } else {
        0.0
    };
    let normalizer = 1.0 * (2.0 / 3.0) + 2.0 * (1.0 / 3.0);
    if single_line {
        (
            vec![Sweep {
                angle: std::f32::consts::FRAC_PI_2,
                shift: horizontal_offset,
            }],
            density * (1.0 / normalizer) as f32,
        )
    } else {
        let horizontal_position =
            (1.0 - (vertical_thirds_float - vertical_thirds_int as f32)) * half_horizontal_period;
        (
            vec![
                Sweep {
                    angle: std::f32::consts::FRAC_PI_2,
                    shift: -horizontal_position + horizontal_offset,
                },
                Sweep {
                    angle: std::f32::consts::FRAC_PI_2,
                    shift: horizontal_position + horizontal_offset,
                },
            ],
            density * (2.0 / normalizer) as f32,
        )
    }
}
