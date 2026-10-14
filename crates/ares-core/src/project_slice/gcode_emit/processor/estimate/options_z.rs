//! `OptionsZCorrector` (`GCodeProcessor.hpp:678-711`) seen by the queued
//! `TimeBlock`s. A block keeps the `move_id` it got at creation
//! (`GCodeProcessor.cpp:4004`); only `calculate_time`'s actual-speed
//! insertion renumbers it (`:6003-6008`). `set()` stores `moves.size() - 1`
//! and nothing renumbers it, so a flush between `set()` and `update()` leaves
//! the index pointing at an earlier element. `update()` erases whatever sits
//! there, and every queued block at or after it resolves to the next element,
//! whose type decides its `g1_times_cache` entry (`:484-488`, `:574`).

use std::ops::Range;

use crate::gcode_processor::{EMoveType, MoveVertex};
use crate::project_slice::gcode_emit::processor::motion::Trapezoid;
use crate::project_slice::gcode_emit::processor::moves::actual_speed::{is_timed, split_count};

/// The block of a `VG1`, which stores no vertex.
const NO_VERTEX: u32 = u32::MAX;

/// Blocks queued and planner events pushed so far.
#[derive(Clone, Copy, Default)]
struct Clock {
    blocks: usize,
    events: usize,
}

struct Update {
    /// The marker vertex `set()` indexed.
    marker: u32,
    set: Clock,
    /// The vertex `update()` appends.
    copy: u32,
    update: Clock,
}

/// Every stored vertex in creation order, erased ones included, and the
/// first vertex each block stored.
pub(in crate::project_slice::gcode_emit::processor) struct Log {
    kinds: Vec<EMoveType>,
    positions: Vec<[f32; 3]>,
    blocks: Vec<u32>,
    events: usize,
    set: Option<(u32, Clock)>,
    updates: Vec<Update>,
}

impl Log {
    pub(in crate::project_slice::gcode_emit::processor) fn new(origin: &MoveVertex) -> Self {
        Self {
            kinds: vec![origin.move_type],
            positions: vec![origin.position],
            blocks: Vec::new(),
            events: 0,
            set: None,
            updates: Vec::new(),
        }
    }

    /// The planner events pushed before the current line.
    pub(in crate::project_slice::gcode_emit::processor) fn tick(&mut self, events: usize) {
        self.events = events;
    }

    pub(in crate::project_slice::gcode_emit::processor) fn next_vertex(&self) -> u32 {
        self.kinds.len() as u32
    }

    /// Queues the block of a command whose vertices start at `first`. Its
    /// `move_id` (`:4004`) skips the copy an `update()` (`:3929`) appended
    /// before the command's own vertices.
    pub(in crate::project_slice::gcode_emit::processor) fn block(&mut self, first: u32) {
        let updated = self
            .updates
            .last()
            .is_some_and(|update| update.copy == first);
        self.blocks.push(first + u32::from(updated));
    }

    pub(in crate::project_slice::gcode_emit::processor) fn virtual_block(&mut self) {
        self.blocks.push(NO_VERTEX);
    }

    pub(in crate::project_slice::gcode_emit::processor) fn vertex(
        &mut self,
        kind: EMoveType,
        position: [f32; 3],
    ) {
        self.kinds.push(kind);
        self.positions.push(position);
    }

    fn clock(&self) -> Clock {
        Clock {
            blocks: self.blocks.len(),
            events: self.events,
        }
    }

    /// `OptionsZCorrector::set` (`GCodeProcessor.hpp:688-691`).
    pub(in crate::project_slice::gcode_emit::processor) fn set(&mut self) {
        self.set = Some((self.next_vertex() - 1, self.clock()));
    }

    /// `OptionsZCorrector::update` (`GCodeProcessor.hpp:693-705`) appends a
    /// copy whose type is only known once the flushes are planned.
    pub(in crate::project_slice::gcode_emit::processor) fn update(&mut self, position: [f32; 3]) {
        let Some((marker, set)) = self.set.take() else {
            return;
        };
        self.vertex(self.kinds[marker as usize], position);
        self.updates.push(Update {
            marker,
            set,
            copy: self.next_vertex() - 1,
            update: self.clock(),
        });
    }

