//! MultiMaterialSegmentation.cpp1144–1168: cut sides against the whole prepared layer.

use super::preprocess::Layer;
use crate::geometry::{ClipperError, ExPolygon, JoinType, difference_ex, offset_expolygons};

pub(super) fn cut_segmented_layers(
    layers: &[Layer],
    segmented: &mut [Vec<Vec<ExPolygon>>],
    width: f32,
    depth: f32,
) -> Result<(), ClipperError> {
    for (index, colors) in segmented.iter_mut().enumerate() {
        let radius = if index % 2 == 0 && depth != 0.0 {
            depth
        } else {
            width
        };
        if radius > 0.0 {
            for color in colors.iter_mut().filter(|color| !color.is_empty()) {
                *color = difference_ex(
                    color,
                    &offset_expolygons(&layers[index].input, -radius, JoinType::Miter, 3.0)?,
                )?;
            }
        }
    }
    Ok(())
}
