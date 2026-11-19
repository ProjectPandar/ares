use crate::{LayerContours, PerimeterOptions, Point2, SliceError};

mod wall;

#[derive(Clone, Debug, PartialEq)]
pub struct GapFillPath {
    points: Vec<Point2>,
}

impl GapFillPath {
    pub fn new(points: Vec<Point2>) -> Result<Self, SliceError> {
        if points.len() < 2 {
            return Err(SliceError::InvalidInput(
                "gap-fill path requires at least two points".to_owned(),
            ));
        }
        Ok(Self { points })
    }

    pub fn points(&self) -> &[Point2] {
        &self.points
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LayerGapFills {
    layer_id: usize,
    print_z: f64,
    paths: Vec<GapFillPath>,
}

impl LayerGapFills {
    pub fn new(layer_id: usize, print_z: f64, paths: Vec<GapFillPath>) -> Self {
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

    pub fn paths(&self) -> &[GapFillPath] {
        &self.paths
    }
}

pub fn generate_gap_fills(
    layers: &[LayerContours],
    options: PerimeterOptions,
    gap_infill_speed_mm_s: f64,
) -> Result<Vec<LayerGapFills>, SliceError> {
    wall::generate(layers, options, gap_infill_speed_mm_s)
}
