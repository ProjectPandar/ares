//! The NFP branch of `_NofitPolyPlacer::_trypack` (`nfpplacer.hpp:779-984` at `345efec64`): per
//! rotation, the item slides along every free-region outline and hole, minimized with subplex
//! from each strided corner. Upstream's parallel evaluations run in order here; their reductions
//! take the first minimum as `std::min_element` does.

use super::super::edges::EdgeCache;
use super::super::geometry::{add, shapes_convex_hull, sub};
use super::super::item::Item;
use super::super::{Alignment, Nest};
use super::Placer;
use crate::arrangement::nlopt::subplex_minimize;
use crate::geometry::{ExPolygon, Point};

/// The item sliding on an NFP: its reference vertex `iv` at `startpos` translation.
#[derive(Clone, Copy)]
struct Sliding<'a> {
    nest: &'a Nest,
    item: &'a Item,
    iv: Point,
    startpos: Point,
}

impl Sliding<'_> {
    /// The translation putting the reference vertex on `point`.
    fn translation_to(self, point: Point) -> Point {
        add(sub(point, self.iv), self.startpos)
    }
}

impl Placer {
    /// Tries every rotation from `start`, the item's translation and rotation on entry; returns
    /// the best score, leaving the item at its final placement, or at the origin in its starting
    /// rotation when no rotation packs.
    pub(super) fn slide(
        &self,
        nest: &mut Nest,
        item: &mut Item,
        remaining: &[usize],
        start: (Point, f64),
    ) -> Option<f64> {
        let (initial_tr, initial_rot) = start;
        let binbb = self.bin.bounding_box();
        let (mut final_tr, mut final_rot) = (Point::new(0, 0), initial_rot);
        let mut global_score = None;
        for &rot in &self.config.rotations {
            item.set_translation(initial_tr);
            item.set_rotation(initial_rot + rot);
            self.place_outside_of_bin(item);
            let nfps = self.calc_nfp(nest, item, binbb);
            nest.before_packing(&self.merged_pile, &self.items, remaining);
            let sliding = Sliding {
                nest: &*nest,
                item: &*item,
                iv: item.reference_vertex(),
                startpos: item.translation(),
            };
            let (best, best_score) = self.optimize_on(sliding, &nfps);
            let Some(point) =
                best.filter(|_| global_score.is_none_or(|global| best_score < global))
            else {
                continue;
            };
            final_tr = sliding.translation_to(point);
            final_rot = initial_rot + rot;
            global_score = Some(best_score);
        }
        item.set_translation(final_tr);
        item.set_rotation(final_rot);
        global_score
    }

    /// The best point on any NFP outline or hole whose pile stays on the bed, and its score.
    fn optimize_on(&self, sliding: Sliding, nfps: &[ExPolygon]) -> (Option<Point>, f64) {
        let mut best = (None, f64::MAX);
        for nfp in nfps {
            let mut cache = EdgeCache::new(nfp, f64::from(self.config.accuracy));
            for hole in std::iter::once(None).chain((0..cache.hole_count()).map(Some)) {
                best = self.better_on(sliding, &mut cache, hole, best);
            }
        }
        best
    }

    /// `best`, or the outline's minimum when it scores lower and its pile stays on the bed.
    fn better_on(
        &self,
        sliding: Sliding,
        cache: &mut EdgeCache,
        hole: Option<usize>,
        best: (Option<Point>, f64),
    ) -> (Option<Point>, f64) {
        match self.outline_minimum(sliding, cache, hole) {
            Some((point, score))
                if score < best.1 && self.boundary_check(sliding, point) <= 0.0 =>
            {
                (Some(point), score)
            }
            _ => best,
        }
    }

    /// The first minimum over the outline's (`None`) or a hole's corner starts, with
    /// `floor(1000 * accuracy)` evaluations and a 1e-20 relative tolerance each.
    fn outline_minimum(
        &self,
        sliding: Sliding,
        cache: &mut EdgeCache,
        hole: Option<usize>,
    ) -> Option<(Point, f64)> {
        let corners = match hole {
            None => cache.corners().to_vec(),
            Some(hole) => cache.hole_corners(hole).to_vec(),
        };
        let cache = &*cache;
        let point_at = |relpos: f64| match hole {
            None => cache.coords(relpos),
            Some(hole) => cache.hole_coords(hole, relpos),
        };
        let maxeval = (1000.0 * self.config.accuracy).floor() as usize;
        let mut minimum: Option<(f64, f64)> = None;
        for &pos in &corners {
            let mut itemcpy = sliding.item.clone();
            let mut objective = |x: &[f64]| {
                itemcpy.set_translation(sliding.translation_to(point_at(x[0])));
                sliding.nest.objective(&self.config.object_fn, &itemcpy)
            };
            let (optimum, score) =
                subplex_minimize(&mut objective, &[pos], (&[0.0], &[1.0]), 1e-20, maxeval);
            if minimum.is_none_or(|(_, min)| score < min) {
                minimum = Some((optimum[0], score));
            }
        }
        minimum.map(|(relpos, score)| (point_at(relpos), score))
    }

    /// `boundaryCheck`: how far the pile's hull with the item at `point` misses the bed.
    fn boundary_check(&self, sliding: Sliding, point: Point) -> f64 {
        let mut moved = sliding.item.clone();
        moved.set_translation(sliding.translation_to(point));
        let mut pile = self.merged_pile.clone();
        pile.push(moved.transformed_shape().clone());
        let hull = shapes_convex_hull(&pile);
        if self.config.alignment == Alignment::DontAlign {
            if self.bin.contains(&hull) { -1.0 } else { 1.0 }
        } else {
            self.bin.overfit(&hull)
        }
    }
}
