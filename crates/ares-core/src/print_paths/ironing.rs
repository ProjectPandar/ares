use super::{
    LayerPrintPaths, PrintPath, PrintPathRole,
    ironing_scanlines::{RectangleBounds, rectilinear_scanlines},
};
use crate::{
    Point2,
    options::ironing_type::{IroningPattern, IroningType, OrdinaryIroningConfig},
};

const EPSILON: f64 = 1e-9;

pub(crate) fn apply_ironing(
    layers: Vec<LayerPrintPaths>,
    config: OrdinaryIroningConfig,
) -> Vec<LayerPrintPaths> {
    let ironing_type = config.ironing_type();
    if ironing_type == IroningType::NoIroning {
        return layers;
    }

    let last_layer_index = layers.len().saturating_sub(1);
    layers
        .into_iter()
        .enumerate()
        .map(|(layer_index, layer)| {
            let ironing_count =
                ordinary_ironing_count(&layer, ironing_type, layer_index, last_layer_index);
            if ironing_count == 0 {
                return layer;
            }

            let mut paths = Vec::with_capacity(layer.paths().len() + ironing_count);
            paths.extend(layer.paths().iter().cloned());
            paths.extend(
                layer
                    .paths()
                    .iter()
                    .filter(|path| {
                        should_iron(ironing_type, layer_index, last_layer_index, path.role())
                    })
                    .flat_map(|path| duplicate_as_ironing(path, &config, layer_index)),
            );
            LayerPrintPaths::new(layer.layer_id(), layer.print_z(), paths)
        })
        .collect()
}

fn ordinary_ironing_count(
    layer: &LayerPrintPaths,
    ironing_type: IroningType,
    layer_index: usize,
    last_layer_index: usize,
) -> usize {
    layer
        .paths()
        .iter()
        .filter(|path| should_iron(ironing_type, layer_index, last_layer_index, path.role()))
        .count()
}

fn should_iron(
    ironing_type: IroningType,
    layer_index: usize,
    last_layer_index: usize,
    role: PrintPathRole,
) -> bool {
    match ironing_type {
        IroningType::NoIroning => false,
        IroningType::TopSurfaces => role == PrintPathRole::TopSolidInfill,
        IroningType::TopmostOnly => {
            layer_index == last_layer_index && role == PrintPathRole::TopSolidInfill
        }
        IroningType::AllSolid => matches!(
            role,
            PrintPathRole::TopSolidInfill
                | PrintPathRole::SolidInfill
                | PrintPathRole::BottomSurface
        ),
    }
}

fn duplicate_as_ironing(
    path: &PrintPath,
    config: &OrdinaryIroningConfig,
    layer_index: usize,
) -> Vec<PrintPath> {
    let Some(points) = inset_points(path, config.inset_mm()) else {
        return Vec::new();
    };
    ironing_geometries(
        path,
        points,
        config.spacing_mm(),
        config.pattern(),
        config.rectilinear_angle_degrees(layer_index),
    )
    .into_iter()
    .map(|(points, closed)| duplicate_path(path, points, closed))
    .collect()
}

fn duplicate_path(path: &PrintPath, points: Vec<Point2>, closed: bool) -> PrintPath {
    let mut duplicate = PrintPath::new(PrintPathRole::Ironing, points)
        .expect("existing print path points are non-empty")
        .with_unsupported_span_mm(path.unsupported_span_mm())
        .with_seam_gap_mm(path.seam_gap_mm())
        .with_closed(closed);
    if let Some(height) = path.effective_layer_height_mm() {
        duplicate = duplicate.with_effective_layer_height_mm(height);
    }
    duplicate
}

fn ironing_geometries(
    path: &PrintPath,
    points: Vec<Point2>,
    spacing_mm: f64,
    pattern: IroningPattern,
    angle_degrees: f64,
) -> Vec<(Vec<Point2>, bool)> {
    if path.is_closed()
        && path.points().len() == 4
        && spacing_mm > 0.0
        && let Some(bounds) = rectangle_bounds(&points)
    {
        return match pattern {
            IroningPattern::Rectilinear => rectilinear_scanlines(bounds, spacing_mm, angle_degrees)
                .into_iter()
                .map(|points| (points, false))
                .collect(),
            IroningPattern::Concentric => concentric_ironing_loops(bounds, spacing_mm)
                .into_iter()
                .map(|points| (points, true))
                .collect(),
        };
    }
    vec![(points, path.is_closed())]
}

