//! FillLightning.cpp:17-25: clipped tree lines and contour connections.
use crate::fill::lightning::generator::Generator;
use crate::project_slice::group_fills::SurfaceFill;
use crate::{
    SliceError,
    geometry::{CoordinateScale, Polyline},
};

pub(super) fn append(
    fill: &SurfaceFill,
    generator: &Generator,
    layer_index: usize,
    scale: CoordinateScale,
    result: &mut Vec<Polyline>,
) -> Result<(), SliceError> {
    let line_overlap = scale
        .checked_scale(0.5 * fill.params.spacing - fill.params.overlap)
        .unwrap_or(0);
    for surface in &fill.expolygons {
        let domains = crate::geometry::offset_expolygon(
            surface,
            ((fill.params.overlap - 0.5 * fill.params.spacing) / scale.factor()) as f32,
            crate::geometry::JoinType::Miter,
            3.0,
        )
        .map_err(super::super::super::transaction::geometry_error)?;
        for expolygon in &domains {
            let mut outline = vec![expolygon.contour().clone()];
            outline.extend(expolygon.holes().iter().cloned());
            let lines = generator
                .convert_to_lines(layer_index, &outline, line_overlap)
                .map_err(super::super::super::transaction::geometry_error)?;
            // `multiline_fill` (FillLightning.cpp:20) — same expansion as
            // the emit path (fill_entities/lightning.rs).
            let lines = crate::fill::multiline_offset::apply(
                lines,
                fill.params.multiline,
                fill.params.spacing,
                scale,
            )
            .map_err(super::super::super::transaction::geometry_error)?;
            // `intersection_pl` after `multiline_fill`
            // (FillLightning.cpp:20-23) — the offsets can leave the
            // contour or enter holes; re-clip to the fill outline.
            let lines = crate::geometry::intersection_open_polylines(&lines, &outline)
                .map_err(super::super::super::transaction::geometry_error)?;
            if lines.is_empty() {
                continue;
            }
            result.extend(
                crate::fill::connect::connect_infill(
                    lines,
                    expolygon,
                    fill.params.spacing,
                    crate::fill::connect::FillConnectionParams {
                        anchor_length: fill.params.anchor_length,
                        anchor_length_max: fill.params.anchor_length_max,
                        multiline: fill.params.multiline,
                        dont_sort: false,
                    },
                    scale,
                )
                .map_err(super::super::super::transaction::geometry_error)?,
            );
        }
    }
    Ok(())
}
