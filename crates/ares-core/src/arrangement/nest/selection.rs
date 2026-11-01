//! libnest2d's `_FirstFitSelection::packItems` (`selections/firstfit.hpp:29-202` at `345efec64`)
//! and `remove_unpackable_items` (`selection_boilerplate.hpp:33-55`): each item goes to the bed
//! where the scores of every bed sum lowest, opening beds as needed. Placers align their pile
//! when they go out of scope, as upstream's destructors do.

use super::item::Item;
use super::placer::{LARGE_COST_TO_REJECT, PackResult, Placer, PlacerConfig};
use super::{Bin, Nest};
use crate::geometry::fixed_gcc_sort_by;

/// `BIN_ID_UNFIT`.
const BIN_ID_UNFIT: i32 = -1;
/// `MAX_NUM_PLATES` (`common.hpp:14`).
const MAX_NUM_PLATES: usize = 36;

/// The open beds of one `packItems` run.
struct FirstFit<'a> {
    bin: &'a Bin,
    config: &'a PlacerConfig,
    placers: Vec<Placer>,
    /// The fixed items by their bed.
    fixed_bins: Vec<Vec<usize>>,
    next_item_id: i32,
}

/// An item's search over the beds: the next bed to try and the best and first-fitting beds
/// found so far, with their results.
#[derive(Default)]
struct Search {
    next_bed: usize,
    best: Option<(usize, PackResult)>,
    best_all: f64,
    first_fit: Option<(usize, PackResult)>,
}

/// `clearItems`' matcher: the fixed items other than a wipe tower.
fn fixed_but_wipe_tower(item: &Item) -> bool {
    item.fixed && !item.is_wipe_tower
}

/// Packs the arena items `items` into copies of `bin`, in the arranger's `sortfunc` order.
pub(crate) fn pack_items(nest: &mut Nest, items: &[usize], bin: &Bin, config: &PlacerConfig) {
    let mut store = Vec::new();
    let mut fixed_bins: Vec<Vec<usize>> = Vec::new();
    for &index in items {
        let item = &mut nest.items[index];
        if !item.fixed {
            store.push(index);
            continue;
        }
        if item.bin_id < 0 {
            item.bin_id = 0;
        }
        let bin_index = item.bin_id as usize;
        if fixed_bins.len() <= bin_index {
            fixed_bins.resize(bin_index + 1, Vec::new());
        }
        fixed_bins[bin_index].push(index);
    }
    fixed_gcc_sort_by(&mut store, |&a, &b| {
        nest.arranger.before(&nest.items[a], &nest.items[b])
    });
    remove_unpackable_items(nest, &store, bin, config);
    let mut run = FirstFit {
        bin,
        config,
        placers: Vec::with_capacity(store.len()),
        fixed_bins,
        next_item_id: 0,
    };
    for (position, &index) in store.iter().enumerate() {
        if nest.items[index].bin_id != BIN_ID_UNFIT {
            run.pack(nest, index, &store[position + 1..]);
        }
    }
    for placer in &run.placers {
        placer.final_align(nest);
    }
}

