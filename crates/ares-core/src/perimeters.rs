use crate::{LayerContours, Point2, SliceError};

mod fuzzy_skin;
mod fuzzy_skin_coherent_noise;
mod fuzzy_skin_noise;
mod options;
mod overhang;
mod overhang_reverse;
mod rectangles;
mod seams;
mod simplification;
mod thin_walls;
mod wall_loops;

pub(crate) use fuzzy_skin::FuzzySkinConfig;
pub use options::{PerimeterOptions, SeamPosition, WallDirection, WallGenerator, WallSequence};

use self::wall_loops::{loop_shrink, order_wall_sequence, resolve_wall_loops};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PerimeterRole {
    External,
    Overhang,
    Internal,
}

impl PerimeterRole {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::External => "external",
            Self::Overhang => "overhang",
            Self::Internal => "internal",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PerimeterPath {
    role: PerimeterRole,
    points: Vec<Point2>,
    unsupported_span_mm: Option<f64>,
    effective_line_width_mm: Option<f64>,
    seam_gap_mm: f64,
    closed: bool,
}

impl PerimeterPath {
    pub fn new(role: PerimeterRole, points: Vec<Point2>) -> Result<Self, SliceError> {
        if points.len() < 3 {
            return Err(SliceError::InvalidInput(
                "perimeter path requires at least three points".to_owned(),
            ));
        }
        Ok(Self {
            role,
            points,
            unsupported_span_mm: None,
            effective_line_width_mm: None,
            seam_gap_mm: 0.0,
            closed: true,
        })
    }

    pub fn open_external_thin_wall(points: Vec<Point2>) -> Result<Self, SliceError> {
        if points.len() < 2 {
            return Err(SliceError::InvalidInput(
                "thin wall perimeter path requires at least two points".to_owned(),
            ));
        }
        Ok(Self {
            role: PerimeterRole::External,
            points,
            unsupported_span_mm: None,
            effective_line_width_mm: None,
            seam_gap_mm: 0.0,
            closed: false,
        })
    }

    pub const fn role(&self) -> PerimeterRole {
        self.role
    }

    pub fn points(&self) -> &[Point2] {
        &self.points
    }

    pub const fn with_unsupported_span_mm(mut self, unsupported_span_mm: Option<f64>) -> Self {
        self.unsupported_span_mm = unsupported_span_mm;
        self
    }

    pub const fn unsupported_span_mm(&self) -> Option<f64> {
        self.unsupported_span_mm
    }

    pub const fn with_effective_line_width_mm(
        mut self,
        effective_line_width_mm: Option<f64>,
    ) -> Self {
        self.effective_line_width_mm = effective_line_width_mm;
        self
    }

    pub const fn effective_line_width_mm(&self) -> Option<f64> {
        self.effective_line_width_mm
    }

    pub const fn with_seam_gap_mm(mut self, seam_gap_mm: f64) -> Self {
        self.seam_gap_mm = seam_gap_mm;
        self
    }

    pub const fn seam_gap_mm(&self) -> f64 {
        self.seam_gap_mm
    }

