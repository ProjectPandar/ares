use super::{GapFillPath, LayerGapFills, SolidSurfaceGapFillInput};
use crate::{
    Point2, SliceError,
    bridge_support::fully_unsupported_layer,
    options::{GapFillTarget, InfillLayerRole},
};

pub(super) fn append(
    gap_fills: &mut [LayerGapFills],
    input: SolidSurfaceGapFillInput<'_>,
) -> Result<(), SliceError> {
    if input.target == GapFillTarget::Nowhere {
        return Ok(());
    }
    validate_layers(gap_fills, &input)?;
    for (layer_index, gap_fill) in gap_fills.iter_mut().enumerate() {
        if skip_bridge_layer(&input, layer_index) {
            continue;
        }
        let role = input
            .infill_options
            .layer_role_for_layers(input.print_layers, layer_index);
        if !target_allows_role(input.target, role) {
            continue;
        }
        let mut paths = Vec::new();
        for contour in input.layer_contours[layer_index].contours() {
            if let Some(path) =
                solid_surface_path(contour.points(), input.infill_options.solid_line_width())?
            {
                paths.push(path);
            }
        }
        gap_fill.append_paths(paths);
    }
    Ok(())
}

fn validate_layers(
    gap_fills: &[LayerGapFills],
    input: &SolidSurfaceGapFillInput<'_>,
) -> Result<(), SliceError> {
    if gap_fills.len() != input.print_layers.len() || gap_fills.len() != input.layer_contours.len()
    {
        return Err(SliceError::InvalidInput(
            "gap-fill, print layer and contour layer counts must match".to_owned(),
        ));
    }
    for ((gap_fill, print_layer), contour_layer) in gap_fills
        .iter()
        .zip(input.print_layers.iter())
        .zip(input.layer_contours.iter())
    {
        if gap_fill.layer_id() != print_layer.id()
            || gap_fill.layer_id() != contour_layer.layer_id()
            || gap_fill.print_z() != print_layer.print_z()
            || gap_fill.print_z() != contour_layer.print_z()
        {
            return Err(SliceError::InvalidInput(
                "gap-fill, print layer and contour layer metadata must match".to_owned(),
            ));
        }
    }
    Ok(())
}

fn target_allows_role(target: GapFillTarget, role: InfillLayerRole) -> bool {
    match role {
        InfillLayerRole::Sparse => false,
        InfillLayerRole::BottomSurface | InfillLayerRole::TopSurface => target.allows_top_bottom(),
        InfillLayerRole::InternalSolid => target.allows_internal_solid(),
    }
}

fn skip_bridge_layer(input: &SolidSurfaceGapFillInput<'_>, layer_index: usize) -> bool {
    if !input.bridge_no_support {
        return false;
    }
    let unsupported_bottom_bridge = !input
        .counterbore_hole_bridging
        .preserves_bridge_surfaces_for_solid_detection()
        && fully_unsupported_layer(input.layer_contours, layer_index);
    unsupported_bottom_bridge
        || (layer_index > 0
            && input.extra_bridge_layer.applies_to_external_bridge()
            && fully_unsupported_layer(input.layer_contours, layer_index - 1)
            && !fully_unsupported_layer(input.layer_contours, layer_index))
}

fn solid_surface_path(
    points: &[Point2],
    solid_width: f64,
) -> Result<Option<GapFillPath>, SliceError> {
    let Some((min_x, min_y, max_x, max_y)) = crate::contours::axis_aligned_rectangle_bounds(points)
    else {
        return Ok(None);
    };
    let width = max_x - min_x;
    let height = max_y - min_y;
    if height > 0.0 && height <= 2.0 * solid_width && width > 2.0 * solid_width {
        let center_y = (min_y + max_y) / 2.0;
        GapFillPath::new(vec![
            Point2::new(min_x + solid_width, center_y),
            Point2::new(max_x - solid_width, center_y),
        ])
        .map(Some)
    } else if width > 0.0 && width <= 2.0 * solid_width && height > 2.0 * solid_width {
        let center_x = (min_x + max_x) / 2.0;
        GapFillPath::new(vec![
            Point2::new(center_x, min_y + solid_width),
            Point2::new(center_x, max_y - solid_width),
        ])
        .map(Some)
    } else {
        Ok(None)
    }
}
