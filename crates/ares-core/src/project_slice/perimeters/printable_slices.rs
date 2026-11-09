//! `LayerRegion.cpp:163-173` / `PerimeterGenerator.cpp:3189-3202` at
//! OrcaSlicer 345efec64: the perimeter generator's printable lower-layer view.

use crate::{
    ObjectOptions, OrcaFloats, ProcessPerimeterGenerator,
    geometry::{ClipperError, CoordinateScale, ExPolygon, JoinType, opening_ex},
};

pub(super) fn minimum_width(
    object: &ObjectOptions,
    nozzle_diameters: &OrcaFloats,
    spiral_mode: bool,
) -> f64 {
    let nozzle = nozzle_diameters
        .0
        .iter()
        .map(|diameter| diameter.0)
        .fold(f64::INFINITY, f64::min);
    let classic_width = nozzle / 3.0;
    match object.wall_generator {
        ProcessPerimeterGenerator::Classic => classic_width,
        ProcessPerimeterGenerator::Arachne => {
            // WallToolPathsParams stores a float before printable_slices promotes it.
            let feature = f64::from((object.min_feature_size.0 * 0.01 * nozzle) as f32);
            if spiral_mode {
                classic_width.min(feature)
            } else {
                feature
            }
        }
    }
}

pub(super) fn printable_slices(
    slices: &[ExPolygon],
    min_width: f64,
    scale: CoordinateScale,
) -> Result<Vec<ExPolygon>, ClipperError> {
    if min_width > 1e-4 {
        opening_ex(
            slices,
            ((min_width / 2.0) / scale.factor()) as f32,
            JoinType::Miter,
            3.0,
        )
    } else {
        Ok(slices.to_vec())
    }
}
