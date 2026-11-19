//! Width checks for Qhull 8.0.2 `qh_mergefacet` and `qh_check_maxout`.
//!
//! Copyright (c) 1993-2020 C.B. Barber.
//! See `QHULL-LICENSE.txt` for distribution and modification notices.

use super::{Hull, HullError, Id, TAIL};

const WIDE_RATIO: f64 = 100.0;
const WIDE_BASE_RATIO: f64 = 10.0 * WIDE_RATIO;

impl Hull {
    fn one_merge(&self) -> f64 {
        3.0 * self.precision.visible + self.precision.round
    }

    pub(super) fn merge_width(&mut self, from: Id, into: Id) -> Option<()> {
        let source = &self.facets[from];
        let target = &self.facets[into];
        let (min, max) = source
            .vertices
            .iter()
            .filter(|v| !target.vertices.contains(v))
            .fold((0.0_f64, 0.0_f64), |(min, max), &v| {
                let dist = target.distance(self.vertex(v));
                (min.min(dist), max.max(dist))
            });
        let limit = (WIDE_RATIO * self.one_merge())
            .max(source.max_outside)
            .max(target.max_outside);
        if max > limit || -min > limit {
            self.wide_merge = true;
            return None;
        }
        self.max_outside = self.max_outside.max(max);
        self.min_vertex = self.min_vertex.min(min);
        let target = &mut self.facets[into];
        target.max_outside = target.max_outside.max(max);
        let wide_facet = 6.0 * self.precision.visible;
        target.keep_centrum |= max > wide_facet || -min > wide_facet;
        Some(())
    }

    pub(super) fn check_precision(&mut self) -> Result<(), HullError> {
        if self.merges == 0 && !self.narrow && self.max_outside == 0.0 {
            return Ok(());
        }
        let mut owner = vec![TAIL; self.points.len()];
        let mut min = 0.0_f64;
        for id in self.suffix(self.first()) {
            let facet = &self.facets[id];
            for &point in facet.outside.iter().chain(&facet.coplanar) {
                owner[point] = id;
            }
            for &vertex in &facet.vertices {
                min = min.min(facet.distance(self.vertex(vertex)));
            }
        }
        for id in self.suffix(self.first()) {
            for &vertex in &self.facets[id].vertices {
                owner[self.vertices[vertex]] = id;
            }
        }
        let one_merge = self.one_merge();
        let base_max = self.max_outside.max(one_merge + self.precision.round);
        let base_min = (-self.min_vertex).max(one_merge + self.precision.round);
        let mut max = 0.0_f64;
        for (point, id) in owner.into_iter().enumerate() {
            if id != TAIL {
                let point = self.points[point];
                let dist = self.facets[id].distance(point);
                max = max.max(self.find_horizon_best(point, (id, dist), true).1);
            }
        }
        let wide = WIDE_BASE_RATIO * one_merge;
        let failed = if max / base_max > WIDE_RATIO {
            true
        } else if base_max > wide {
            max > wide
        } else if -min / base_min > WIDE_RATIO {
            true
        } else {
            base_min > wide && -min > wide
        };
        if failed {
            Err(HullError::Precision)
        } else {
            Ok(())
        }
    }
}
