//! Raft fill parameter derivation, ported from
//! `OrcaSlicer/src/libslic3r/Support/SupportParameters.hpp:100-175,284-285`
//! and the per-layer fill selection of
//! `SupportCommon.cpp:1440-1523`:
//!
//! - `support_spacing = support_base_pattern_spacing +
//!   support_material_flow.spacing()`;
//!   `support_density = min(1, flow.spacing / support_spacing)`.
//! - `raft_interface_spacing = support_interface_spacing +
//!   raft_interface_flow.spacing()`;
//!   `raft_interface_density = min(1, spacing / raft_interface_spacing)`
//!   (ironing forces 0 spacing — deferred).
//! - `base_fill_pattern = honeycomb? Honeycomb : (density > 0.95 ||
//!   with_sheath ? Rectilinear : SupportBase)`;
//!   `raft_interface_fill_pattern = density > 0.95 ? Rectilinear :
//!   SupportBase` (`:133-135`).
//! - Angles (`:147-175`): with `base_raft_layers > 1` the full stack
//!   exists — flange = interface_angle (support_angle + 90°), base =
//!   base_angle (support_angle), interface = interface_angle, rotated
//!   by +90° when `interface_raft_layers` is even so the object's 1st
//!   layer hatches perpendicularly. Per-layer interface angle
//!   alternates ±45° (`:284-285`).

use std::f64::consts::{FRAC_PI_2, PI};

/// `tree_branch_diameter_double_wall_area_scaled`
/// (`SupportParameters.hpp:186-188`): branches larger than this area get a
/// second wall; lattice units².
pub(crate) fn tree_double_wall_area(tree_support_wall_count: i32, units_per_mm: f64) -> f64 {
    match tree_support_wall_count {
        count if count > 1 => 0.1,
        0 => 0.25 * (5.0 * units_per_mm).powi(2) * PI,
        _ => f64::MAX,
    }
}

