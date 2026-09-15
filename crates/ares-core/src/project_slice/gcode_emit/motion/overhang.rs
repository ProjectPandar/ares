#[cfg(test)]
mod tests;

mod curl;
mod sections;

pub(in crate::project_slice::gcode_emit) use curl::CurlTracker;
use sections::{scale_trunc, speed_for_distance, speed_sections};

use super::{LayerGeometry, MotionOptions, features::PathProperties};
use crate::{FloatOrPercent, geometry::Point};

pub(super) const OVERLAPS: [f64; 6] = [90.0, 75.0, 50.0, 25.0, 13.0, 0.0];
pub(super) const INTERSECTION_EPSILON_MM: f64 = 1e-4;

pub(super) struct EstimateRequest<'a> {
    pub(super) points: &'a [(f64, f64)],
    pub(super) properties: PathProperties<'a>,
    pub(super) geometry: LayerGeometry<'a>,
    pub(super) options: &'a MotionOptions,
    pub(super) layer_index: usize,
    pub(super) original_speed: f64,
    pub(super) curl: &'a mut CurlTracker,
}

#[derive(Clone, Copy)]
pub(super) struct ProcessedPoint {
    pub(super) x: f64,
    pub(super) y: f64,
    pub(super) speed: f64,
    pub(super) overlap: f64,
}

#[derive(Clone, Copy)]
pub(super) struct ExtendedPoint {
    x: f64,
    y: f64,
    distance: f32,
}

pub(super) struct BoundaryContext<'a> {
    pub(super) tree: &'a crate::geometry::LineDistanceTree<'a>,
    pub(super) scale: crate::geometry::CoordinateScale,
    pub(super) offset: f32,
    pub(super) minimum_spacing: f64,
}

pub(super) fn estimate(request: EstimateRequest<'_>) -> Option<Vec<ProcessedPoint>> {
    // `estimate_malformations` (`PrintObject.cpp:900-914`) runs over every
    // layer's external perimeters while overhang speed is enabled —
    // including the first layer, whose curl state feeds the next layer's
    // artificial distance (`SupportSpotsGenerator.cpp:141-196`).
    if request.options.enable_overhang_speed && request.properties.feature == "Outer wall" {
        request.curl.collect(
            request.layer_index,
            request.points,
            request.properties.width,
            request.properties.height,
            request.geometry.previous_layer_boundary,
            request.geometry.scale,
        );
    }
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
    let reference_speed = if request.options.max_volumetric_speed > 0.0 {
        role_speed.min(
            request.options.max_volumetric_speed
                / (request.properties.mm3_per_mm
                    * request.options.print_flow_ratio
                    * request.options.filament_flow_ratio),
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
    let extended = context.add_boundary_intersections(request.points);
    let extended = context.add_segmentation_points(&extended, minimum_slowdown_distance);
    // The artificial curl distance (`ExtrusionProcessor.hpp:415-418`):
    // min() the curl speed into the pair speed like the boundary bands.
    let curled_lines = if request.options.slowdown_for_curled_perimeters {
        let curled = request.curl.previous_curled(request.layer_index);
        (!curled.is_empty()).then_some(curled)
    } else {
        None
    };
    if let Ok(level) = std::env::var("ARES_DUMP_OVERHANG") {
        eprintln!(
            "OH layer={} feature={} width={:.4} ref={:.4} orig={:.4} minsd={:.4} pts={} curled={} variable",
            request.layer_index,
            request.properties.feature,
            request.properties.width,
            reference_speed,
            request.original_speed,
            minimum_slowdown_distance,
            extended.len(),
            curled_lines.map_or(0, <[curl::CurledLine]>::len),
        );
        if level == "2" {
            for (index, point) in extended.iter().enumerate() {
                let artificial = curled_lines.map_or(0.0, |curled| {
                    let next = extended.get(index + 1).copied().unwrap_or(*point);
                    curl::artificial_distance(
                        curled,
                        (point.x, point.y),
                        (next.x, next.y),
                        request.properties.width,
                        request.properties.height,
                    )
                });
                eprintln!(
                    "OHL {index:03} x={:.4} y={:.4} dist={:.6} artificial={:.6} speed={:.3}",
                    point.x,
                    point.y,
                    point.distance,
                    artificial,
                    speed_for_distance(point.distance, &sections, original_speed)
                );
            }
        }
    }

    let mut processed = Vec::with_capacity(extended.len());
    let mut variable = false;
    for index in 0..extended.len() {
        let current = extended[index];
        let next = extended.get(index + 1).copied().unwrap_or(current);
        let mut speed = speed_for_distance(current.distance, &sections, original_speed)
            .min(speed_for_distance(next.distance, &sections, original_speed))
            .min(original_speed);
        if let Some(curled) = curled_lines {
            let artificial = curl::artificial_distance(
                curled,
                (current.x, current.y),
                (next.x, next.y),
                request.properties.width,
                request.properties.height,
            );
            speed = speed.min(speed_for_distance(artificial, &sections, original_speed));
        }
        variable |= (f64::from(speed) - request.original_speed).abs() > 1.0;
        let width_inverse = 1.0_f32 / request.properties.width;
        processed.push(ProcessedPoint {
            x: context.scale.unscale(scale_trunc(current.x, context.scale)),
            y: context.scale.unscale(scale_trunc(current.y, context.scale)),
            speed: f64::from(speed),
            overlap: f64::from(
                (1.0_f32 - current.distance * width_inverse)
                    .min(1.0_f32 - next.distance * width_inverse),
            ),
        });
    }
    variable.then_some(processed)
}
