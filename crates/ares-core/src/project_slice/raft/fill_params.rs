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

use std::f64::consts::{FRAC_PI_2, FRAC_PI_4, PI};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RaftFillPattern {
    Rectilinear,
    SupportBase,
    Honeycomb,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct RaftFillParams {
    pub(crate) support_density: f64,
    pub(crate) raft_interface_density: f64,
    pub(crate) base_fill_pattern: RaftFillPattern,
    pub(crate) interface_fill_pattern: RaftFillPattern,
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
            let adjusted = if self.interface_raft_layers % 2 == 0 {
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

/// Normalized angle into [0, π) for comparing line directions.
pub(crate) fn normalized_direction(angle: f64) -> f64 {
    let mut angle = angle.rem_euclid(PI);
    if angle < 0.0 {
        angle += PI;
    }
    angle
}

#[cfg(test)]
mod tests;