    pub const fn is_closed(&self) -> bool {
        self.closed
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LayerPerimeters {
    layer_id: usize,
    print_z: f64,
    paths: Vec<PerimeterPath>,
}

impl LayerPerimeters {
    pub fn new(layer_id: usize, print_z: f64, paths: Vec<PerimeterPath>) -> Self {
        Self {
            layer_id,
            print_z,
            paths,
        }
    }

    pub const fn layer_id(&self) -> usize {
        self.layer_id
    }

    pub const fn print_z(&self) -> f64 {
        self.print_z
    }

    pub fn paths(&self) -> &[PerimeterPath] {
        &self.paths
    }
}

pub fn generate_perimeters(
    layers: &[LayerContours],
    options: PerimeterOptions,
) -> Result<Vec<LayerPerimeters>, SliceError> {
    let topmost_layer_id = layers.last().map(LayerContours::layer_id);
    layers
        .iter()
        .enumerate()
        .map(|(layer_index, layer)| {
            let mut paths = Vec::new();
            let previous_layer = layer_index
                .checked_sub(1)
                .and_then(|previous_index| layers.get(previous_index));
            for contour in layer.contours() {
                paths.extend(perimeters_for_contour(
                    contour.points(),
                    options,
                    layer,
                    topmost_layer_id,
                    previous_layer,
                )?);
            }
            Ok(LayerPerimeters::new(
                layer.layer_id(),
                layer.print_z(),
                paths,
            ))
        })
        .collect()
}

fn perimeters_for_contour(
    points: &[Point2],
    options: PerimeterOptions,
    layer: &LayerContours,
    topmost_layer_id: Option<usize>,
    previous_layer: Option<&LayerContours>,
) -> Result<Vec<PerimeterPath>, SliceError> {
    let layer_id = layer.layer_id();
    let print_z = layer.print_z();
    let effective_wall_loops = resolve_wall_loops(options, layer_id, topmost_layer_id);
    if effective_wall_loops == 0 {
        return Ok(Vec::new());
    }

    let external_role = overhang::external_role(points, previous_layer, options);
    let external_points = options.fuzzy_skin().external_points(
        simplification::simplify_closed_loop(
            options.wall_direction().orient_points(points.to_vec()),
            options,
        ),
        layer_id,
        print_z,
    );
    let mut paths = vec![
        PerimeterPath::new(
            external_role.role(),
            seams::position_loop(
                overhang_reverse::orient_points(
                    external_points,
                    external_role,
                    true,
                    layer_id,
                    options,
                ),
                external_role.role(),
                options,
                0.0,
            ),
        )?
        .with_unsupported_span_mm(external_role.unsupported_span_mm())
        .with_seam_gap_mm(options.seam_gap_mm()),
    ];
    let Some((min_x, min_y, max_x, max_y)) = rectangles::bounds(points) else {
        return Ok(order_wall_sequence(
            paths,
            options.wall_sequence(),
            layer_id,
        ));
    };

    for loop_index in 1..effective_wall_loops {
        let shrink = loop_shrink(loop_index, options)?;
        let inner_min_x = min_x + shrink;
        let inner_min_y = min_y + shrink;
        let inner_max_x = max_x - shrink;
        let inner_max_y = max_y - shrink;
        if inner_min_x >= inner_max_x || inner_min_y >= inner_max_y {
            continue;
        }
        let internal_points = options.fuzzy_skin().internal_wall_points(
            simplification::simplify_closed_loop(
                options.wall_direction().orient_points(vec![
                    Point2::new(inner_min_x, inner_min_y),
                    Point2::new(inner_max_x, inner_min_y),
                    Point2::new(inner_max_x, inner_max_y),
                    Point2::new(inner_min_x, inner_max_y),
                ]),
                options,
            ),
            layer_id,
            print_z,
        );
        paths.push(
            PerimeterPath::new(
                PerimeterRole::Internal,
                seams::position_loop(
                    overhang_reverse::orient_points(
                        internal_points,
                        external_role,
                        false,
                        layer_id,
                        options,
                    ),
                    PerimeterRole::Internal,
                    options,
                    shrink,
                ),
            )?
            .with_seam_gap_mm(options.seam_gap_mm()),
        );
    }
    if options.extra_perimeters_on_overhangs()
        && external_role.role() == PerimeterRole::Overhang
        && effective_wall_loops > 0
    {
        let shrink = loop_shrink(effective_wall_loops, options)?;
        let inner_min_x = min_x + shrink;
        let inner_min_y = min_y + shrink;
        let inner_max_x = max_x - shrink;
        let inner_max_y = max_y - shrink;
        if inner_min_x < inner_max_x && inner_min_y < inner_max_y {
            let extra_points = simplification::simplify_closed_loop(
                options.wall_direction().orient_points(vec![
                    Point2::new(inner_min_x, inner_min_y),
                    Point2::new(inner_max_x, inner_min_y),
                    Point2::new(inner_max_x, inner_max_y),
                    Point2::new(inner_min_x, inner_max_y),
                ]),
                options,
            );
            paths.push(
                PerimeterPath::new(
                    PerimeterRole::Overhang,
                    seams::position_loop(
                        overhang_reverse::orient_points(
                            extra_points,
                            external_role,
                            false,
                            layer_id,
                            options,
                        ),
                        PerimeterRole::Overhang,
                        options,
                        shrink,
                    ),
                )?
                .with_unsupported_span_mm(external_role.unsupported_span_mm())
                .with_seam_gap_mm(options.seam_gap_mm()),
            );
        }
    }
    thin_walls::append_rectangular_thin_wall(
        &mut paths,
        points,
        options,
        thin_walls::RectangularThinWallConfig {
            effective_wall_loops,
            layer_id,
            uses_surface_length_threshold: layer_id == 0 || topmost_layer_id == Some(layer_id),
        },
    )?;

    Ok(order_wall_sequence(
        paths,
        options.wall_sequence(),
        layer_id,
    ))
}

#[cfg(test)]
mod tests;
