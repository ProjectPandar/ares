#[cfg(test)]
mod tests;

mod curl;
mod sections;

pub(in crate::project_slice) use curl::{
    CurlEstimate, CurledLine, ExternalEntityPoints, estimate_object,
};
use sections::{scale_trunc, speed_for_distance, speed_sections};

use super::{LayerGeometry, MotionOptions, features::PathProperties};

pub(super) const OVERLAPS: [f64; 6] = [90.0, 75.0, 50.0, 25.0, 13.0, 0.0];
pub(super) const INTERSECTION_EPSILON_MM: f64 = 1e-4;

pub(super) struct EstimateRequest<'a> {
    pub(super) points: &'a [(f64, f64)],
    pub(super) properties: PathProperties<'a>,
    pub(super) geometry: LayerGeometry<'a>,
    pub(super) options: &'a MotionOptions,
    pub(super) layer_index: usize,
    pub(super) original_speed: f64,
}

#[derive(Clone, Copy)]
pub(super) struct ProcessedPoint {
    pub(super) x: f64,
    pub(super) y: f64,
    pub(super) z: f64,
    pub(super) speed: f64,
    pub(super) overlap: f64,
}

#[derive(Clone, Copy)]
pub(super) struct ExtendedPoint {
    x: f64,
    y: f64,
    z: f64,
    distance: f32,
}

pub(super) struct BoundaryContext<'a> {
    pub(super) tree: &'a crate::geometry::LineDistanceTree<'a>,
    pub(super) scale: crate::geometry::CoordinateScale,
    pub(super) offset: f32,
    pub(super) minimum_spacing: f64,
}

pub(super) fn estimate(request: EstimateRequest<'_>) -> Option<Vec<ProcessedPoint>> {
    let boundary = request.geometry.previous_layer_boundary?;
    if !request.options.enable_overhang_speed
        || request.layer_index == 0
        || !matches!(
            request.properties.feature,
            "Inner wall" | "Outer wall" | "Overhang wall" | "Bridge" | "Internal Bridge"
        )
        || request.points.len() < 2
    {
        return None;
    }

    // The reference speed caps against the same effective `_mm3_per_mm` as
    // the extrusion speed cap (`GCode.cpp:6658-6665`), and only while
    // `filament_max_volumetric_speed > 0`.
    let role_speed = if matches!(request.properties.feature, "Outer wall" | "Overhang wall") {
        request.options.outer_wall_speed
    } else {
        request.options.inner_wall_speed
    };
    // First-layer paths returned above; share the complete effective flow.
    let reference_speed = if request.options.max_volumetric_speed > 0.0 {
        role_speed.min(
            request.options.max_volumetric_speed
                / super::extrusion::effective_flow(request.properties, request.options, false),
        )
    } else {
        role_speed
    };
    let reference_speed = request
        .properties
        .slope
        .map_or(reference_speed, |slope| reference_speed.min(slope.speed));
    let sections = speed_sections(request.properties.width, reference_speed, request.options);
    let original_speed = request.original_speed as f32;
    let minimum_slowdown_distance = sections
        .iter()
        .filter(|section| section.1 <= original_speed)
        .map(|section| section.0)
        .reduce(f32::min)
        .unwrap_or(-1.0);
    let context = BoundaryContext {
        tree: boundary,
        scale: request.geometry.scale,
        offset: 0.5_f32 * request.properties.width,
        minimum_spacing: f64::from(request.properties.width) * 0.25,
    };
    let extended = context.add_boundary_intersections(request.points, request.properties.contour);
    let extended = context.add_segmentation_points(&extended, minimum_slowdown_distance);
    // The artificial curl distance (`ExtrusionProcessor.hpp:415-418`):
    // min() the curl speed into the pair speed like the boundary bands.
    // The previous layer's curled lines are the pre-estimated
    // `Layer::curled_lines` (`PrintObject::estimate_curled_extrusions`
    // consumed by `prepare_for_new_layer`).
    let curled_lines = request
        .options
        .slowdown_for_curled_perimeters
        .then_some(request.geometry.previous_curled)
        .filter(|lines| !lines.is_empty());

    let mut processed = Vec::with_capacity(extended.len());
    let mut variable = false;
    for index in 0..extended.len() {
        let current = extended[index];
        let next = extended.get(index + 1).copied().unwrap_or(current);
        let mut speed = speed_for_distance(current.distance, &sections, original_speed)
            .min(speed_for_distance(next.distance, &sections, original_speed))
            .min(original_speed);
        // `artificial_distance_to_curled_lines` starts at 0 per segment
        // (`ExtrusionProcessor.hpp:379`) and is added to BOTH endpoint
        // distances in the overlap (`hpp:455`) — it drives the overhang
        // bridge fan toggles through `check_overhang_fan`, not just the
        // curled speed floor.
        let mut artificial = 0.0_f32;
        if let Some(curled) = curled_lines {
            artificial = curl::artificial_distance(
                curled,
                [(current.x, current.y), (next.x, next.y)],
                request.properties.width,
                request.properties.height,
                request.geometry.scale,
            );
            speed = speed.min(speed_for_distance(artificial, &sections, original_speed));
        }
        variable |= (f64::from(speed) - request.original_speed).abs() > 1.0;
        let width_inverse = 1.0_f32 / request.properties.width;
        processed.push(ProcessedPoint {
            x: context.scale.unscale(scale_trunc(current.x, context.scale)),
            y: context.scale.unscale(scale_trunc(current.y, context.scale)),
            z: context.scale.unscale(scale_trunc(current.z, context.scale)),
            speed: f64::from(speed),
            overlap: f64::from(
                (1.0_f32 - (current.distance + artificial) * width_inverse)
                    .min(1.0_f32 - (next.distance + artificial) * width_inverse),
            ),
        });
    }
    variable.then_some(processed)
}
