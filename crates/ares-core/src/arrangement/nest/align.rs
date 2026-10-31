//! `_NofitPolyPlacer::finalAlign` (`nfpplacer.hpp:1047-1205` at `345efec64`), run when a placer
//! clears its items or goes out of scope.

use super::super::geometry::{
    NBox, add, merge_boxes, nfp_convex_only, nfp_inner_rect_bed, point_within, rightmost_up_vertex,
    shapes_convex_hull, sub, subtract,
};
use super::super::item::Item;
use super::super::{Alignment, Bin, Nest, bounding_circle, projection_onto};
use super::{Placer, correct_nfp_position};
use crate::geometry::{ExPolygon, Point};

impl Placer {
    /// The `calcnfp` of a pile hull against fixed shapes (`finalAlign`).
    fn calc_nfp_of_shape(
        sliding: &ExPolygon,
        stationaries: &[ExPolygon],
        bed: NBox,
    ) -> Vec<ExPolygon> {
        let sliding_item = Item::new(sliding.clone());
        let nfps: Vec<ExPolygon> = stationaries
            .iter()
            .map(|stationary| {
                let mut nfp = nfp_convex_only(stationary, sliding);
                correct_nfp_position(
                    &mut nfp,
                    rightmost_up_vertex(stationary.contour().points()),
                    &sliding_item,
                );
                nfp.0
            })
            .collect();
        let inner: Vec<ExPolygon> = nfp_inner_rect_bed(bed, sliding).into_iter().collect();
        subtract(&inner, &nfps)
    }

    /// `finalAlign`: moves the pile to the alignment point, kept on the bed and, with fixed shapes,
    /// clear of them.
    pub(crate) fn final_align(&self, nest: &mut Nest) {
        if self.items.is_empty() || self.config.alignment == Alignment::DontAlign {
            return;
        }
        if let Bin::Circle { center, .. } = self.bin {
            let shapes: Vec<ExPolygon> = self
                .items
                .iter()
                .map(|&index| nest.items[index].transformed_shape().clone())
                .collect();
            let hull = shapes_convex_hull(&shapes);
            let circle = bounding_circle(&hull);
            let d = sub(center, circle.0);
            for &index in &self.items {
                nest.items[index].translate(d);
            }
            return;
        }
        let bbin = self.bin.bounding_box();
        let bb = self
            .items
            .iter()
            .map(|&index| &nest.items[index])
            .filter(|item| !item.is_virt_object)
            .fold(NBox::undefined(), |bb, item| {
                merge_boxes(item.bounding_box(), bb)
            });
        let (ci, cb) = match self.config.alignment {
            Alignment::Center => (bb.center(), bbin.center()),
            Alignment::BottomLeft => (bb.min, bbin.min),
            Alignment::TopRight => (bb.max, bbin.max),
            Alignment::UserDefined => (bb.center(), self.config.best_object_pos),
            Alignment::DontAlign => (Point::new(0, 0), Point::new(0, 0)),
        };
        let on_bin = |lo: i64, hi: i64, bin_lo: i64, bin_hi: i64, mut shift: i64| {
            if hi - lo >= bin_hi - bin_lo {
                return (bin_lo + bin_hi) / 2 - (lo + hi) / 2;
            }
            if lo + shift < bin_lo {
                shift = bin_lo - lo;
            }
            if hi + shift > bin_hi {
                shift = bin_hi - hi;
            }
            shift
        };
        let d = sub(cb, ci);
        let mut d = Point::new(
            on_bin(bb.min.x(), bb.max.x(), bbin.min.x(), bbin.max.x(), d.x()),
            on_bin(bb.min.y(), bb.max.y(), bbin.min.y(), bbin.max.y(), d.y()),
        );
        let (mut objs, mut excludes) = (Vec::new(), Vec::new());
        for &index in &self.items {
            let item = &nest.items[index];
            if item.fixed {
                excludes.push(item.transformed_shape().clone())
            } else {
                objs.push(item.transformed_shape().clone())
            }
        }
        if objs.is_empty() {
            return;
        }
        if !excludes.is_empty() {
            let hull = shapes_convex_hull(&objs);
            let nfps = Self::calc_nfp_of_shape(&hull, &excludes, bbin);
            if nfps.is_empty() {
                return;
            }
            let ref_aligned = add(Item::new(hull).reference_vertex(), d);
            if !nfps.iter().any(|nfp| point_within(ref_aligned, nfp)) {
                d = add(d, sub(projection_onto(&nfps, ref_aligned), ref_aligned));
            }
        }
        for &index in &self.items {
            if !nest.items[index].is_virt_object {
                nest.items[index].translate(d);
            }
        }
    }
}
