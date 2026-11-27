pub(in crate::project_slice) mod adaptive_fields;
pub(in crate::project_slice) mod bridge_over_infill;
pub(in crate::project_slice) mod combine_infill;
pub(in crate::project_slice) mod external_surfaces;
pub(in crate::project_slice) mod fill_surfaces;
pub(in crate::project_slice) mod horizontal_shell_promotion;
pub(in crate::project_slice) mod horizontal_shell_propagation;
pub(in crate::project_slice) mod surface_type_detection;
pub(in crate::project_slice) mod vertical_shell_assignment;
pub(in crate::project_slice) mod vertical_shell_filtering;
pub(in crate::project_slice) mod vertical_shell_projection;
pub(in crate::project_slice) mod vertical_shell_regularization;
pub(in crate::project_slice) mod vertical_shell_trimming;
pub(in crate::project_slice) mod vertical_shells;

use super::perimeters::layer_region::PreparedPostLayerRegionPerimeters;
use crate::{CancelToken, SliceError};

pub(super) fn prepare(
    walls: PreparedPostLayerRegionPerimeters,
    cancel: &CancelToken,
) -> Result<combine_infill::PreparedPostInfillCombination, SliceError> {
    let staged = surface_type_detection::prepare(walls, cancel)?;
    let staged = fill_surfaces::prepare(staged, cancel)?;
    let staged = vertical_shells::prepare(staged, cancel)?;
    let staged = vertical_shell_projection::prepare(staged, cancel)?;
    let staged = vertical_shell_trimming::prepare(staged, cancel)?;
    let staged = vertical_shell_regularization::prepare(staged, cancel)?;
    let staged = vertical_shell_filtering::prepare(staged, cancel)?;
    let staged = vertical_shell_assignment::prepare(staged, cancel)?;
    let staged = horizontal_shell_promotion::prepare(staged, cancel)?;
    let staged = horizontal_shell_propagation::prepare(staged, cancel)?;
    let external = external_surfaces::prepare(staged, cancel)?;
    let candidates = bridge_over_infill::prepare(external, cancel)?;
    let bridged = bridge_over_infill::transaction::prepare(candidates, cancel)?;
    combine_infill::prepare(bridged, cancel)
}
