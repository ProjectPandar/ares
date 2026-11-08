//! The single source of region-scoped `MotionOptions` field
//! resolution: `m_config.apply(print.get_print_region(idx).config())`
//! at `GCode.cpp:6136` (`extrude_perimeters`) and `:6163`
//! (`extrude_infill`) re-applies the whole region config before
//! emitting that region's entities, and the `:6516-6545` feedrate
//! switch (plus flow/pattern/wipe/scarf consumers) reads the
//! region-scoped values from `m_config`.
//!
//! `assemble` builds only non-region fields and then calls `apply`
//! for the initially resolved region (`context::prelude`), so there
//! is exactly one vocabulary for the `full.process.region` fallback
//! and the resolved override — no parallel per-field path to drift.

use super::super::{
    MotionOptions,
    helpers::{absolute, acceleration},
};
use super::scarf;
use crate::options::ProcessRegionSourceOptions;
use crate::project::effective_config::types::ResolvedProjectObject;
use crate::{ProjectSettings, RegionOptions};

/// Raw object-scoped references for `ConfigBase::get_abs_value` percentages.
/// `GCode.cpp:6414-6439` rounds only the selected role acceleration.
pub(in crate::project_slice::gcode_emit) struct RegionAccelerations {
    pub(in crate::project_slice::gcode_emit) outer_wall_acceleration: f64,
    pub(in crate::project_slice::gcode_emit) default_acceleration: f64,
}

pub(in crate::project_slice::gcode_emit) fn accelerations(
    full: &ProjectSettings,
    object: Option<&ResolvedProjectObject>,
    nozzle_index: usize,
) -> RegionAccelerations {
    RegionAccelerations {
        outer_wall_acceleration: acceleration(
            object.map(|value| &value.object),
            full.process
                .object
                .outer_wall_acceleration
                .get_at(nozzle_index),
            |value| value.outer_wall_acceleration.get_at(nozzle_index),
        ),
        default_acceleration: acceleration(
            object.map(|value| &value.object),
            full.process
                .object
                .default_acceleration
                .get_at(nozzle_index),
            |value| value.default_acceleration.get_at(nozzle_index),
        ),
    }
}

