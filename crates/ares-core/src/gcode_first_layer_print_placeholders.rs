use crate::gcode_format::format_decimal;
use crate::{LayerPrintPaths, Point2};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct FirstLayerPrintPlaceholders {
    min: String,
    max: String,
    size: String,
    center: String,
    bounds: Option<FirstLayerPrintBounds>,
}

impl FirstLayerPrintPlaceholders {
    pub(crate) fn min_list(&self) -> &str {
        &self.min
    }

    pub(crate) fn max_list(&self) -> &str {
        &self.max
    }

    pub(crate) fn size_list(&self) -> &str {
        &self.size
    }

    pub(crate) fn center_list(&self) -> &str {
        &self.center
    }

    pub(crate) const fn bounds(&self) -> Option<FirstLayerPrintBounds> {
        self.bounds
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct FirstLayerPrintBounds {
    min: Point2,
    max: Point2,
}

impl FirstLayerPrintBounds {
    pub(crate) const fn new(min: Point2, max: Point2) -> Self {
        Self { min, max }
    }

    pub(crate) const fn min(&self) -> Point2 {
        self.min
    }

    pub(crate) const fn max(&self) -> Point2 {
        self.max
    }

    pub(crate) fn include(&mut self, point: Point2) {
        self.min = Point2::new(self.min.x().min(point.x()), self.min.y().min(point.y()));
        self.max = Point2::new(self.max.x().max(point.x()), self.max.y().max(point.y()));
    }
}

pub(crate) fn placeholders(layer_print_paths: &[LayerPrintPaths]) -> FirstLayerPrintPlaceholders {
    let Some(bounds) = first_layer_bounds(layer_print_paths) else {
        return empty_placeholders();
    };
    let center = Point2::new(
        (bounds.min.x() + bounds.max.x()) / 2.0,
        (bounds.min.y() + bounds.max.y()) / 2.0,
    );

    FirstLayerPrintPlaceholders {
        min: format!(
            "{},{}",
            format_decimal(bounds.min.x()),
            format_decimal(bounds.min.y())
        ),
        max: format!(
            "{},{}",
            format_decimal(bounds.max.x()),
            format_decimal(bounds.max.y())
        ),
        size: format!(
            "{},{}",
            format_decimal(bounds.max.x() - bounds.min.x()),
            format_decimal(bounds.max.y() - bounds.min.y())
        ),
        center: format!(
            "{},{}",
            format_decimal(center.x()),
            format_decimal(center.y())
        ),
        bounds: Some(bounds),
    }
}

fn empty_placeholders() -> FirstLayerPrintPlaceholders {
    FirstLayerPrintPlaceholders {
        min: String::new(),
        max: String::new(),
        size: String::new(),
        center: String::new(),
        bounds: None,
    }
}

fn first_layer_bounds(layer_print_paths: &[LayerPrintPaths]) -> Option<FirstLayerPrintBounds> {
    layer_print_paths
        .first()
        .and_then(|layer| points_bounds(layer.paths().iter().flat_map(|path| path.points())))
}

fn points_bounds<'a>(
    points: impl IntoIterator<Item = &'a Point2>,
) -> Option<FirstLayerPrintBounds> {
    let mut points = points.into_iter();
    let first = *points.next()?;
    let mut bounds = FirstLayerPrintBounds::new(first, first);
    for point in points {
        bounds.include(*point);
    }
    Some(bounds)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PrintPath, PrintPathRole};

    #[test]
    fn empty_first_layer_paths_render_empty_placeholder_strings() {
        let rendered = placeholders(&[]);

        assert_eq!(rendered.min_list(), "");
        assert_eq!(rendered.max_list(), "");
        assert_eq!(rendered.size_list(), "");
        assert_eq!(rendered.center_list(), "");
        assert_eq!(rendered.bounds(), None);

        let empty_layer = LayerPrintPaths::new(0, 0.2, Vec::new());
        let rendered = placeholders(&[empty_layer]);

        assert_eq!(rendered.min_list(), "");
        assert_eq!(rendered.max_list(), "");
        assert_eq!(rendered.size_list(), "");
        assert_eq!(rendered.center_list(), "");
        assert_eq!(rendered.bounds(), None);
    }

    #[test]
    fn asymmetric_first_layer_paths_render_center_from_bounds() {
        let layer = LayerPrintPaths::new(
            0,
            0.2,
            vec![
                PrintPath::new(
                    PrintPathRole::ExternalPerimeter,
                    vec![
                        Point2::new(1.0, -3.0),
                        Point2::new(7.0, 9.0),
                        Point2::new(3.0, 4.0),
                    ],
                )
                .unwrap(),
            ],
        );

        let rendered = placeholders(&[layer]);

        assert_eq!(rendered.min_list(), "1,-3");
        assert_eq!(rendered.max_list(), "7,9");
        assert_eq!(rendered.size_list(), "6,12");
        assert_eq!(rendered.center_list(), "4,3");
        assert_eq!(
            rendered.bounds(),
            Some(FirstLayerPrintBounds::new(
                Point2::new(1.0, -3.0),
                Point2::new(7.0, 9.0),
            ))
        );
    }
}
