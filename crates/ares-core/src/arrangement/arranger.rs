//! `Arrange.cpp`'s `AutoArranger` (`:455-1090` at `345efec64`): the objective the placers
//! minimize, its `before_packing` state and the item order.

use super::nest::bin::overfit_box;
use super::nest::geometry::{NBox, distance, merge_boxes, shapes_box, shapes_convex_hull};
use super::nest::item::Item;
use super::nest::placer::LARGE_COST_TO_REJECT;
use super::nest::{Alignment, Bin, ObjectFn};
use crate::geometry::{ExPolygon, Point};

/// `BIG_ITEM_TRESHOLD` (`Arrange.cpp:105`).
const BIG_ITEM_THRESHOLD: f64 = 0.02;

/// The `ArrangeParams` the objective reads.
#[derive(Clone, Debug)]
pub(crate) struct ObjectiveParams {
    pub(crate) is_seq_print: bool,
    pub(crate) clearance_height_to_lid: f64,
    pub(crate) clearance_height_to_rod: f64,
    pub(crate) printable_height: f64,
    pub(crate) allow_multi_materials_on_same_plate: bool,
}

pub(crate) struct Arranger {
    bin: Bin,
    bin_area: f64,
    norm: f64,
    pub(crate) starting_point: Alignment,
    params: ObjectiveParams,
    merged_pile: Vec<ExPolygon>,
    pilebb: NBox,
    remaining: Vec<usize>,
    items: Vec<usize>,
    /// The bounding boxes of the big packed items and of every packed item, by `items` position.
    rtree: Vec<(NBox, usize)>,
    small_rtree: Vec<(NBox, usize)>,
    pub(crate) item_count: usize,
}

/// `Print::is_filaments_compatible` (`Print.cpp:1797-1813`): no high- with low-temperature type.
fn filaments_compatible(types: [i32; 2]) -> bool {
    !(types.contains(&0) && types.contains(&1))
}

/// `std::includes` over sorted unique sets.
fn includes(
    outer: &std::collections::BTreeSet<i32>,
    inner: &std::collections::BTreeSet<i32>,
) -> bool {
    inner.is_subset(outer)
}

impl Arranger {
    pub(crate) fn new(bin: Bin, starting_point: Alignment, params: ObjectiveParams) -> Self {
        let bin_area = bin.area().abs();
        Self {
            bin,
            bin_area,
            norm: bin_area.sqrt(),
            starting_point,
            params,
            merged_pile: Vec::new(),
            pilebb: NBox::undefined(),
            remaining: Vec::new(),
            items: Vec::new(),
            rtree: Vec::new(),
            small_rtree: Vec::new(),
            item_count: 0,
        }
    }

    fn norm(&self, value: f64) -> f64 {
        value / self.norm
    }

    fn is_big(&self, area: f64) -> bool {
        area / self.bin_area > BIG_ITEM_THRESHOLD
    }

    fn corner_packing(&self) -> bool {
        self.starting_point == Alignment::BottomLeft
    }

    /// `before_packing` (`Arrange.cpp:878-903`).
    pub(crate) fn before_packing(
        &mut self,
        arena: &[Item],
        pile: &[ExPolygon],
        packed: &[usize],
        remaining: &[usize],
    ) {
        self.items = packed.to_vec();
        self.merged_pile = pile.to_vec();
        self.remaining = remaining.to_vec();
        self.pilebb = if pile.is_empty() {
            NBox::undefined()
        } else {
            NBox {
                defined: true,
                ..shapes_box(pile)
            }
        };
        self.rtree.clear();
        self.small_rtree.clear();
        for (position, &index) in packed.iter().enumerate() {
            let item = &arena[index];
            if item.is_virt_object {
                continue;
            }
            if self.is_big(item.area()) {
                self.rtree.push((item.bounding_box(), position));
            }
            self.small_rtree.push((item.bounding_box(), position));
        }
    }

    /// The placer's `object_function`.
    pub(crate) fn objective(&self, arena: &[Item], object_fn: &ObjectFn, item: &Item) -> f64 {
        match *object_fn {
            ObjectFn::Preloaded {
                binbb,
                starting_point,
            } => {
                // `fixed_overfit` (`Arrange.cpp:416-426`).
                let (score, pile) = self.objfunc(arena, item, starting_point);
                let diff = merge_boxes(pile, binbb).area() - binbb.area();
                if diff > 0.0 { score + diff } else { score }
            }
            ObjectFn::Bin { origin_pack } => {
                let (score, fullbb) = self.objfunc(arena, item, origin_pack);
                self.bin_penalty(item, score, fullbb)
            }
        }
    }

