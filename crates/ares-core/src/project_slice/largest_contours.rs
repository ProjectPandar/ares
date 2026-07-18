use crate::{geometry::keep_largest_contour_only, mesh_slicer::SlicingMode};

use super::closing::PostClosingPrintObject;

pub(super) fn apply_project_largest_contours(objects: &mut [PostClosingPrintObject]) {
    let layers = objects
        .iter_mut()
        .flat_map(PostClosingPrintObject::volumes_mut)
        .flat_map(|volume| volume.layers_mut());
    for layer in layers {
        if layer.mode() == SlicingMode::PositiveLargestContour {
            keep_largest_contour_only(layer.expolygons_mut());
        }
    }
}
