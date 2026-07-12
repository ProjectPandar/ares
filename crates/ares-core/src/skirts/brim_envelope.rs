use crate::{LayerBrims, LayerContours, SliceError};

use super::{
    DraftShield, LayerSkirtContext, LayerSkirts, SkirtOptions, SkirtType, generate_layer_skirts,
    min_length, per_object,
};

pub(super) fn generate_skirts_after_brims(
    contours: &[LayerContours],
    brims: &[LayerBrims],
    options: SkirtOptions,
    effective_line_width: f64,
    skirt_extrusion_per_mm: f64,
) -> Result<Vec<LayerSkirts>, SliceError> {
    if !skirt_extrusion_per_mm.is_finite() || skirt_extrusion_per_mm <= 0.0 {
        return Err(SliceError::InvalidInput(
            "skirt extrusion per mm must be positive".to_owned(),
        ));
    }

    let brim_bounds = first_layer_brim_bounds(brims);
    let combined_bounds = combined_skirt_bounds(contours, brim_bounds, options);
    let mut applied_min_length = false;
    contours
        .iter()
        .map(|layer| {
            let paths = if options.generates_on_layer(layer) {
                let apply_min_length = !applied_min_length;
                let paths = if options.skirt_type() == SkirtType::PerObject {
                    per_object::generate_layer_skirts(
                        layer,
                        options,
                        effective_line_width,
                        skirt_extrusion_per_mm,
                        apply_min_length,
                    )?
                } else {
                    generate_layer_skirts(
                        layer,
                        options,
                        effective_line_width,
                        skirt_extrusion_per_mm,
                        LayerSkirtContext {
                            apply_min_length,
                            brim_bounds: combined_bounds,
                        },
                    )?
                };
                if !paths.is_empty() {
                    applied_min_length = true;
                }
                paths
            } else {
                Vec::new()
            };
            Ok(LayerSkirts::new(layer.layer_id(), layer.print_z(), paths))
        })
        .collect()
}

fn combined_skirt_bounds(
    contours: &[LayerContours],
    brim_bounds: Option<min_length::Bounds>,
    options: SkirtOptions,
) -> Option<min_length::Bounds> {
    if !matches!(options.skirt_type(), SkirtType::Combined) {
        return None;
    }
    contours
        .iter()
        .filter(|layer| options.generates_on_layer(layer))
        .filter_map(super::contour_bounds)
        .reduce(|bounds, layer_bounds| min_length::Bounds {
            min_x: bounds.min_x.min(layer_bounds.min_x),
            min_y: bounds.min_y.min(layer_bounds.min_y),
            max_x: bounds.max_x.max(layer_bounds.max_x),
            max_y: bounds.max_y.max(layer_bounds.max_y),
        })
        .map(|bounds| merge_bounds(bounds, brim_bounds, options))
}

pub(super) fn merge_bounds(
    layer_bounds: min_length::Bounds,
    brim_bounds: Option<min_length::Bounds>,
    options: SkirtOptions,
) -> min_length::Bounds {
    if let (true, Some(brim_bounds)) = (uses_brim_envelope(options), brim_bounds) {
        return min_length::Bounds {
            min_x: layer_bounds.min_x.min(brim_bounds.min_x),
            min_y: layer_bounds.min_y.min(brim_bounds.min_y),
            max_x: layer_bounds.max_x.max(brim_bounds.max_x),
            max_y: layer_bounds.max_y.max(brim_bounds.max_y),
        };
    }
    layer_bounds
}

fn first_layer_brim_bounds(brims: &[LayerBrims]) -> Option<min_length::Bounds> {
    brims
        .iter()
        .find(|layer| layer.layer_id() == 0)
        .and_then(|layer| {
            layer
                .paths()
                .iter()
                .flat_map(|path| path.points())
                .fold(None, |bounds, point| {
                    Some(match bounds {
                        Some(bounds) => merge_point(bounds, point.x(), point.y()),
                        None => min_length::Bounds {
                            min_x: point.x(),
                            min_y: point.y(),
                            max_x: point.x(),
                            max_y: point.y(),
                        },
                    })
                })
        })
}

const fn uses_brim_envelope(options: SkirtOptions) -> bool {
    matches!(options.skirt_type(), SkirtType::Combined)
        && matches!(options.draft_shield(), DraftShield::Disabled)
}

fn merge_point(mut bounds: min_length::Bounds, x: f64, y: f64) -> min_length::Bounds {
    bounds.min_x = bounds.min_x.min(x);
    bounds.min_y = bounds.min_y.min(y);
    bounds.max_x = bounds.max_x.max(x);
    bounds.max_y = bounds.max_y.max(y);
    bounds
}