    /// The per bin type `get_objfn` on top of `objfunc` (`Arrange.cpp:1005-1090`).
    fn bin_penalty(&self, item: &Item, score: f64, fullbb: NBox) -> f64 {
        let squared_miss = |miss: f64| miss.max(0.0) * miss.max(0.0);
        match &self.bin {
            Bin::Box(bin) => {
                let score = score + squared_miss(overfit_box(fullbb, *bin));
                if score > LARGE_COST_TO_REJECT {
                    1.5 * LARGE_COST_TO_REJECT
                } else {
                    score
                }
            }
            Bin::Circle { .. } if self.is_big(item.area()) => {
                score + squared_miss(self.bin.overfit(&self.hull_with(item)))
            }
            Bin::Circle { .. } => score,
            Bin::Polygon(_) if self.corner_packing() => {
                if self.bin.contains(&self.hull_with(item)) {
                    score
                } else {
                    score + LARGE_COST_TO_REJECT
                }
            }
            Bin::Polygon(_) => score + squared_miss(self.bin.overfit(&self.hull_with(item))),
        }
    }

    fn hull_with(&self, item: &Item) -> ExPolygon {
        let mut pile = self.merged_pile.clone();
        pile.push(item.transformed_shape().clone());
        shapes_convex_hull(&pile)
    }

    /// `dist_for_BOTTOM_LEFT`.
    fn dist_for_bottom_left(&self, ibb: NBox, origin: Point) -> f64 {
        let dy = (ibb.min.y() - origin.y()) as f64;
        let dx = (ibb.min.x() - origin.x()) as f64;
        let mut bindist = (ibb.max.y() - ibb.min.y()) as f64;
        if dx >= 0.0 && dy >= 0.0 {
            bindist += dy + dx;
        } else {
            if dx < 0.0 {
                bindist += 10.0 * -dx;
            }
            if dy < 0.0 {
                bindist += 10.0 * -dy;
            }
        }
        self.norm(bindist)
    }

    fn dist_to_bin(&self, ibb: NBox, origin: Point) -> f64 {
        self.norm(match self.starting_point {
            Alignment::BottomLeft => distance(ibb.min, origin),
            Alignment::TopRight => distance(ibb.max, origin),
            _ => distance(ibb.center(), origin),
        })
    }

    /// `objfunc` (`Arrange.cpp:556-847`): the score and the pile box with the item.
    fn objfunc(&self, arena: &[Item], item: &Item, origin: Point) -> (f64, NBox) {
        let ibb = item.bounding_box();
        let fullbb = merge_boxes(self.pilebb, ibb);
        let bigbb = if self.rtree.is_empty() {
            fullbb
        } else {
            self.rtree
                .iter()
                .fold(NBox::undefined(), |bb, (b, _)| merge_boxes(*b, bb))
        };
        let item_area = item.area();
        let big = self.is_big(item_area) || self.rtree.is_empty();
        let score = if !self.params.is_seq_print && big && !self.remaining.is_empty() {
            self.big_item_score(arena, item, fullbb, origin)
        } else if big && self.remaining.is_empty() {
            if self.corner_packing() {
                self.dist_for_bottom_left(ibb, origin)
            } else if self.pilebb.defined {
                0.5 * self.norm(distance(ibb.center(), self.pilebb.center()))
            } else {
                0.5 * self.norm(distance(ibb.center(), origin))
            }
        } else if self.corner_packing() {
            self.dist_for_bottom_left(ibb, origin)
        } else {
            0.8 * self.norm(distance(ibb.center(), bigbb.center()))
                + 0.2 * self.norm(distance(ibb.center(), origin))
        };
        (self.placement_penalties(arena, item, ibb, score), fullbb)
    }

    /// The `BIG_ITEM` case (`Arrange.cpp:618-700`).
    fn big_item_score(&self, arena: &[Item], item: &Item, fullbb: NBox, origin: Point) -> f64 {
        let ibb = item.bounding_box();
        let cc = fullbb.center();
        let corners = [
            ibb.min,
            ibb.max,
            ibb.center(),
            Point::new(ibb.min.x(), ibb.max.y()),
            Point::new(ibb.max.x(), ibb.min.y()),
        ];
        let mut dist = self.norm(
            corners
                .iter()
                .map(|&corner| distance(corner, cc))
                .fold(f64::MAX, f64::min),
        );
        if self.corner_packing() {
            return 0.2 * dist + 0.8 * self.dist_for_bottom_left(ibb, origin);
        }
        dist = 0.8 * dist + 0.2 * self.dist_to_bin(ibb, origin);
        let item_area = item.area();
        let index = if self.is_big(item_area) {
            &self.rtree
        } else {
            &self.small_rtree
        };
        let mut alignment_score = 1.0_f64;
        let mut found = false;
        for &(bb, position) in index {
            if bb.max.x() < ibb.min.x()
                || bb.min.x() > ibb.max.x()
                || bb.max.y() < ibb.min.y()
                || bb.min.y() > ibb.max.y()
            {
                continue;
            }
            found = true;
            let packed = &arena[self.items[position]];
            let parea = packed.area();
            if (1.0 - parea / item_area).abs() < 1e-6 {
                let bbarea = merge_boxes(packed.bounding_box(), ibb).area();
                alignment_score = alignment_score.min(1.0 - (item_area + parea) / bbarea);
            }
        }
        let density = (self.norm(fullbb.width() as f64) * self.norm(fullbb.height() as f64)).sqrt();
        let remaining_ratio = self.remaining.len() as f64 / self.item_count as f64;
        let alignment_weight = (0.6 * item_area / ibb.area()).max(0.3);
        if found {
            (1.0 - 0.2 - alignment_weight) * dist
                + (1.0 - remaining_ratio) * 0.20 * density
                + alignment_weight * alignment_score
        } else {
            0.50 * dist + 0.50 * density
        }
    }

