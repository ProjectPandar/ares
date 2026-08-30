// OrcaSlicer v2.4.2 `Feature/FuzzySkin/FuzzySkin.cpp::apply_fuzzy_skin`
// classic-polygon seam after surface simplification and before overhang clipping.

#[cfg(test)]
mod tests;

use crate::{
    SliceError,
    geometry::{CoordinateScale, Polygon},
    perimeters::FuzzySkinConfig,
};

#[derive(Clone, Copy)]
pub(super) struct FuzzySkinInput {
    pub(super) config: FuzzySkinConfig,
    pub(super) layer_id: usize,
    pub(super) slice_z: f64,
    pub(super) loop_index: usize,
    pub(super) is_contour: bool,
    pub(super) scale: CoordinateScale,
}

pub(super) fn apply(polygon: &Polygon, input: FuzzySkinInput) -> Result<Polygon, SliceError> {
    if !input
        .config
        .should_fuzzify(input.layer_id, input.loop_index, input.is_contour)
    {
        return Ok(polygon.clone());
    }

    Ok(Polygon::new(input.config.fuzzified_scaled_points(
        polygon.points().to_vec(),
        input.layer_id,
        input.slice_z,
        input.scale,
    )))
}
