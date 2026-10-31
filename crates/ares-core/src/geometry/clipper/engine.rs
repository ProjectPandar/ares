use super::polytree::PolyTree;
use super::round_policy::{Clipper1Round, RoundPolicy};
use super::types::ExecutionConfig;
use super::z::ZPath;
use super::{ClipOperation, Clipper, FillRule};
use crate::geometry::Polygon;

impl Clipper {
    pub(crate) fn execute_paths(
        &mut self,
        operation: ClipOperation,
        subject_fill: FillRule,
        clip_fill: FillRule,
    ) -> Result<Vec<Polygon>, super::ClipperError> {
        self.execute_paths_with_policy::<Clipper1Round>(operation, subject_fill, clip_fill)
    }

    pub(super) fn execute_paths_with_policy<P: RoundPolicy>(
        &mut self,
        operation: ClipOperation,
        subject_fill: FillRule,
        clip_fill: FillRule,
    ) -> Result<Vec<Polygon>, super::ClipperError> {
        if self.has_open_paths {
            return Err(super::ClipperError::OpenPathsRequirePolyTree);
        }
        self.using_polytree = false;
        let config = ExecutionConfig {
            operation,
            subject_fill,
            clip_fill,
        };
        let succeeded = self.execute_internal::<P>(config);
        let paths = if succeeded {
            self.build_paths()
        } else {
            Vec::new()
        };
        self.dispose_all_out_recs();
        Ok(paths)
    }

    pub(in crate::geometry) fn execute_z_paths(
        &mut self,
        operation: ClipOperation,
        subject_fill: FillRule,
        clip_fill: FillRule,
    ) -> (Vec<ZPath>, Vec<(i64, i64)>) {
        debug_assert!(self.z_intersections.is_none());
        self.z_intersections = Some(Vec::new());
        self.using_polytree = true;
        let succeeded = self.execute_internal::<Clipper1Round>(ExecutionConfig {
            operation,
            subject_fill,
            clip_fill,
        });
        let paths = if succeeded {
            self.build_polytree().into_z_paths()
        } else {
            Vec::new()
        };
        let intersections = self
            .z_intersections
            .take()
            .expect("Z execution owns its collector");
        self.dispose_all_out_recs();
        (paths, intersections)
    }

    pub(in crate::geometry) fn execute_z_source_paths(
        &mut self,
        operation: ClipOperation,
    ) -> Vec<ZPath> {
        self.z_fill_source_index = true;
        let (paths, _) = self.execute_z_paths(operation, FillRule::NonZero, FillRule::NonZero);
        self.z_fill_source_index = false;
        paths
    }

    pub(in crate::geometry) fn execute_z_width_paths(
        &mut self,
        operation: ClipOperation,
    ) -> Vec<ZPath> {
        self.z_fill_width = true;
        let (paths, _) = self.execute_z_paths(operation, FillRule::NonZero, FillRule::NonZero);
        self.z_fill_width = false;
        paths
    }

    /// `ClipperLib_Z::Clipper::Execute(ClipType, Paths&, ...)` with a
    /// `ZFillFunction` setting each intersection Z to `mark`.
    pub(in crate::geometry) fn execute_z_paths_marked(
        &mut self,
        operation: ClipOperation,
        subject_fill: FillRule,
        clip_fill: FillRule,
        mark: i64,
    ) -> Vec<ZPath> {
        self.z_intersections = Some(Vec::new());
        self.z_fill_mark = Some(mark);
        self.using_polytree = false;
        let succeeded = self.execute_internal::<Clipper1Round>(ExecutionConfig {
            operation,
            subject_fill,
            clip_fill,
        });
        let paths = if succeeded {
            self.build_z_paths()
        } else {
            Vec::new()
        };
        self.z_intersections = None;
        self.z_fill_mark = None;
        self.dispose_all_out_recs();
        paths
    }

    pub(crate) fn execute_polytree(
        &mut self,
        operation: ClipOperation,
        subject_fill: FillRule,
        clip_fill: FillRule,
    ) -> PolyTree {
        self.using_polytree = true;
        let succeeded = self.execute_internal::<Clipper1Round>(ExecutionConfig {
            operation,
            subject_fill,
            clip_fill,
        });
        let tree = if succeeded {
            self.build_polytree()
        } else {
            PolyTree::empty()
        };
        self.dispose_all_out_recs();
        tree
    }

    pub(crate) fn clear(&mut self) {
        self.minima.clear();
        self.edges.clear();
        self.use_full_range = false;
        self.scanbeam.clear();
        self.active_edges = None;
        self.sorted_edges = None;
        self.dispose_all_out_recs();
        self.joins.clear();
        self.ghost_joins.clear();
        self.intersections.clear();
        self.maxima.clear();
        self.has_open_paths = false;
        self.z_intersections = None;
        self.z_fill_mark = None;
        self.z_fill_width = false;
        self.z_fill_source_index = false;
    }

    pub(super) fn execute_internal<P: RoundPolicy>(&mut self, config: ExecutionConfig) -> bool {
        self.reset_for_execute::<P>();
        if self.minima.is_empty() {
            return true;
        }
        let mut bottom_y = self
            .pop_scanbeam()
            .expect("nonempty local minima must seed the scanbeam");
        let mut succeeded = true;
        loop {
            self.insert_local_minima_into_ael(bottom_y, config);
            self.process_horizontals(config);
            self.ghost_joins.clear();
            let Some(top_y) = self.pop_scanbeam() else {
                break;
            };
            if !self.process_intersections::<P>(top_y, config) {
                succeeded = false;
                break;
            }
            self.process_edges_at_top(top_y, config);
            bottom_y = top_y;
            if self.scanbeam.is_empty() && self.minima.is_empty() {
                break;
            }
        }

        self.maxima.clear();
        if succeeded {
            self.finish_output::<P>();
        }
        self.joins.clear();
        self.ghost_joins.clear();
        succeeded
    }

    fn pop_scanbeam(&mut self) -> Option<i64> {
        let y = self.scanbeam.pop()?;
        while self.scanbeam.peek() == Some(&y) {
            self.scanbeam.pop();
        }
        Some(y)
    }

    fn fix_output_orientations(&mut self) {
        for index in 0..self.out_recs.len() {
            let out_rec = self.out_recs[index];
            let Some(points) = out_rec.points else {
                continue;
            };
            if !out_rec.is_open
                && (out_rec.is_hole ^ self.options.reverse_solution)
                    == (self.out_ring_area(points) > 0.0)
            {
                self.reverse_out_ring(points);
            }
        }
    }

    fn finish_output<P: RoundPolicy>(&mut self) {
        self.fix_output_orientations();
        self.join_common_edges();
        for index in 0..self.out_recs.len() {
            self.fixup_output_record::<P>(index);
        }
        if self.options.strictly_simple {
            self.do_simple_polygons();
        }
    }

    fn fixup_output_record<P: RoundPolicy>(&mut self, index: usize) {
        if self.out_recs[index].points.is_none() {
            return;
        }
        let out_rec = super::types::OutRecId(index);
        if self.out_recs[index].is_open {
            self.fixup_out_polyline(out_rec);
        } else if P::PRESERVE_OUTPUT_FRONT {
            self.fixup_positive_front(out_rec);
        } else {
            self.fixup_out_polygon(out_rec);
        }
    }
}
