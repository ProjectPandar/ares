//! `FillPlanePath.cpp:308–338::SmoothingPolylineOutput`.

use super::output::InfillPolylineOutput;
use crate::{
    fill::corner_smoothing::{CornerSmoother, CornerSmoothing},
    geometry::{ClipperError, Point},
};

pub(super) trait PolylineOutput {
    fn reserve(&mut self, count: usize);
    fn add_point(&mut self, x: f64, y: f64) -> Result<(), ClipperError>;
}

impl PolylineOutput for InfillPolylineOutput {
    fn reserve(&mut self, count: usize) {
        self.reserve(count);
    }

    fn add_point(&mut self, x: f64, y: f64) -> Result<(), ClipperError> {
        self.add_point(x, y)
    }
}

pub(super) struct SmoothingPolylineOutput<'a> {
    output: &'a mut InfillPolylineOutput,
    smoother: CornerSmoother,
    last_snapped: Option<Point>,
}

impl<'a> SmoothingPolylineOutput<'a> {
    pub(super) fn new(output: &'a mut InfillPolylineOutput, smoothing: CornerSmoothing) -> Self {
        Self {
            output,
            smoother: CornerSmoother::new(smoothing),
            last_snapped: None,
        }
    }

    pub(super) fn finish(&mut self) -> Result<(), ClipperError> {
        self.smoother
            .flush(&mut emitter(self.output, &mut self.last_snapped))
    }
}

impl PolylineOutput for SmoothingPolylineOutput<'_> {
    fn reserve(&mut self, count: usize) {
        self.output.reserve(count);
    }

    fn add_point(&mut self, x: f64, y: f64) -> Result<(), ClipperError> {
        self.smoother
            .push([x, y], &mut emitter(self.output, &mut self.last_snapped))
    }
}

fn emitter<'a>(
    output: &'a mut InfillPolylineOutput,
    last_snapped: &'a mut Option<Point>,
) -> impl FnMut([f64; 2]) -> Result<(), ClipperError> + 'a {
    |[x, y]| {
        let snapped = output.scaled_point(x, y)?;
        if *last_snapped == Some(snapped) {
            return Ok(());
        }
        *last_snapped = Some(snapped);
        output.add_point(x, y)
    }
}
