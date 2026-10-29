//! Density projection for sparse anchoring.

use crate::RegionOptions;

pub(in crate::project_slice) fn projected_sparse_density(options: &RegionOptions) -> f32 {
    let density_percent = options.sparse_infill_density.0 as f32;
    (0.01_f64 * f64::from(density_percent)) as f32
}