impl FirstFit<'_> {
    /// One item of the `packItems` loop (`firstfit.hpp:116-197`).
    fn pack(&mut self, nest: &mut Nest, index: usize, remaining: &[usize]) {
        let mut search = Search {
            best_all: f64::MAX,
            ..Search::default()
        };
        let at_limit = |candidate: Option<(usize, PackResult)>| {
            candidate.is_some_and(|(bed, _)| bed == MAX_NUM_PLATES)
        };
        loop {
            self.scan(nest, index, remaining, &mut search);
            if at_limit(search.best) && at_limit(search.first_fit) {
                nest.items[index].bin_id = BIN_ID_UNFIT;
                return;
            }
            if at_limit(search.best) {
                search.best = search.first_fit;
            }
            if let Some((bed, result)) = search.best {
                self.accept(nest, index, bed, &result);
                return;
            }
            if self.retry_with_excluded(nest, index) {
                search.next_bed -= 1;
                continue;
            }
            let mut placer = Placer::new(self.bin.clone(), self.config.clone());
            if let Some(fixed) = self.fixed_bins.get(self.placers.len()) {
                placer.preload(nest, &fixed.clone());
            }
            self.placers.push(placer);
            search.next_bed = self.placers.len() - 1;
        }
    }

    /// Tries the beds from `search.next_bed` on, recording the first and the best by summed
    /// score.
    fn scan(&mut self, nest: &mut Nest, index: usize, remaining: &[usize], search: &mut Search) {
        for bed in search.next_bed..self.placers.len() {
            let result = self.placers[bed].try_pack(nest, index, remaining);
            let score_all = self
                .placers
                .iter()
                .fold(result.score, |sum, placer| sum + placer.score);
            if !(0.0..LARGE_COST_TO_REJECT).contains(&result.score) {
                continue;
            }
            if search.first_fit.is_none() {
                search.first_fit = Some((bed, result));
            }
            if score_all < search.best_all {
                search.best_all = score_all;
                search.best = Some((bed, result));
            }
        }
        search.next_bed = self.placers.len();
    }

    /// Accepts the item on `bed`; an item that needed the excluded regions cleared restores the
    /// last bed's fixed items on it.
    fn accept(&mut self, nest: &mut Nest, index: usize, bed: usize, result: &PackResult) {
        nest.items[index].bin_id = bed as i32;
        nest.items[index].item_id = self.next_item_id;
        self.next_item_id += 1;
        self.placers[bed].accept(nest, result);
        if !nest.items[index].has_tried_with_excluded {
            return;
        }
        self.placers[bed].clear_items(nest, fixed_but_wipe_tower);
        if self.fixed_bins.len() >= self.placers.len() {
            let fixed = self.fixed_bins[self.placers.len() - 1].clone();
            self.placers[bed].preload(nest, &fixed);
        }
    }

    /// When the last bed is still empty, retries it once with only the excluded regions, then
    /// restores its fixed items (`firstfit.hpp:169-180`). Returns whether to retry.
    fn retry_with_excluded(&mut self, nest: &mut Nest, index: usize) -> bool {
        let Some(last) = self.placers.len().checked_sub(1) else {
            return false;
        };
        // `getPackedSize() == 0` (`nester.hpp:673-681`) counts the items that are not fixed: a bed
        // holding only fixed items is still empty.
        if self.placers[last]
            .items
            .iter()
            .any(|&item| !nest.items[item].fixed)
        {
            return false;
        }
        self.placers[last].clear_items(nest, fixed_but_wipe_tower);
        if !nest.items[index].has_tried_with_excluded {
            nest.items[index].has_tried_with_excluded = true;
            self.placers[last].preload(nest, &self.config.excluded_items);
            return true;
        }
        // Upstream indexes `fixed_bins` past its end when the bed has none; nothing is preloaded.
        if let Some(fixed) = self.fixed_bins.get(last) {
            self.placers[last].preload(nest, &fixed.clone());
        }
        false
    }
}

/// `remove_unpackable_items`: an item that does not fit an empty bed with the excluded regions,
/// or has no area, is unfit. The probe placer aligns its pile on leaving.
fn remove_unpackable_items(nest: &mut Nest, store: &[usize], bin: &Bin, config: &PlacerConfig) {
    let mut probe = Placer::new(bin.clone(), config.clone());
    probe.preload(nest, &config.excluded_items);
    for &index in store {
        let copy = nest.items[index].clone();
        nest.items.push(copy);
        let result = probe.try_pack(nest, nest.items.len() - 1, &[]);
        nest.items.pop();
        if nest.items[index].area() <= 0.0 || result.item.is_none() {
            nest.items[index].bin_id = BIN_ID_UNFIT;
        }
    }
    probe.final_align(nest);
}
