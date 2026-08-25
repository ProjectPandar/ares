use crate::geometry::CoordinateScale;

use super::regions::MonotonicRegion;
use super::segments::RectilinearSlice;

const SOURCE_EPSILON: f32 = 1.0e-4;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct AntPath {
    pub(crate) length: f32,
    pub(crate) visibility: f32,
    pub(crate) pheromone: f32,
}

pub(crate) struct MonotonicPathMatrix<'a> {
    regions: &'a [MonotonicRegion],
    slice: &'a RectilinearSlice,
    scale: CoordinateScale,
    paths: Vec<AntPath>,
}

impl<'a> MonotonicPathMatrix<'a> {
    pub(crate) fn new(
        regions: &'a [MonotonicRegion],
        slice: &'a RectilinearSlice,
        scale: CoordinateScale,
        initial_pheromone: f32,
    ) -> Self {
        Self {
            regions,
            slice,
            scale,
            paths: vec![
                AntPath {
                    length: -1.0,
                    visibility: -1.0,
                    pheromone: initial_pheromone,
                };
                regions.len() * regions.len() * 4
            ],
        }
    }

    pub(crate) fn update_initial_pheromone(&mut self, initial_pheromone: f32) {
        for path in &mut self.paths {
            path.pheromone = initial_pheromone;
        }
    }

    pub(crate) fn edge(
        &mut self,
        from: usize,
        from_flipped: bool,
        to: usize,
        to_flipped: bool,
    ) -> &mut AntPath {
        let index = (2 * from + from_flipped as usize) * self.regions.len() * 2
            + 2 * to
            + to_flipped as usize;
        if self.paths[index].length == -1.0 {
            let from_region = &self.regions[from];
            let to_region = &self.regions[to];
            let from_point = self.slice.lines[from_region.right.line].intersections
                [from_region.right_intersection(from_flipped)]
            .point;
            let to_point = self.slice.lines[to_region.left.line].intersections
                [to_region.left_intersection(to_flipped)]
            .point;
            let x = (to_point.x() - from_point.x()) as f32;
            let y = (to_point.y() - from_point.y()) as f32;
            let length = (x * x + y * y).sqrt() * self.scale.factor() as f32;
            self.paths[index].length = length;
            self.paths[index].visibility = 1.0 / (length + SOURCE_EPSILON);
        }
        &mut self.paths[index]
    }
}
