mod curl;
mod interior;
mod printing_state;
mod sampling;
mod sections;

pub(in crate::project_slice) use curl::{
    CurlEstimate, CurledLine, ExternalEntityPoints, estimate_object,
};
use printing_state::PrintingState;
use sections::{scale_trunc, speed_for_distance, speed_sections};

use super::{LayerGeometry, MotionOptions, fan, features::PathProperties};

pub(super) const OVERLAPS: [f64; 6] = [90.0, 75.0, 50.0, 25.0, 13.0, 0.0];
pub(super) const INTERSECTION_EPSILON_MM: f64 = 1e-4;

pub(super) struct EstimateRequest<'a> {
    pub(super) points: &'a [(f64, f64)],
    pub(super) properties: PathProperties<'a>,
    pub(super) geometry: LayerGeometry<'a>,
    pub(super) options: &'a MotionOptions,
    pub(super) on_first_layer: bool,
    pub(super) object_layer_over_raft: bool,
    pub(super) original_speed: f64,
}

pub(super) struct Estimate {
    pub(super) points: Vec<ProcessedPoint>,
    pub(super) segmented: bool,
    pub(super) fan_active: bool,
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

pub(super) fn estimate(request: EstimateRequest<'_>) -> Option<Estimate> {
    let boundary = request.geometry.previous_layer_boundary?;
    if !(request.options.enable_overhang_speed || request.options.enable_overhang_bridge_fan)
        || request.on_first_layer
        || request.object_layer_over_raft
        || !matches!(
            request.properties.feature,
            "Inner wall" | "Outer wall" | "Overhang wall" | "Bridge" | "Internal Bridge"
        )
        || request.points.len() < 2
    {
        return None;
    }
    let sections = speed_sections(
        request.properties.width,
        reference_speed(&request),
        request.options,
        request.properties.feature == "Bridge",
    );
    let original_speed = request.original_speed as f32;
    let minimum_slowdown_distance = sections
        .iter()
        .filter(|section| section.1 <= original_speed)
        .map(|section| section.0)
        .reduce(f32::min)
        .unwrap_or(-1.0);
    let fan_threshold = if request.options.enable_overhang_bridge_fan
        && !matches!(request.properties.feature, "Bridge" | "Overhang wall")
    {
        fan::overlap_threshold(request.options.overhang_fan_threshold)
    } else {
        -1.0
    };
    let context = BoundaryContext {
        tree: boundary,
        scale: request.geometry.scale,
        offset: 0.5_f32 * request.properties.width,
        minimum_spacing: f64::from(request.properties.width) * 0.25,
    };
    let printing = PrintingState {
        sections: &sections,
        original_speed,
        width_inverse: 1.0_f32 / request.properties.width,
        fan_threshold,
    };
    let extended = context.add_boundary_intersections(request.points, request.properties.contour);
    let extended = context.add_interior_points(extended, minimum_slowdown_distance, &printing);
    let extended = context.add_segmentation_points(&extended, &printing);
    let points = process_points(&request, &context, &extended, &sections);
    let speed_changes = points
        .iter()
        .any(|point| (point.speed - request.original_speed).abs() > 1.0);
    let threshold = f64::from(fan_threshold);
    let fan_active = fan_threshold >= 0.0 && points[0].overlap <= threshold;
    let fan_changes = fan_threshold >= 0.0
        && points
            .windows(2)
            .any(|pair| (pair[0].overlap <= threshold) != (pair[1].overlap <= threshold));
    Some(Estimate {
        points,
        segmented: speed_changes || fan_changes,
        fan_active,
    })
}

fn reference_speed(request: &EstimateRequest<'_>) -> f64 {
    // GCode.cpp:9559–9570: role speed, automatic speed, actual-flow cap, scarf cap.
    let role_speed = if matches!(request.properties.feature, "Outer wall" | "Overhang wall") {
        request.options.outer_wall_speed
    } else {
        request.options.inner_wall_speed
    };
    let volumetric_speed = request.options.max_volumetric_speed
        / super::extrusion::effective_flow(request.properties, request.options, false);
    let reference_speed = if role_speed == 0.0 {
        volumetric_speed
    } else {
        role_speed
    };
    let reference_speed = if request.options.max_volumetric_speed > 0.0 {
        reference_speed.min(volumetric_speed)
    } else {
        reference_speed
    };
    request
        .properties
        .slope
        .map_or(reference_speed, |slope| reference_speed.min(slope.speed))
}

fn process_points(
    request: &EstimateRequest<'_>,
    context: &BoundaryContext<'_>,
    extended: &[ExtendedPoint],
    sections: &[(f32, f32)],
) -> Vec<ProcessedPoint> {
    let original_speed = request.original_speed as f32;
    let curled_lines = request
        .options
        .slowdown_for_curled_perimeters
        .then_some(request.geometry.previous_curled)
        .filter(|lines| !lines.is_empty());
    let mut processed = Vec::with_capacity(extended.len());
    for index in 0..extended.len() {
        let current = extended[index];
        let next = extended.get(index + 1).copied().unwrap_or(current);
        let mut speed = speed_for_distance(current.distance, sections, original_speed)
            .min(speed_for_distance(next.distance, sections, original_speed))
            .min(original_speed);
        // Existing source curl distance contributes to speed AND overlap.
        let mut artificial = 0.0_f32;
        if let Some(curled) = curled_lines {
            artificial = curl::artificial_distance(
                curled,
                [(current.x, current.y), (next.x, next.y)],
                request.properties.width,
                request.properties.height,
                request.geometry.scale,
            );
            speed = speed.min(speed_for_distance(artificial, sections, original_speed));
        }
        if !request.options.enable_overhang_speed {
            speed = original_speed;
        }
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
    processed
}
