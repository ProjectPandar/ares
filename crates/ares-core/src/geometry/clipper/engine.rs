use super::polytree::PolyTree;
use super::types::ExecutionConfig;
use super::{ClipOperation, ClosedClipper, FillRule};
use crate::geometry::Polygon;

impl ClosedClipper {
    pub(crate) fn execute_paths(
        &mut self,
        operation: ClipOperation,
        subject_fill: FillRule,
        clip_fill: FillRule,
    ) -> Vec<Polygon> {
        self.using_polytree = false;
        let config = ExecutionConfig {
            operation,
            subject_fill,
            clip_fill,
        };
        let succeeded = self.execute_internal(config);
        let paths = if succeeded {
            self.build_paths()
        } else {
            Vec::new()
        };
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
        let succeeded = self.execute_internal(ExecutionConfig {
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
        #[cfg(test)]
        self.collected_maxima_for_test.clear();
        #[cfg(test)]
        self.simple_repairs_for_test.clear();
    }

    pub(super) fn execute_internal(&mut self, config: ExecutionConfig) -> bool {
        #[cfg(test)]
        self.simple_repairs_for_test.clear();
        self.reset_for_execute();
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
            if !self.process_intersections(top_y, config) {
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
            self.finish_output();
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
            if (out_rec.is_hole ^ self.options.reverse_solution)
                == (self.out_ring_area(points) > 0.0)
            {
                self.reverse_out_ring(points);
            }
        }
    }

    fn finish_output(&mut self) {
        self.fix_output_orientations();
        self.join_common_edges();
        for index in 0..self.out_recs.len() {
            if self.out_recs[index].points.is_some() {
                self.fixup_out_polygon(super::types::OutRecId(index));
            }
        }
        if self.options.strictly_simple {
            self.do_simple_polygons();
        }
    }
}
