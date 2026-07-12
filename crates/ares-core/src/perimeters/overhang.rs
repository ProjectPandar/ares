use crate::{LayerContours, PerimeterOptions, PerimeterRole, Point2};

pub(super) type RectBounds = (f64, f64, f64, f64);

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct ExternalRole {
    role: PerimeterRole,
    unsupported_span_mm: Option<f64>,
}

impl ExternalRole {
    pub(super) const fn new(role: PerimeterRole, unsupported_span_mm: Option<f64>) -> Self {
        Self {
            role,
            unsupported_span_mm,
        }
    }

    pub(super) const fn role(self) -> PerimeterRole {
        self.role
    }

    pub(super) const fn unsupported_span_mm(self) -> Option<f64> {
        self.unsupported_span_mm
    }
}

pub(super) fn external_role(
    points: &[Point2],
    previous_layer: Option<&LayerContours>,
    options: PerimeterOptions,
) -> ExternalRole {
    if !options.detect_overhang_wall() {
        return ExternalRole::new(PerimeterRole::External, None);
    }
    let Some(current) = super::rectangles::bounds(points) else {
        return ExternalRole::new(PerimeterRole::External, None);
    };
    let Some(previous_layer) = previous_layer else {
        return ExternalRole::new(PerimeterRole::External, None);
    };
    if previous_layer.contours().iter().any(|contour| {
        contour_bounds(contour.points())
            .is_some_and(|previous| has_positive_area_overlap(current, previous))
    }) {
        ExternalRole::new(PerimeterRole::External, None)
    } else {
        ExternalRole::new(PerimeterRole::Overhang, Some(unsupported_span(current)))
    }
}

fn contour_bounds(points: &[Point2]) -> Option<RectBounds> {
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

fn has_positive_area_overlap(current: RectBounds, previous: RectBounds) -> bool {
    let overlap_min_x = current.0.max(previous.0);
    let overlap_min_y = current.1.max(previous.1);
    let overlap_max_x = current.2.min(previous.2);
    let overlap_max_y = current.3.min(previous.3);
    overlap_max_x > overlap_min_x && overlap_max_y > overlap_min_y
}

fn unsupported_span(bounds: RectBounds) -> f64 {
    (bounds.2 - bounds.0).max(bounds.3 - bounds.1)
}
