//! `FillLightning` port (`src/libslic3r/Fill/Lightning/*`): the lightning
//! infill generator — a per-object tree grown top-down through the layer stack
//! then converted to lines per surface. `DistanceField` samples the overhang;
//! `Layer`/`TreeNode` build + propagate the support tree; `Generator`
//! orchestrates across all layers.

mod distance_field;
