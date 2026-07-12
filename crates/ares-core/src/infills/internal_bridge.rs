use crate::{
    InfillOptions, Point2,
    options::{InfillLayerRole, InternalBridgeFilter},
};

type Bounds = (f64, f64, f64, f64);

#[derive(Clone, Copy, Debug)]
pub(super) struct BridgeInfillOverride {
    pub(super) fixed_angle_degrees: Option<f64>,
    pub(super) density_percent: f64,
}

pub(super) fn uses_density(
    role: InfillLayerRole,
    options: &InfillOptions,
    contours: &[&[Point2]],
) -> bool {
    role == InfillLayerRole::InternalSolid
        && options.has_shell_layers()
        && options.internal_bridge_density_percent() < 100.0
        && options
            .internal_bridge_filter()
            .allows_internal_bridge(contours, options.solid_line_width())
}

pub(super) fn angle_override(
    role: InfillLayerRole,
    options: &InfillOptions,
    contours: &[&[Point2]],
) -> Option<f64> {
    if !uses_density(role, options, contours) {
        return None;
    }
    if options.internal_bridge_angle_degrees() > 0.0 {
        Some(options.internal_bridge_angle_degrees())
    } else {
        auto_angle_degrees(contours)
    }
}

pub(super) fn bridge_infill_override(
    role: InfillLayerRole,
    layer_index: usize,
    options: &InfillOptions,
    bridge_context: Option<super::InfillBridgeContext<'_>>,
) -> Option<BridgeInfillOverride> {
    if role != InfillLayerRole::BottomSurface
        || !bridge_context.is_some_and(|context| {
            context.overrides_bottom_surface(layer_index)
                || context.overrides_extra_external_bridge_layer(layer_index)
        })
    {
        return None;
    }
    Some(BridgeInfillOverride {
        fixed_angle_degrees: (options.bridge_angle_degrees() > 0.0)
            .then(|| options.bridge_angle_degrees()),
        density_percent: options.bridge_density_percent(),
    })
}

pub(super) fn fixed_angle_degrees(
    bridge_override: Option<BridgeInfillOverride>,
    contours: &[&[Point2]],
) -> Option<f64> {
    match bridge_override {
        Some(BridgeInfillOverride {
            fixed_angle_degrees: Some(angle),
            ..
        }) => Some(angle),
        Some(BridgeInfillOverride {
            fixed_angle_degrees: None,
            ..
        }) => auto_angle_degrees(contours),
        None => None,
    }
}

impl InternalBridgeFilter {
    const DISABLED_SPAN_FILTER_LINE_WIDTH_MULTIPLIER: f64 = 6.0;
    const LIMITED_SPAN_FILTER_LINE_WIDTH_MULTIPLIER: f64 = 2.0;

    pub(crate) fn allows_internal_bridge(
        self,
        contours: &[&[Point2]],
        solid_line_width: f64,
    ) -> bool {
        match self {
            Self::Disabled => {
                largest_span(contours)
                    >= solid_line_width * Self::DISABLED_SPAN_FILTER_LINE_WIDTH_MULTIPLIER
            }
            Self::Limited => {
                largest_span(contours)
                    >= solid_line_width * Self::LIMITED_SPAN_FILTER_LINE_WIDTH_MULTIPLIER
            }
            Self::NoFilter => true,
        }
    }
}

fn largest_span(contours: &[&[Point2]]) -> f64 {
    contours
        .iter()
        .filter_map(|points| contour_span(points))
        .fold(0.0, f64::max)
}

fn contour_span(points: &[Point2]) -> Option<f64> {
    let (min_x, min_y, max_x, max_y) = contour_bounds(points)?;
    Some((max_x - min_x).max(max_y - min_y))
}

fn contour_bounds(points: &[Point2]) -> Option<Bounds> {
    let first = points.first()?;
    let mut min_x = first.x();
    let mut max_x = first.x();
    let mut min_y = first.y();
    let mut max_y = first.y();
    for point in &points[1..] {
        min_x = min_x.min(point.x());
        max_x = max_x.max(point.x());
        min_y = min_y.min(point.y());
        max_y = max_y.max(point.y());
    }
    Some((min_x, min_y, max_x, max_y))
}

fn auto_angle_degrees(contours: &[&[Point2]]) -> Option<f64> {
    let (min_x, min_y, max_x, max_y) = combined_bounds(contours)?;
    let width = max_x - min_x;
    let height = max_y - min_y;
    if width <= 0.0 || height <= 0.0 || width == height {
        return None;
    }
    Some(if width > height { 90.0 } else { 0.0 })
}

fn combined_bounds(contours: &[&[Point2]]) -> Option<Bounds> {
    let mut bounds = contours.iter().filter_map(|points| contour_bounds(points));
    let first = bounds.next()?;
    Some(bounds.fold(first, |acc, bounds| {
        (
            acc.0.min(bounds.0),
            acc.1.min(bounds.1),
            acc.2.max(bounds.2),
            acc.3.max(bounds.3),
        )
    }))
}
