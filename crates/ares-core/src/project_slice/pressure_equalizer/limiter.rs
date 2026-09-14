//! `adjust_volumetric_rate` (`PressureEqualizer.cpp:733-848`): the
//! backward/forward volumetric-rate slope limiter over a window of
//! extrusion lines.

use super::line::GCodeLine;
use crate::ExtrusionRole;

/// Per-role slope limits (`ExtrusionRateSlope` × `erCount`). Index by the
/// upstream role discriminant (`role_index`).
#[derive(Clone, Copy, Debug)]
pub(crate) struct RateSlope {
    pub(crate) positive: f32,
    pub(crate) negative: f32,
}

pub(crate) fn role_index(role: ExtrusionRole) -> usize {
    // Mirrors the upstream `ExtrusionRole` enumerators 1..erCount used to
    // index `m_max_volumetric_extrusion_rate_slopes` (1-based in the C++
    // loops; keep the same discriminants as the role markers).
    match role {
        ExtrusionRole::None => 0,
        ExtrusionRole::Perimeter => 1,
        ExtrusionRole::ExternalPerimeter => 2,
        ExtrusionRole::OverhangPerimeter => 3,
        ExtrusionRole::InternalInfill => 4,
        ExtrusionRole::SolidInfill => 5,
        ExtrusionRole::TopSolidInfill => 6,
        ExtrusionRole::BottomSurface => 7,
        ExtrusionRole::Ironing => 8,
        ExtrusionRole::BridgeInfill => 9,
        ExtrusionRole::InternalBridgeInfill => 10,
        ExtrusionRole::GapFill => 11,
        ExtrusionRole::Skirt => 12,
        ExtrusionRole::Brim => 13,
        ExtrusionRole::SupportMaterial => 14,
        ExtrusionRole::SupportMaterialInterface => 15,
        ExtrusionRole::SupportTransition => 16,
        ExtrusionRole::WipeTower => 17,
        ExtrusionRole::Custom => 18,
        ExtrusionRole::Mixed => 19,
    }
}

const ROLE_COUNT: usize = 20;
const UNLIMITED: f32 = f32::MAX;

/// `adjust_volumetric_rate(first_line_idx, last_line_idx)`: limit the
/// volumetric rate slopes of `lines[first..=last]` (a continuous
/// extrusion segment window) so that no line's entry/exit rate exceeds
/// the configured slope relative to its extruding neighbours.
mod tests;