fn concentric_ironing_loops(bounds: RectangleBounds, spacing_mm: f64) -> Vec<Vec<Point2>> {
    let mut loops = Vec::new();
    let mut inset = 0.0;
    loop {
        let min_x = bounds.min_x + inset;
        let min_y = bounds.min_y + inset;
        let max_x = bounds.max_x - inset;
        let max_y = bounds.max_y - inset;
        if max_x - min_x <= EPSILON || max_y - min_y <= EPSILON {
            break;
        }
        loops.push(vec![
            Point2::new(min_x, min_y),
            Point2::new(max_x, min_y),
            Point2::new(max_x, max_y),
            Point2::new(min_x, max_y),
        ]);
        let next_inset = inset + spacing_mm;
        if next_inset <= inset {
            break;
        }
        inset = next_inset;
    }
    loops
}

fn inset_points(path: &PrintPath, inset_mm: f64) -> Option<Vec<Point2>> {
    if path.points().len() == 2 {
        return inset_line(path.points(), inset_mm);
    }
    if path.is_closed() && path.points().len() == 4 {
        return match inset_rectangle(path.points(), inset_mm) {
            RectangleInset::Inset(points) => Some(points),
            RectangleInset::Collapsed => None,
            RectangleInset::NotRectangle => Some(path.points().to_vec()),
        };
    }
    Some(path.points().to_vec())
}

fn inset_line(points: &[Point2], inset_mm: f64) -> Option<Vec<Point2>> {
    let start = points[0];
    let end = points[1];
    let dx = end.x() - start.x();
    let dy = end.y() - start.y();
    let length = (dx * dx + dy * dy).sqrt();
    if length <= 2.0 * inset_mm + EPSILON {
        return None;
    }
    let unit_x = dx / length;
    let unit_y = dy / length;
    Some(vec![
        Point2::new(start.x() + unit_x * inset_mm, start.y() + unit_y * inset_mm),
        Point2::new(end.x() - unit_x * inset_mm, end.y() - unit_y * inset_mm),
    ])
}

enum RectangleInset {
    Inset(Vec<Point2>),
    Collapsed,
    NotRectangle,
}

fn inset_rectangle(points: &[Point2], inset_mm: f64) -> RectangleInset {
    let Some(bounds) = rectangle_bounds(points) else {
        return RectangleInset::NotRectangle;
    };
    if bounds.max_x - bounds.min_x <= 2.0 * inset_mm + EPSILON
        || bounds.max_y - bounds.min_y <= 2.0 * inset_mm + EPSILON
    {
        return RectangleInset::Collapsed;
    }
    RectangleInset::Inset(
        points
            .iter()
            .map(|point| {
                Point2::new(
                    if (point.x() - bounds.min_x).abs() <= EPSILON {
                        bounds.min_x + inset_mm
                    } else {
                        bounds.max_x - inset_mm
                    },
                    if (point.y() - bounds.min_y).abs() <= EPSILON {
                        bounds.min_y + inset_mm
                    } else {
                        bounds.max_y - inset_mm
                    },
                )
            })
            .collect(),
    )
}

fn rectangle_bounds(points: &[Point2]) -> Option<RectangleBounds> {
    let min_x = points
        .iter()
        .map(|point| point.x())
        .fold(f64::INFINITY, f64::min);
    let max_x = points
        .iter()
        .map(|point| point.x())
        .fold(f64::NEG_INFINITY, f64::max);
    let min_y = points
        .iter()
        .map(|point| point.y())
        .fold(f64::INFINITY, f64::min);
    let max_y = points
        .iter()
        .map(|point| point.y())
        .fold(f64::NEG_INFINITY, f64::max);
    if max_x - min_x <= EPSILON || max_y - min_y <= EPSILON {
        return None;
    }
    let corners = [
        Point2::new(min_x, min_y),
        Point2::new(max_x, min_y),
        Point2::new(max_x, max_y),
        Point2::new(min_x, max_y),
    ];
    let reversed = [corners[0], corners[3], corners[2], corners[1]];
    (cyclic_matches(points, corners) || cyclic_matches(points, reversed)).then_some(
        RectangleBounds {
            min_x,
            min_y,
            max_x,
            max_y,
        },
    )
}

fn cyclic_matches(points: &[Point2], corners: [Point2; 4]) -> bool {
    (0..4).any(|start| {
        points
            .iter()
            .enumerate()
            .all(|(index, point)| point_eq(*point, corners[(start + index) % 4]))
    })
}

fn point_eq(left: Point2, right: Point2) -> bool {
    (left.x() - right.x()).abs() <= EPSILON && (left.y() - right.y()).abs() <= EPSILON
}
