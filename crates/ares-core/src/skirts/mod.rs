use crate::{LayerContours, Point2, SliceError};

mod brim_envelope;
mod min_length;
mod per_object;
#[cfg(test)]
mod tests;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DraftShield {
    Disabled,
    Enabled,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SkirtType {
    Combined,
    PerObject,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SkirtOptions {
    loops: u32,
    distance_mm: f64,
    height_layers: u32,
    speed_mm_s: f64,
    draft_shield: DraftShield,
    skirt_type: SkirtType,
    min_skirt_length_mm: f64,
    single_loop_draft_shield: bool,
    skirt_start_angle_degrees: f64,
}

impl SkirtOptions {
    pub const fn new(loops: u32, distance_mm: f64, height_layers: u32, speed_mm_s: f64) -> Self {
        Self {
            loops,
            distance_mm,
            height_layers,
            speed_mm_s,
            draft_shield: DraftShield::Disabled,
            skirt_type: SkirtType::Combined,
            min_skirt_length_mm: 0.0,
            single_loop_draft_shield: false,
            skirt_start_angle_degrees: -135.0,
        }
    }

    pub const fn loops(&self) -> u32 {
        self.loops
    }

    pub const fn distance_mm(&self) -> f64 {
        self.distance_mm
    }

    pub const fn height_layers(&self) -> u32 {
        self.height_layers
    }

    pub const fn speed_mm_s(&self) -> f64 {
        self.speed_mm_s
    }

    pub const fn draft_shield(&self) -> DraftShield {
        self.draft_shield
    }

    pub const fn skirt_type(&self) -> SkirtType {
        self.skirt_type
    }

    pub const fn min_skirt_length_mm(&self) -> f64 {
        self.min_skirt_length_mm
    }

    pub const fn single_loop_draft_shield(&self) -> bool {
        self.single_loop_draft_shield
    }

    pub const fn skirt_start_angle_degrees(&self) -> f64 {
        self.skirt_start_angle_degrees
    }

    pub const fn with_draft_shield(mut self, draft_shield: DraftShield) -> Self {
        self.draft_shield = draft_shield;
        self
    }

    pub const fn with_skirt_type(mut self, skirt_type: SkirtType) -> Self {
        self.skirt_type = skirt_type;
        self
    }

    pub const fn with_min_skirt_length_mm(mut self, min_skirt_length_mm: f64) -> Self {
        self.min_skirt_length_mm = min_skirt_length_mm;
        self
    }

    pub const fn with_single_loop_draft_shield(mut self, single_loop_draft_shield: bool) -> Self {
        self.single_loop_draft_shield = single_loop_draft_shield;
        self
    }

    pub const fn with_skirt_start_angle_degrees(mut self, skirt_start_angle_degrees: f64) -> Self {
        self.skirt_start_angle_degrees = skirt_start_angle_degrees;
        self
    }

    const fn is_draft_shield_enabled(&self) -> bool {
        matches!(self.draft_shield, DraftShield::Enabled)
    }

    const fn effective_loop_count(&self) -> u32 {
        if self.loops == 0 && self.is_draft_shield_enabled() {
            1
        } else {
            self.loops
        }
    }

    fn generates_on_layer(&self, layer: &LayerContours) -> bool {
        self.effective_loop_count() > 0
            && (layer.layer_id() < self.height_layers() as usize || self.is_draft_shield_enabled())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SkirtPath {
    points: Vec<Point2>,
}

impl SkirtPath {
    pub fn new(points: Vec<Point2>) -> Result<Self, SliceError> {
        if points.len() < 3 {
            return Err(SliceError::InvalidInput(
                "skirt path requires at least three points".to_owned(),
            ));
        }
        Ok(Self { points })
    }

    pub fn points(&self) -> &[Point2] {
        &self.points
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LayerSkirts {
    layer_id: usize,
    print_z: f64,
    paths: Vec<SkirtPath>,
}

impl LayerSkirts {
    pub fn new(layer_id: usize, print_z: f64, paths: Vec<SkirtPath>) -> Self {
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

    pub fn paths(&self) -> &[SkirtPath] {
        &self.paths
    }
}

pub fn generate_skirts(
    contours: &[LayerContours],
    options: SkirtOptions,
    effective_line_width: f64,
    skirt_extrusion_per_mm: f64,
) -> Result<Vec<LayerSkirts>, SliceError> {
    if !skirt_extrusion_per_mm.is_finite() || skirt_extrusion_per_mm <= 0.0 {
        return Err(SliceError::InvalidInput(
            "skirt extrusion per mm must be positive".to_owned(),
        ));
    }

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
                            brim_bounds: None,
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

pub(crate) fn generate_skirts_after_brims(
    contours: &[LayerContours],
    brims: &[crate::LayerBrims],
    options: SkirtOptions,
    effective_line_width: f64,
    skirt_extrusion_per_mm: f64,
) -> Result<Vec<LayerSkirts>, SliceError> {
    brim_envelope::generate_skirts_after_brims(
        contours,
        brims,
        options,
        effective_line_width,
        skirt_extrusion_per_mm,
    )
}

fn generate_layer_skirts(
    layer: &LayerContours,
    options: SkirtOptions,
    effective_line_width: f64,
    skirt_extrusion_per_mm: f64,
    context: LayerSkirtContext,
) -> Result<Vec<SkirtPath>, SliceError> {
    let Some(bounds) = contour_bounds(layer) else {
        return Ok(Vec::new());
    };
    let bounds = brim_envelope::merge_bounds(bounds, context.brim_bounds, options);

    generate_bounds_skirt_paths(
        bounds,
        options,
        effective_line_width,
        skirt_extrusion_per_mm,
        context.apply_min_length,
    )
}

#[derive(Clone, Copy)]
pub(in crate::skirts) struct LayerSkirtContext {
    apply_min_length: bool,
    brim_bounds: Option<min_length::Bounds>,
}

pub(in crate::skirts) fn generate_bounds_skirt_paths(
    bounds: min_length::Bounds,
    options: SkirtOptions,
    effective_line_width: f64,
    skirt_extrusion_per_mm: f64,
    apply_min_length: bool,
) -> Result<Vec<SkirtPath>, SliceError> {
    let min_x = bounds.min_x;
    let min_y = bounds.min_y;
    let max_x = bounds.max_x;
    let max_y = bounds.max_y;
    let loop_count = min_length::loop_count(min_length::LoopCountInput {
        configured_loops: options.effective_loop_count(),
        min_skirt_length_mm: options.min_skirt_length_mm(),
        distance_mm: options.distance_mm(),
        effective_line_width,
        bounds,
        skirt_extrusion_per_mm,
        apply_min_length,
    })?;

    loop_indices(loop_count, options, apply_min_length)
        .into_iter()
        .enumerate()
        .map(|(path_position, loop_index)| {
            let expand = options.distance_mm() + f64::from(loop_index) * effective_line_width;
            let mut points = vec![
                Point2::new(min_x - expand, min_y - expand),
                Point2::new(max_x + expand, min_y - expand),
                Point2::new(max_x + expand, max_y + expand),
                Point2::new(min_x - expand, max_y + expand),
            ];
            maybe_apply_start_angle(&mut points, options, apply_min_length && path_position == 0);
            SkirtPath::new(points)
        })
        .collect()
}

fn loop_indices(loop_count: u32, options: SkirtOptions, apply_min_length: bool) -> Vec<u32> {
    if apply_min_length || !options.single_loop_draft_shield() {
        (0..loop_count).collect()
    } else {
        loop_count.checked_sub(1).into_iter().collect()
    }
}

fn maybe_apply_start_angle(points: &mut [Point2], options: SkirtOptions, apply_start_angle: bool) {
    if !apply_start_angle {
        return;
    }
    let Some(start_index) = start_angle_corner_index(points, options.skirt_start_angle_degrees())
    else {
        return;
    };
    points.rotate_left(start_index);
}

fn start_angle_corner_index(points: &[Point2], angle_degrees: f64) -> Option<usize> {
    let (min_x, min_y, max_x, max_y) = path_bounds(points)?;
    let center_x = (min_x + max_x) / 2.0;
    let center_y = (min_y + max_y) / 2.0;
    let radius = ((center_x - min_x).powi(2) + (center_y - min_y).powi(2)).sqrt();
    let radians = angle_degrees.to_radians();
    let target_x = center_x + radius * radians.cos();
    let target_y = center_y + radius * radians.sin();

    points
        .iter()
        .enumerate()
        .min_by(|(_, left), (_, right)| {
            squared_distance(left, target_x, target_y)
                .total_cmp(&squared_distance(right, target_x, target_y))
        })
        .map(|(index, _)| index)
}

fn path_bounds(points: &[Point2]) -> Option<(f64, f64, f64, f64)> {
    let first = points.first()?;
    let mut min_x = first.x();
    let mut min_y = first.y();
    let mut max_x = first.x();
    let mut max_y = first.y();

    for point in &points[1..] {
        min_x = min_x.min(point.x());
        min_y = min_y.min(point.y());
        max_x = max_x.max(point.x());
        max_y = max_y.max(point.y());
    }

    Some((min_x, min_y, max_x, max_y))
}

fn squared_distance(point: &Point2, target_x: f64, target_y: f64) -> f64 {
    (point.x() - target_x).powi(2) + (point.y() - target_y).powi(2)
}

fn contour_bounds(layer: &LayerContours) -> Option<min_length::Bounds> {
    let mut points = layer.contours().iter().flat_map(|contour| contour.points());
    let first = points.next()?;
    let mut bounds = min_length::Bounds {
        min_x: first.x(),
        min_y: first.y(),
        max_x: first.x(),
        max_y: first.y(),
    };

    for point in points {
        bounds.min_x = bounds.min_x.min(point.x());
        bounds.min_y = bounds.min_y.min(point.y());
        bounds.max_x = bounds.max_x.max(point.x());
        bounds.max_y = bounds.max_y.max(point.y());
    }

    Some(bounds)
}