    /// Corrects `cache_eligible` of the blocks the updates shifted.
    /// `pushed[i]` is the processed-block count once block `i` is queued and
    /// `flush_ends[e]` the count after planner event `e`.
    pub(in crate::project_slice::gcode_emit::processor) fn correct(
        &mut self,
        trapezoids: &[Trapezoid],
        pushed: &[usize],
        flush_ends: &[usize],
        cache_eligible: &mut [bool],
    ) {
        debug_assert_eq!(self.blocks.len(), cache_eligible.len());
        let processed = |clock: Clock| {
            let refreshed = clock.blocks.checked_sub(1).map_or(0, |i| pushed[i]);
            let flushed = clock.events.checked_sub(1).map_or(0, |e| flush_ends[e]);
            refreshed.max(flushed)
        };
        let updates = std::mem::take(&mut self.updates);
        let mut list = List {
            log: &*self,
            trapezoids,
            resolved: self.blocks.clone(),
            erased: Vec::new(),
        };
        let mut copies = Vec::with_capacity(updates.len());
        for update in &updates {
            let queued = processed(update.update)..update.update.blocks;
            let flushed = processed(update.set)..queued.start;
            let steps = list.inserted_before(update.marker, flushed, &copies);
            let (vertex, kind) = list.erase(update.marker, steps, queued.start, &copies);
            copies.push((update.copy, kind));
            list.shift(queued, vertex);
        }
        for ((eligible, &vertex), &own) in cache_eligible
            .iter_mut()
            .zip(&list.resolved)
            .zip(&self.blocks)
        {
            if vertex != own {
                *eligible = is_timed(list.kind(vertex, &copies));
            }
        }
    }
}

/// `GCodeProcessorResult::moves` as seen through the stored vertices.
struct List<'a> {
    log: &'a Log,
    trapezoids: &'a [Trapezoid],
    /// The vertex each block's `move_id` resolves to.
    resolved: Vec<u32>,
    erased: Vec<u32>,
}

impl List<'_> {
    /// The actual-speed vertices `calculate_time` inserts in front of the
    /// vertex `block` resolves to (`GCodeProcessor.cpp:497-546`).
    fn splits(&self, block: usize, copies: &[(u32, EMoveType)]) -> usize {
        let vertex = self.resolved[block];
        if vertex == NO_VERTEX || !is_timed(self.kind(vertex, copies)) {
            return 0;
        }
        split_count(
            self.log.positions[self.previous(vertex) as usize],
            self.log.positions[vertex as usize],
            &self.trapezoids[block],
        )
    }

    fn kind(&self, vertex: u32, copies: &[(u32, EMoveType)]) -> EMoveType {
        copies
            .iter()
            .find_map(|&(copy, kind)| (copy == vertex).then_some(kind))
            .unwrap_or(self.log.kinds[vertex as usize])
    }

    /// Vertices the flushes between `set()` and `update()` inserted in
    /// front of `marker`; each pushes it one past the stored index.
    fn inserted_before(
        &self,
        marker: u32,
        flushed: Range<usize>,
        copies: &[(u32, EMoveType)],
    ) -> usize {
        flushed
            .filter(|&block| self.resolved[block] < marker)
            .map(|block| self.splits(block, copies))
            .sum()
    }

    /// Erases the element `steps` in front of `marker` and returns the first
    /// vertex at or after it with the erased element's type; an actual-speed
    /// vertex has the type of the vertex it splits.
    fn erase(
        &mut self,
        marker: u32,
        mut steps: usize,
        queued: usize,
        copies: &[(u32, EMoveType)],
    ) -> (u32, EMoveType) {
        let mut vertex = marker;
        let mut owner = queued;
        while steps > 0 {
            while let Some(prev) = owner.checked_sub(1)
                && (self.resolved[prev] == NO_VERTEX || self.resolved[prev] > vertex)
            {
                owner = prev;
            }
            let inserted = owner
                .checked_sub(1)
                .filter(|&prev| self.resolved[prev] == vertex)
                .map_or(0, |prev| self.splits(prev, copies));
            if steps <= inserted {
                return (vertex, self.kind(vertex, copies));
            }
            steps -= inserted + 1;
            vertex = self.previous(vertex);
        }
        self.erased.push(vertex);
        (vertex, self.kind(vertex, copies))
    }

    /// Queued blocks at or after the erased element resolve to the next one.
    fn shift(&mut self, queued: Range<usize>, erased: u32) {
        for block in queued {
            let vertex = self.resolved[block];
            if vertex != NO_VERTEX && vertex >= erased {
                self.resolved[block] = self.next(vertex);
            }
        }
    }

    fn previous(&self, mut vertex: u32) -> u32 {
        vertex -= 1;
        while self.erased.contains(&vertex) {
            vertex -= 1;
        }
        vertex
    }

    fn next(&self, mut vertex: u32) -> u32 {
        vertex += 1;
        while self.erased.contains(&vertex) {
            vertex += 1;
        }
        vertex
    }
}
