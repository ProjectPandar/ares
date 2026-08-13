use crate::geometry::{ClipperError, Polygon, difference_polygons_paths};

use super::types::CandidateSurface;

const EPSILON: f64 = 1.0e-4;

pub(in crate::project_slice) struct ClusterBridgeHistoryLayer<'a> {
    pub(in crate::project_slice) print_z: f64,
    pub(in crate::project_slice) candidates: &'a [CandidateSurface],
}

#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "retained for the unwired bridge-over-infill transaction"
    )
)]
pub(in crate::project_slice) fn subtract_filled_lower_cluster_bridges(
    deep_infill_area: &[Polygon],
    previous_cluster_layers: &[ClusterBridgeHistoryLayer<'_>],
    current_print_z: f64,
    target_flow_height: f64,
) -> Result<Vec<Polygon>, ClipperError> {
    let bottom_z = (current_print_z - target_flow_height) - EPSILON;
    let mut filled_lower_polygons = Vec::new();
    for layer in previous_cluster_layers.iter().rev() {
        if layer.print_z < bottom_z {
            break;
        }
        for candidate in layer.candidates {
            filled_lower_polygons.extend(candidate.new_polygons.iter().cloned());
        }
    }
    difference_polygons_paths(deep_infill_area, &filled_lower_polygons)
}

#[cfg(test)]
mod tests;