/// `rounded_rectangle_extrusion_spacing` (`Flow.cpp:200-206`):
/// `width - height·float(1 − 0.25·π)` in f32 — the double factor narrows
/// to f32 once, not an f32 π multiplication.
pub(crate) fn flow_spacing(width: f64, height: f64) -> f64 {
    let factor = (1.0 - 0.25 * std::f64::consts::PI) as f32;
    (width as f32 - height as f32 * factor) as f64
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RaftFillPattern {
    Rectilinear,
    SupportBase,
    Honeycomb,
    Concentric,
    Grid,
}

/// `SupportParameters::contact_fill_pattern` (`SupportParameters.hpp:134-144`).
pub(crate) fn contact_fill_pattern(
    object: &crate::ObjectOptions,
    interface_spacing: f64,
    raft_interface_pattern: RaftFillPattern,
) -> RaftFillPattern {
    use crate::ProcessSupportInterfacePattern;
    // `filler_raft_contact_ptr` (`SupportCommon.cpp:1569-1572`).
    if object.support_interface_top_layers.0 == 0 {
        return raft_interface_pattern;
    }
    let top_spacing = if object.support_ironing.0 {
        0.0
    } else {
        object.support_interface_spacing.0
    };
    let density = (interface_spacing / (top_spacing + interface_spacing)).min(1.0);
    let zero_gap_top =
        object.support_interface_top_layers.0 > 0 && object.support_top_z_distance.0 == 0.0;
    match object.support_interface_pattern {
        ProcessSupportInterfacePattern::Grid => RaftFillPattern::Grid,
        ProcessSupportInterfacePattern::RectilinearInterlaced => RaftFillPattern::Rectilinear,
        ProcessSupportInterfacePattern::Concentric => RaftFillPattern::Concentric,
        ProcessSupportInterfacePattern::Auto if zero_gap_top => RaftFillPattern::Concentric,
        ProcessSupportInterfacePattern::Auto | ProcessSupportInterfacePattern::Rectilinear => {
            if density > 0.95 {
                RaftFillPattern::Rectilinear
            } else {
                RaftFillPattern::SupportBase
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct RaftFillParams {
    pub(crate) support_density: f64,
    pub(crate) raft_interface_density: f64,
    pub(crate) base_fill_pattern: RaftFillPattern,
    pub(crate) interface_fill_pattern: RaftFillPattern,
    /// `with_sheath` (`SupportParameters.hpp:130`): the base layers draw a
    /// sheath perimeter too (`SupportCommon.cpp:1484`).
    pub(crate) with_sheath: bool,
    /// See [`tree_double_wall_area`].
    pub(crate) tree_double_wall_area: f64,
    /// Radians.
    pub(crate) flange_angle: f64,
    pub(crate) base_angle: f64,
    /// Base interface angle before the per-layer ±45° alternation.
    pub(crate) interface_angle: f64,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct RaftFillInputs {
    /// `support_material_flow.spacing()` in mm.
    pub(crate) support_flow_spacing: f64,
    /// `raft_interface_flow.spacing()` in mm.
    pub(crate) interface_flow_spacing: f64,
    /// `support_base_pattern_spacing` in mm.
    pub(crate) base_pattern_spacing: f64,
    /// `support_interface_spacing` in mm.
    pub(crate) interface_spacing: f64,
    /// `support_angle` in degrees.
    pub(crate) support_angle_degrees: f64,
    pub(crate) base_raft_layers: usize,
    pub(crate) interface_raft_layers: usize,
    /// `tree_support_wall_count > 0`.
    pub(crate) with_sheath: bool,
    /// `support_base_pattern == honeycomb`.
    pub(crate) honeycomb_base: bool,
    /// See [`tree_double_wall_area`].
    pub(crate) tree_double_wall_area: f64,
}

impl RaftFillInputs {
    pub(crate) fn derive(self) -> RaftFillParams {
        let support_spacing = self.base_pattern_spacing + self.support_flow_spacing;
        let support_density = (self.support_flow_spacing / support_spacing).min(1.0);
        let raft_interface_spacing = self.interface_spacing + self.interface_flow_spacing;
        let raft_interface_density =
            (self.interface_flow_spacing / raft_interface_spacing).min(1.0);

        let base_fill_pattern = if self.honeycomb_base {
            RaftFillPattern::Honeycomb
        } else if support_density > 0.95 || self.with_sheath {
            RaftFillPattern::Rectilinear
        } else {
            RaftFillPattern::SupportBase
        };
        let interface_fill_pattern = if raft_interface_density > 0.95 {
            RaftFillPattern::Rectilinear
        } else {
            RaftFillPattern::SupportBase
        };

        // Angles (`SupportParameters.hpp:147-175`). Upstream stores
        // `base_angle`/`interface_angle`/`raft_angle_*` as `float` and
        // does the ±π/2 / ±π/4 arithmetic in f32; the cos/sin used by
        // `Point::rotate` and `ExPolygonWithOffset` are f64 of that
        // f32 angle, so ares must round-trip through f32 to keep the
        // rotated intersection endpoints on the same lattice unit.
        let base_angle = f64::from((self.support_angle_degrees as f32).to_radians());
        let interface_angle = f64::from(((self.support_angle_degrees + 90.0) as f32).to_radians());
        // Upstream assigns `interface_angle + 0.5 * M_PI` to a `float`
        // field: the addition is f64 (float lhs promoted), then the
        // result is truncated to f32. Model as `f32(f64(f32) + f64)`.
        let rotate_quarter = |angle_f32_bits: f64| -> f64 {
            f64::from(((angle_f32_bits) + 0.5 * std::f64::consts::PI) as f32)
        };
        let (flange_angle, interface_angle) = if self.base_raft_layers > 1 {
            // Only the interface angle gets the even-layer rotation;
            // the flange keeps the plain interface angle
            // (`SupportParameters.hpp:153-158`).
            let adjusted = if self.interface_raft_layers.is_multiple_of(2) {
                rotate_quarter(interface_angle)
            } else {
                interface_angle
            };
            (interface_angle, adjusted)
        } else if self.base_raft_layers == 1 || self.interface_raft_layers > 1 {
            (base_angle, rotate_quarter(interface_angle))
        } else {
            (FRAC_PI_2, FRAC_PI_2)
        };

        RaftFillParams {
            support_density,
            raft_interface_density,
            base_fill_pattern,
            interface_fill_pattern,
            with_sheath: self.with_sheath,
            tree_double_wall_area: self.tree_double_wall_area,
            flange_angle,
            base_angle,
            interface_angle,
        }
    }
}

impl RaftFillParams {
    /// `raft_interface_angle(interface_id)`
    /// (`SupportParameters.hpp:284-285`): ±45° per layer parity.
    /// Upstream adds `float(±M_PI/4)` to the `float` field — f32
    /// arithmetic all the way.
    pub(crate) fn interface_angle_for_layer(&self, interface_id: usize) -> f64 {
        let toggle = if interface_id & 1 == 0 {
            std::f32::consts::FRAC_PI_4
        } else {
            -std::f32::consts::FRAC_PI_4
        };
        f64::from((self.interface_angle as f32) + toggle)
    }
}