pub(crate) fn adjust_volumetric_rate(
    lines: &mut [GCodeLine],
    slopes: &[RateSlope; ROLE_COUNT],
    smoothing_external_perimeter_only: bool,
    first_line_idx: usize,
    last_line_idx: usize,
) {
    // Don't bother adjusting volumetric rate if there's no gcode to adjust.
    if last_line_idx - first_line_idx < 2 {
        return;
    }

    let mut line_idx = last_line_idx;
    if line_idx == first_line_idx || !lines[line_idx].extruding() {
        // Nothing to do, the last move is not extruding.
        return;
    }

    let mut feedrate_per_role = [UNLIMITED; ROLE_COUNT];
    feedrate_per_role[role_index(lines[line_idx].extrusion_role)] =
        lines[line_idx].volumetric_extrusion_rate_start;

    // Backward pass: limit the exit rates of earlier lines so they can
    // decelerate into the current one.
    while line_idx != first_line_idx {
        let mut idx_prev = line_idx - 1;
        while !lines[idx_prev].extruding() && idx_prev != first_line_idx {
            idx_prev -= 1;
        }
        if !lines[idx_prev].extruding() {
            break;
        }
        // Don't decelerate before ironing.
        if lines[line_idx].extrusion_role == ExtrusionRole::Ironing {
            line_idx = idx_prev;
            continue;
        }
        let rate_succ = lines[line_idx].volumetric_extrusion_rate_start;
        line_idx = idx_prev;

        for i_role in 1..ROLE_COUNT {
            let rate_slope = slopes[i_role].negative;
            if rate_slope == 0.0 || feedrate_per_role[i_role] == UNLIMITED {
                continue;
            }
            let line = &mut lines[line_idx];
            let mut rate_end = feedrate_per_role[i_role];
            if i_role == role_index(line.extrusion_role) && rate_succ < rate_end {
                rate_end = rate_succ;
            }
            if flow_not_adjustable(line, smoothing_external_perimeter_only) {
                rate_end = line.volumetric_extrusion_rate_end;
            } else if line.volumetric_extrusion_rate_end > rate_end {
                line.volumetric_extrusion_rate_end = rate_end;
                line.max_volumetric_extrusion_rate_slope_negative = rate_slope;
                line.modified = true;
            } else if i_role == role_index(line.extrusion_role) {
                rate_end = line.volumetric_extrusion_rate_end;
            }

            if line.adjustable_flow {
                let rate_start = (rate_end * rate_end
                    + 2.0 * line.volumetric_extrusion_rate * line.dist_xyz() * rate_slope
                        / line.feedrate())
                .sqrt();
                if rate_start < line.volumetric_extrusion_rate_start {
                    line.volumetric_extrusion_rate_start = rate_start;
                    line.max_volumetric_extrusion_rate_slope_negative = rate_slope;
                    line.modified = true;
                }
            }
            let line = &lines[line_idx];
            if line.extrusion_role != ExtrusionRole::Ironing {
                feedrate_per_role[i_role] = line.volumetric_extrusion_rate_start;
            }
        }
    }

    let mut feedrate_per_role = [UNLIMITED; ROLE_COUNT];
    feedrate_per_role[role_index(lines[line_idx].extrusion_role)] =
        lines[line_idx].volumetric_extrusion_rate_end;

    // Forward pass: limit the entry rates of later lines so they can
    // accelerate from the current one.
    while line_idx != last_line_idx {
        let mut idx_next = line_idx + 1;
        while !lines[idx_next].extruding() && idx_next != last_line_idx {
            idx_next += 1;
        }
        if !lines[idx_next].extruding() {
            break;
        }
        // Don't accelerate after ironing.
        if lines[line_idx].extrusion_role == ExtrusionRole::Ironing {
            line_idx = idx_next;
            continue;
        }
        let rate_prec = lines[line_idx].volumetric_extrusion_rate_end;
        line_idx = idx_next;

        for i_role in 1..ROLE_COUNT {
            let rate_slope = slopes[i_role].positive;
            if rate_slope == 0.0 || feedrate_per_role[i_role] == UNLIMITED {
                continue;
            }
            let line = &mut lines[line_idx];
            let mut rate_start = feedrate_per_role[i_role];
            if flow_not_adjustable(line, smoothing_external_perimeter_only) {
                rate_start = line.volumetric_extrusion_rate_start;
            } else if i_role == role_index(line.extrusion_role) && rate_prec < rate_start {
                rate_start = rate_prec;
            }

            if line.volumetric_extrusion_rate_start > rate_start {
                line.volumetric_extrusion_rate_start = rate_start;
                line.max_volumetric_extrusion_rate_slope_positive = rate_slope;
                line.modified = true;
            } else if i_role == role_index(line.extrusion_role) {
                rate_start = line.volumetric_extrusion_rate_start;
            }

            if line.adjustable_flow {
                let rate_end = (rate_start * rate_start
                    + 2.0 * line.volumetric_extrusion_rate * line.dist_xyz() * rate_slope
                        / line.feedrate())
                .sqrt();
                if rate_end < line.volumetric_extrusion_rate_end {
                    line.volumetric_extrusion_rate_end = rate_end;
                    line.max_volumetric_extrusion_rate_slope_positive = rate_slope;
                    line.modified = true;
                }
            }
            let line = &lines[line_idx];
            if line.extrusion_role != ExtrusionRole::Ironing {
                feedrate_per_role[i_role] = line.volumetric_extrusion_rate_end;
            }
        }
    }
}

/// The shared "don't alter the flow rate" guard from both passes
/// (`PressureEqualizer.cpp:765-770, 805-810`).
fn flow_not_adjustable(line: &GCodeLine, smoothing_external_perimeter_only: bool) -> bool {
    !line.adjustable_flow
        || line.extrusion_role == ExtrusionRole::BridgeInfill
        || line.extrusion_role == ExtrusionRole::Ironing
        || (smoothing_external_perimeter_only
            && line.extrusion_role != ExtrusionRole::OverhangPerimeter
            && line.extrusion_role != ExtrusionRole::ExternalPerimeter)
}
