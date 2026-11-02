/// The density, multiline and spacing-adjustment fields of
/// `libslic3r/Fill/FillBase.hpp::FillParams`; other fields remain in
/// the pattern-specific rewrite slices. Spacing belongs to `Fill`, not `FillParams`.
pub(crate) struct FillParams {
    pub(crate) density: f32,
    pub(crate) multiline: i32,
    pub(crate) dont_adjust: bool,
}

pub(crate) mod adaptive;
pub(crate) mod base_support;
pub(crate) mod checked_rotate;
pub(crate) mod concentric_sparse;
pub(crate) mod connect;
pub(crate) mod cross_hatch;
pub(crate) mod cubic_trapezoidal;
pub(crate) mod grid_trapezoidal;
pub(crate) mod gyroid;
pub(crate) mod gyroid_field;
pub(crate) mod honeycomb;
pub(crate) mod join_close_ends;
pub(crate) mod lightning;
pub(crate) mod line;
pub(crate) mod locked_zag;
pub(crate) mod multiline;
pub(crate) mod multiline_offset;
pub(crate) mod plane_path;
#[cfg_attr(
    not(test),
    allow(
        dead_code,
        unused_imports,
        reason = "vertical segmentation is consumed by a later rectilinear link-graph slice"
    )
)]
pub(crate) mod rectilinear;
pub(crate) mod rectilinear_multiline;
pub(crate) mod stars_trapezoidal;
pub(crate) mod three_d_honeycomb;
pub(crate) mod tpms_d;
pub(crate) mod tpms_fk;
mod trapezoidal_tail;
pub(crate) mod triangles_trapezoidal;
