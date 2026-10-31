//! libnest2d's `_NofitPolyPlacer` (`placers/nfpplacer.hpp:440-1244` at `345efec64`) with the
//! convex-only NFP. The first object goes to the starting corner; later ones slide along the
//! free region's outline (`slide.rs`); a pile aligns when its placer is done (`align.rs`).

#[path = "align.rs"]
mod align;
#[path = "slide.rs"]
mod slide;

use super::geometry::{
    NBox, add, merge, nfp_convex_only, nfp_inner_rect_bed, point_touches, point_within, sub,
    subtract,
};
use super::item::{Item, translated};
use super::{Alignment, Bin, Nest, ObjectFn};
use crate::geometry::{ExPolygon, Point};

/// `LARGE_COST_TO_REJECT` (`nester.hpp`).
pub(crate) const LARGE_COST_TO_REJECT: f64 = 1e7;

/// `NfpPConfig` with the fields arrange sets.
#[derive(Clone, Debug)]
pub(crate) struct PlacerConfig {
    pub(crate) rotations: Vec<f64>,
    pub(crate) alignment: Alignment,
    pub(crate) starting_point: Alignment,
    pub(crate) best_object_pos: Point,
    pub(crate) object_fn: ObjectFn,
    pub(crate) accuracy: f32,
    /// `m_excluded_items`: the excluded regions, by arena index.
    pub(crate) excluded_items: Vec<usize>,
}

/// `PackResult`.
#[derive(Clone, Copy, Debug)]
pub(crate) struct PackResult {
    pub(crate) item: Option<usize>,
    pub(crate) translation: Point,
    pub(crate) rotation: f64,
    pub(crate) score: f64,
}

impl PackResult {
    /// `PackResult(overfit)`: no item and the default score.
    const FAILED: Self = Self {
        item: None,
        translation: Point::new(0, 0),
        rotation: 0.0,
        score: -1.11,
    };
}

pub(crate) struct Placer {
    pub(crate) bin: Bin,
    pub(crate) items: Vec<usize>,
    pub(crate) config: PlacerConfig,
    pub(crate) score: f64,
    merged_pile: Vec<ExPolygon>,
}

/// `should_check_collision` of `fill_config` (`Arrange.cpp:392-398`).
fn should_check_collision(candidate: &Item, packed: &Item) -> bool {
    if packed.is_bed_exclusion {
        return super::exclusion_applies(candidate, packed);
    }
    if candidate.is_bed_exclusion {
        return super::exclusion_applies(packed, candidate);
    }
    true
}

/// `correctNfpPosition`: moves the NFP so its reference point is the orbiter's top-right vertex
/// when its bottom-left vertex touches the stationary shape's top-right one.
pub(super) fn correct_nfp_position(
    nfp: &mut (ExPolygon, Point),
    stationary_top: Point,
    orbiter: &Item,
) {
    let dtouch = sub(stationary_top, orbiter.leftmost_bottom_vertex());
    let top_other = add(orbiter.reference_vertex(), dtouch);
    nfp.0 = translated(&nfp.0, sub(top_other, nfp.1));
}

impl Placer {
    pub(crate) fn new(bin: Bin, config: PlacerConfig) -> Self {
        Self {
            bin,
            items: Vec::new(),
            config,
            score: 0.0,
            merged_pile: Vec::new(),
        }
    }

    /// `preload`, then the arranger's `on_preload` (`Arrange.cpp:907-919`): with fixed items the
    /// objective starts from the bed's corner or centre, or from a wipe tower.
    pub(crate) fn preload(&mut self, nest: &Nest, items: &[usize]) {
        self.items.extend_from_slice(items);
        if items.is_empty() {
            return;
        }
        let binbb = self.bin.bounding_box();
        let mut starting_point = if self.config.starting_point == Alignment::BottomLeft {
            binbb.min
        } else {
            binbb.center()
        };
        if let Some(tower) = items.iter().find(|&&index| nest.items[index].is_wipe_tower) {
            starting_point = nest.items[*tower].bounding_box().center();
        }
        self.config.object_fn = ObjectFn::Preloaded {
            binbb,
            starting_point,
        };
    }

    pub(crate) fn accept(&mut self, nest: &mut Nest, result: &PackResult) {
        let Some(index) = result.item else {
            return;
        };
        let item = &mut nest.items[index];
        item.set_translation(result.translation);
        item.set_rotation(result.rotation);
        self.items.push(index);
        let mut pile = merge(&self.merged_pile);
        pile.push(nest.items[index].transformed_shape().clone());
        self.merged_pile = merge(&pile);
        self.score += result.score;
    }

    /// `clearItems(clearFunc)`: aligns the pile, then drops the items `clear` matches.
    pub(crate) fn clear_items(&mut self, nest: &mut Nest, clear: impl Fn(&Item) -> bool) {
        self.final_align(nest);
        self.items.retain(|&index| !clear(&nest.items[index]));
    }