    /// Adds the sequential-print conflicts or height similarity, the filament temperature and
    /// colour penalties to `score`, in upstream's order (`Arrange.cpp:726-826`).
    fn placement_penalties(&self, arena: &[Item], item: &Item, ibb: NBox, mut score: f64) -> f64 {
        let packed: Vec<&Item> = self
            .items
            .iter()
            .map(|&index| &arena[index])
            .filter(|packed| !packed.is_virt_object)
            .collect();
        score = if self.params.is_seq_print {
            self.sequential_penalty(item, &packed, score)
        } else {
            self.layered_score(item, ibb, &packed, score)
        };
        let mut extruders: std::collections::BTreeSet<i32> = packed
            .iter()
            .flat_map(|p| p.extrude_ids.iter().copied())
            .collect();
        let own: std::collections::BTreeSet<i32> = item.extrude_ids.iter().copied().collect();
        if !self.params.allow_multi_materials_on_same_plate
            && !(extruders.is_empty() || includes(&own, &extruders) || includes(&extruders, &own))
        {
            score += LARGE_COST_TO_REJECT * 1.3;
        }
        let before = extruders.len();
        extruders.extend(own);
        if !self.params.is_seq_print {
            score += (extruders.len() - before) as f64;
        }
        score
    }

    /// The sequential-print row and lid height conflicts and the incompatible filaments
    /// (`Arrange.cpp:726-765`).
    fn sequential_penalty(&self, item: &Item, packed: &[&Item], mut score: f64) -> f64 {
        let bb = item.bounding_box();
        let rod = self.params.clearance_height_to_rod;
        let row_conflict = |p: &&Item| {
            let pbb = p.bounding_box();
            let overlap = bb.max.y().min(pbb.max.y()) - bb.min.y().max(pbb.min.y());
            // Of two overlapping rows the left object prints first.
            let first_printed = if bb.min.x() < pbb.min.x() {
                item.height
            } else {
                p.height
            };
            match (bb.max.y() < pbb.min.y(), overlap > 0) {
                (true, _) => item.height > rod,
                (false, true) => first_printed > rod,
                (false, false) => false,
            }
        };
        let row = packed.iter().any(row_conflict);
        let lid = packed
            .iter()
            .any(|p| p.height > self.params.clearance_height_to_lid);
        for _ in packed
            .iter()
            .filter(|p| !filaments_compatible([item.filament_temp_type, p.filament_temp_type]))
        {
            score += LARGE_COST_TO_REJECT * 1.1;
        }
        let lambda4 = LARGE_COST_TO_REJECT * 1.2;
        score + (lambda4 * f64::from(u8::from(row)) + lambda4 * f64::from(u8::from(lid)))
    }

    /// Layered printing keeps similar heights together and rejects incompatible filaments
    /// (`Arrange.cpp:766-784`).
    fn layered_score(&self, item: &Item, ibb: NBox, packed: &[&Item], mut score: f64) -> f64 {
        let mut height_score = 0.0;
        let mut counted = 0_u32;
        for p in packed {
            counted += 1;
            height_score += (1.0 - (item.height - p.height).abs() / self.params.printable_height)
                * self.norm(distance(ibb.center(), p.bounding_box().center()));
            if !filaments_compatible([item.filament_temp_type, p.filament_temp_type]) {
                score += LARGE_COST_TO_REJECT;
                break;
            }
        }
        if counted > 0 {
            score += height_score / f64::from(counted);
        }
        score
    }

    /// `sortfunc` (`Arrange.cpp:946-960`).
    pub(crate) fn before(&self, a: &Item, b: &Item) -> bool {
        if a.priority != b.priority {
            return a.priority > b.priority;
        }
        if a.bed_temp != b.bed_temp {
            return a.bed_temp > b.bed_temp;
        }
        let front = |item: &Item| item.extrude_ids.first().copied().unwrap_or(0);
        if self.params.is_seq_print {
            if a.height != b.height {
                return a.height < b.height;
            }
            if (a.area() / b.area() - 1.0).abs() > 0.2 {
                return a.area() > b.area();
            }
            return front(a) < front(b);
        }
        if a.extrude_ids != b.extrude_ids {
            front(a) < front(b)
        } else {
            a.area() > b.area()
        }
    }
}
