use crate::{LayerContours, Point2, SliceError};

mod combine;
mod ears;
mod efc_outline;

const DEFAULT_BRIM_EARS_MAX_ANGLE_DEGREES: f64 = 125.0;
const DEFAULT_BRIM_EARS_DETECTION_LENGTH_MM: f64 = 1.0;
const MAX_BRIM_LOOPS: u32 = 10_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BrimType {
    AutoBrim,
    BrimEars,
    Painted,
    OuterOnly,
    InnerOnly,
    OuterAndInner,
    NoBrim,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BrimOptions {
    width_mm: f64,
    object_gap_mm: f64,
    brim_type: BrimType,
    combine_brims: bool,
    efc_outline_offset_mm: Option<f64>,
    brim_ears_max_angle_degrees: f64,
    brim_ears_detection_length_mm: f64,
}

impl BrimOptions {
    pub const fn new(width_mm: f64, object_gap_mm: f64, brim_type: BrimType) -> Self {
        Self {
            width_mm,
            object_gap_mm,
            brim_type,
            combine_brims: false,
            efc_outline_offset_mm: None,
            brim_ears_max_angle_degrees: DEFAULT_BRIM_EARS_MAX_ANGLE_DEGREES,
            brim_ears_detection_length_mm: DEFAULT_BRIM_EARS_DETECTION_LENGTH_MM,
        }
    }

    pub const fn width_mm(&self) -> f64 {
        self.width_mm
    }

    pub const fn object_gap_mm(&self) -> f64 {
        self.object_gap_mm
    }

    pub const fn brim_type(&self) -> BrimType {
        self.brim_type
    }

    pub const fn combine_brims(&self) -> bool {
        self.combine_brims
    }

    pub const fn efc_outline_offset_mm(&self) -> Option<f64> {
        self.efc_outline_offset_mm
    }

    pub const fn brim_ears_max_angle_degrees(&self) -> f64 {
        self.brim_ears_max_angle_degrees
    }

    pub const fn brim_ears_detection_length_mm(&self) -> f64 {
        self.brim_ears_detection_length_mm
    }

    pub(crate) const fn with_brim_ears_max_angle_degrees(mut self, value: f64) -> Self {
        self.brim_ears_max_angle_degrees = value;
        self
    }

    pub(crate) const fn with_brim_ears_detection_length_mm(mut self, value: f64) -> Self {
        self.brim_ears_detection_length_mm = value;
        self
    }

    pub(crate) const fn with_combine_brims(mut self, value: bool) -> Self {
        self.combine_brims = value;
        self
    }

    pub(crate) const fn with_efc_outline_offset_mm(mut self, value: Option<f64>) -> Self {
        self.efc_outline_offset_mm = value;
        self
    }

    const fn generates_outer_brim(&self) -> bool {
        matches!(
            self.brim_type,
            BrimType::AutoBrim | BrimType::BrimEars | BrimType::OuterOnly | BrimType::OuterAndInner
        )
    }

    const fn generates_inner_brim(&self) -> bool {
        matches!(
            self.brim_type,
            BrimType::InnerOnly | BrimType::OuterAndInner
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct BrimPath {
    points: Vec<Point2>,
}

impl BrimPath {
    pub fn new(points: Vec<Point2>) -> Result<Self, SliceError> {
        if points.len() < 3 {
            return Err(SliceError::InvalidInput(
                "brim path requires at least three points".to_owned(),
            ));
        }
        Ok(Self { points })
    }

    pub fn points(&self) -> &[Point2] {
        &self.points
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LayerBrims {
    layer_id: usize,
    print_z: f64,
    paths: Vec<BrimPath>,
}

impl LayerBrims {
    pub fn new(layer_id: usize, print_z: f64, paths: Vec<BrimPath>) -> Self {
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

    pub fn paths(&self) -> &[BrimPath] {
        &self.paths
    }
}

pub fn generate_brims(
    contours: &[LayerContours],
    options: BrimOptions,
    effective_line_width: f64,
) -> Result<Vec<LayerBrims>, SliceError> {
    contours
        .iter()
        .map(|layer| {
            let paths = if layer.layer_id() == 0
                && options.width_mm() > 0.0
                && (options.generates_outer_brim() || options.generates_inner_brim())
            {
                generate_layer_brims(layer, options, effective_line_width)?
            } else {
                Vec::new()
            };
            Ok(LayerBrims::new(layer.layer_id(), layer.print_z(), paths))
        })
        .collect()
}

fn generate_layer_brims(
    layer: &LayerContours,
    options: BrimOptions,
    effective_line_width: f64,
) -> Result<Vec<BrimPath>, SliceError> {
    if layer.contours().is_empty() {
        return Ok(Vec::new());
    }

    let loop_count = brim_loop_count(options.width_mm(), effective_line_width)?;
    let mut paths = Vec::new();
    if options.combine_brims()
        && options.generates_outer_brim()
        && options.brim_type() != BrimType::BrimEars
    {
        paths.extend(combine::outer_brim_paths(
            layer,
            options,
            effective_line_width,
            loop_count,
        )?);
    } else {
        for contour in layer.contours() {
            if options.generates_outer_brim() && layer.is_outer_contour(contour) {
                extend_outer_brim_paths(
                    &mut paths,
                    contour,
                    options,
                    effective_line_width,
                    loop_count,
                )?;
            }
        }
    }
    for contour in layer.contours() {
        if options.generates_inner_brim() && !layer.is_outer_contour(contour) {
            paths.extend(inner_brim_paths(
                contour,
                options,
                effective_line_width,
                loop_count,
            )?);
        }
    }
    Ok(paths)
}

fn extend_outer_brim_paths(
    paths: &mut Vec<BrimPath>,
    contour: &crate::Contour,
    options: BrimOptions,
    effective_line_width: f64,
    loop_count: u32,
) -> Result<(), SliceError> {
    if options.brim_type() == BrimType::BrimEars {
        paths.extend(brim_ear_paths(
            contour,
            options,
            effective_line_width,
            loop_count,
        )?);
    } else {
        paths.extend(outer_brim_paths(
            contour,
            options,
            effective_line_width,
            loop_count,
        )?);
    }
    Ok(())
}

fn brim_loop_count(width_mm: f64, effective_line_width: f64) -> Result<u32, SliceError> {
    if !effective_line_width.is_finite() || effective_line_width <= 0.0 {
        return Err(SliceError::InvalidInput(
            "brim line width must be positive".to_owned(),
        ));
    }
    let loop_count = (width_mm / effective_line_width).ceil();
    if loop_count > f64::from(MAX_BRIM_LOOPS) {
        Err(SliceError::InvalidInput(
            "brim loop count exceeds supported limit".to_owned(),
        ))
    } else {
        Ok(loop_count as u32)
    }
}

fn outer_brim_paths(
    contour: &crate::Contour,
    options: BrimOptions,
    effective_line_width: f64,
    loop_count: u32,
) -> Result<Vec<BrimPath>, SliceError> {
    let Some(bounds) = contour_bounds(contour) else {
        return Ok(Vec::new());
    };
    let Some(bounds) = efc_outline::bounds_or_adjusted(bounds, options.efc_outline_offset_mm())
    else {
        return Ok(Vec::new());
    };

    combine::outer_brim_bounds_paths(bounds, options, effective_line_width, loop_count)
}

fn brim_ear_paths(
    contour: &crate::Contour,
    options: BrimOptions,
    effective_line_width: f64,
    loop_count: u32,
) -> Result<Vec<BrimPath>, SliceError> {
    ears::brim_ear_paths(contour, options, effective_line_width, loop_count)
}

fn inner_brim_paths(
    contour: &crate::Contour,
    options: BrimOptions,
    effective_line_width: f64,
    loop_count: u32,
) -> Result<Vec<BrimPath>, SliceError> {
    let Some((min_x, min_y, max_x, max_y)) = contour_bounds(contour) else {
        return Ok(Vec::new());
    };

    let mut paths = Vec::new();
    for loop_index in 0..loop_count {
        let brim_offset =
            (f64::from(loop_index + 1) * effective_line_width).min(options.width_mm());
        let shrink = options.object_gap_mm() + brim_offset;
        let inner_min_x = min_x + shrink;
        let inner_min_y = min_y + shrink;
        let inner_max_x = max_x - shrink;
        let inner_max_y = max_y - shrink;
        if inner_min_x >= inner_max_x || inner_min_y >= inner_max_y {
            continue;
        }
        paths.push(BrimPath::new(vec![
            Point2::new(inner_min_x, inner_min_y),
            Point2::new(inner_max_x, inner_min_y),
            Point2::new(inner_max_x, inner_max_y),
            Point2::new(inner_min_x, inner_max_y),
        ])?);
    }
    Ok(paths)
}

fn contour_bounds(contour: &crate::Contour) -> Option<(f64, f64, f64, f64)> {
    let mut points = contour.points().iter();
    let first = points.next()?;
    let mut min_x = first.x();
    let mut min_y = first.y();
    let mut max_x = first.x();
    let mut max_y = first.y();

    for point in points {
        min_x = min_x.min(point.x());
        min_y = min_y.min(point.y());
        max_x = max_x.max(point.x());
        max_y = max_y.max(point.y());
    }

    Some((min_x, min_y, max_x, max_y))
}

#[cfg(test)]
mod tests;