/// Region `config()` apply — `default` is the print-level
/// `full.process.region` fallback, `region` the resolved region being
/// emitted (`None` only on the initial build without any region).
///
/// Still deferred upstream consumers (`GCode.cpp`): `is_infill_first`
/// at `:6137-6141`, per-path overhang-band selection, resonance
/// avoidance. `has_sparse_infill` is refreshed here for parity with
/// the region density source; the emit loop additionally resolves it
/// per phase (`:6132-6141` ordering).
pub(in crate::project_slice::gcode_emit) fn apply(
    options: &mut MotionOptions,
    default: &ProcessRegionSourceOptions,
    region: Option<&RegionOptions>,
    travel_speed: f64,
    nozzle: f64,
) {
    options.wall_direction = region.map_or(default.wall_direction, |value| value.wall_direction);
    options.print_flow_ratio =
        region.map_or(default.print_flow_ratio.0, |value| value.print_flow_ratio.0);
    options.outer_wall_flow_ratio = region.map_or(default.outer_wall_flow_ratio.0, |value| {
        value.outer_wall_flow_ratio.0
    });
    options.inner_wall_flow_ratio = region.map_or(default.inner_wall_flow_ratio.0, |value| {
        value.inner_wall_flow_ratio.0
    });
    options.overhang_flow_ratio = region.map_or(default.overhang_flow_ratio.0, |value| {
        value.overhang_flow_ratio.0
    });
    options.sparse_infill_flow_ratio = region.map_or(default.sparse_infill_flow_ratio.0, |value| {
        value.sparse_infill_flow_ratio.0
    });
    options.internal_solid_infill_flow_ratio = region
        .map_or(default.internal_solid_infill_flow_ratio.0, |value| {
            value.internal_solid_infill_flow_ratio.0
        });
    options.gap_fill_flow_ratio = region.map_or(default.gap_fill_flow_ratio.0, |value| {
        value.gap_fill_flow_ratio.0
    });
    options.first_layer_flow_ratio = region.map_or(default.first_layer_flow_ratio.0, |value| {
        value.first_layer_flow_ratio.0
    });
    let slot = options.nozzle_config_index;
    options.inner_wall_speed = region.map_or_else(
        || default.inner_wall_speed.get_at(slot),
        |value| value.inner_wall_speed.get_at(slot),
    );
    options.outer_wall_speed = region.map_or_else(
        || default.outer_wall_speed.get_at(slot),
        |value| value.outer_wall_speed.get_at(slot),
    );
    let small_perimeter = region.map_or(default.small_perimeter_speed.get_at(slot), |value| {
        value.small_perimeter_speed.get_at(slot)
    });
    options.small_perimeter_speed =
        small_perimeter_speed(small_perimeter, options.outer_wall_speed);
    options.small_perimeter_threshold = region
        .map_or(default.small_perimeter_threshold.get_at(slot), |value| {
            value.small_perimeter_threshold.get_at(slot)
        });
    options.bridge_speed = region.map_or(default.bridge_speed.get_at(slot), |value| {
        value.bridge_speed.get_at(slot)
    });
    options.internal_bridge_speed = absolute(
        region.map_or(default.internal_bridge_speed.get_at(slot), |value| {
            value.internal_bridge_speed.get_at(slot)
        }),
        options.bridge_speed,
    );
    options.sparse_infill_speed = region
        .map_or(default.sparse_infill_speed.get_at(slot), |value| {
            value.sparse_infill_speed.get_at(slot)
        });
    options.internal_solid_infill_speed = region
        .map_or(default.internal_solid_infill_speed.get_at(slot), |value| {
            value.internal_solid_infill_speed.get_at(slot)
        });
    options.top_surface_speed = region.map_or(default.top_surface_speed.get_at(slot), |value| {
        value.top_surface_speed.get_at(slot)
    });
    options.ironing_speed = region.map_or(default.ironing_speed.0, |value| value.ironing_speed.0);
    options.gap_infill_speed = region.map_or(default.gap_infill_speed.get_at(slot), |value| {
        value.gap_infill_speed.get_at(slot)
    });
    options.enable_overhang_speed = region.map_or_else(
        || default.enable_overhang_speed.get_at(slot),
        |value| value.enable_overhang_speed.get_at(slot),
    );
    options.slowdown_for_curled_perimeters = region.map_or_else(
        || default.slowdown_for_curled_perimeters.get_at(slot),
        |value| value.slowdown_for_curled_perimeters.get_at(slot),
    );
    options.overhang_speed_bands = [
        Some(
            region.map_or(default.overhang_1_4_speed.get_at(slot), |value| {
                value.overhang_1_4_speed.get_at(slot)
            }),
        ),
        Some(
            region.map_or(default.overhang_2_4_speed.get_at(slot), |value| {
                value.overhang_2_4_speed.get_at(slot)
            }),
        ),
        Some(
            region.map_or(default.overhang_3_4_speed.get_at(slot), |value| {
                value.overhang_3_4_speed.get_at(slot)
            }),
        ),
        Some(
            region.map_or(default.overhang_4_4_speed.get_at(slot), |value| {
                value.overhang_4_4_speed.get_at(slot)
            }),
        ),
    ];
    options.bottom_surface_pattern_hilbert = super::fields::hilbert::is_hilbert(
        region.map_or(default.bottom_surface_pattern, |value| {
            value.bottom_surface_pattern
        }),
    );
    options.top_surface_pattern_hilbert =
        super::fields::hilbert::is_hilbert(region.map_or(default.top_surface_pattern, |value| {
            value.top_surface_pattern
        }));
    options.has_sparse_infill = region.map_or(default.sparse_infill_density.0 > 0.0, |value| {
        value.sparse_infill_density.0 > 0.0
    });
    options.role_based_wipe_speed = region.map_or(default.role_based_wipe_speed.0, |value| {
        value.role_based_wipe_speed.0
    });
    options.wipe_speed = absolute(
        region.map_or(default.wipe_speed, |value| value.wipe_speed),
        travel_speed,
    );
    options.wipe_on_loops = region.map_or(default.wipe_on_loops.0, |value| value.wipe_on_loops.0);
    options.wipe_inward = region.map_or(default.wipe_inward.0, |value| value.wipe_inward.0);
    options.wipe_inward_distance = region.map_or(default.wipe_inward_distance, |value| {
        value.wipe_inward_distance
    });
    options.wipe_before_external_loop = region
        .map_or(default.wipe_before_external_loop.0, |value| {
            value.wipe_before_external_loop.0
        });
    options.wall_loops = region.map_or(default.wall_loops.0, |value| value.wall_loops.0);
    options.seam_gap = absolute(
        region.map_or(default.seam_gap, |value| value.seam_gap),
        nozzle,
    );
    scarf::apply(&mut options.scarf, default, region, nozzle);
}

fn small_perimeter_speed(configured: crate::FloatOrPercent, outer_wall_speed: f64) -> f64 {
    let speed = absolute(configured, outer_wall_speed);
    if speed > 0.0 {
        speed
    } else {
        0.5 * outer_wall_speed
    }
}