    /// The `calcnfp` of an item: where its reference vertex may go inside `bed` clear of the
    /// packed items it may collide with.
    fn calc_nfp(&self, nest: &Nest, item: &Item, bed: NBox) -> Vec<ExPolygon> {
        let nfps: Vec<ExPolygon> = self
            .items
            .iter()
            .map(|&index| &nest.items[index])
            .filter(|packed| should_check_collision(item, packed))
            .map(|packed| {
                let mut nfp = nfp_convex_only(packed.transformed_shape(), item.transformed_shape());
                correct_nfp_position(&mut nfp, packed.reference_vertex(), item);
                nfp.0
            })
            .collect();
        let inner: Vec<ExPolygon> = nfp_inner_rect_bed(bed, item.transformed_shape())
            .into_iter()
            .collect();
        subtract(&inner, &nfps)
    }

    /// `setInitialPosition`: the item's starting-point corner on the bed shrunk by 10 units.
    fn set_initial_position(&self, item: &mut Item) {
        let bb = item.bounding_box();
        let bin = self.bin.bounding_box();
        let bin = NBox {
            min: add(bin.min, Point::new(10, 10)),
            max: sub(bin.max, Point::new(10, 10)),
            defined: bin.defined,
        };
        let (ci, cb) = match self.config.starting_point {
            Alignment::Center => (bb.center(), bin.center()),
            Alignment::BottomLeft => (bb.min, bin.min),
            Alignment::TopRight => (bb.max, bin.max),
            _ => (Point::new(0, 0), Point::new(0, 0)),
        };
        item.translate(sub(cb, ci));
    }

    /// `placeOutsideOfBin`: the item's bottom-right corner to the bed's top-right one.
    fn place_outside_of_bin(&self, item: &mut Item) {
        let bb = item.bounding_box();
        let binbb = self.bin.bounding_box();
        item.translate(Point::new(
            binbb.max.x() - bb.max.x(),
            binbb.max.y() - bb.min.y(),
        ));
    }

    /// `_trypack` (`nfpplacer.hpp:672-1007`). The item in the arena ends at the tried position.
    pub(crate) fn try_pack(
        &mut self,
        nest: &mut Nest,
        index: usize,
        remaining: &[usize],
    ) -> PackResult {
        let mut item = nest.items[index].clone();
        let start = (item.translation(), item.rotation());
        let first_object = self
            .items
            .iter()
            .all(|&packed| nest.items[packed].is_virt_object && !nest.items[packed].is_wipe_tower);
        let mut score = None;
        if first_object {
            score = self.place_first(nest, &mut item, start);
        }
        if score.is_none() {
            score = self.slide(nest, &mut item, remaining, start);
        }
        let stored = &mut nest.items[index];
        stored.set_translation(item.translation());
        stored.set_rotation(item.rotation());
        match score {
            Some(score) => PackResult {
                item: Some(index),
                translation: item.translation(),
                rotation: item.rotation(),
                score,
            },
            None => PackResult::FAILED,
        }
    }

    /// The first object at the starting corner in its best-fitting rotation; it packs with the
    /// score 0.2 when it fits and misses the virtual objects. `start` is the item's translation
    /// and rotation on entry.
    fn place_first(&self, nest: &Nest, item: &mut Item, start: (Point, f64)) -> Option<f64> {
        let (initial_tr, initial_rot) = start;
        self.set_initial_position(item);
        let (mut best_tr, mut best_rot) = (item.translation(), item.rotation());
        let mut best_overfit =
            self.bin.overfit(item.transformed_shape()) + self.virtual_overlap(nest, item);
        for &rot in &self.config.rotations {
            item.set_translation(initial_tr);
            item.set_rotation(initial_rot + rot);
            self.set_initial_position(item);
            let of = self.bin.overfit(item.transformed_shape());
            if of + self.virtual_overlap(nest, item) < best_overfit {
                best_overfit = of;
                best_tr = item.translation();
                best_rot = item.rotation();
            }
        }
        item.set_rotation(best_rot);
        item.set_translation(best_tr);
        (best_overfit <= 0.0).then_some(0.2)
    }

    /// `overlapWithVirtObject`: 0 when the item's reference vertex is inside or on an NFP.
    fn virtual_overlap(&self, nest: &Nest, item: &Item) -> f64 {
        if self.items.is_empty() {
            return 0.0;
        }
        let v = item.reference_vertex();
        let clear = self
            .calc_nfp(nest, item, self.bin.bounding_box())
            .iter()
            .any(|nfp| point_within(v, nfp) || point_touches(v, nfp));
        if clear { 0.0 } else { 1.0 }
    }
}
