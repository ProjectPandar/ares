//! `OptionsZCorrector` (`GCodeProcessor.hpp:678-711`) replayed on the move
//! list `GCodeProcessor::calculate_time` grows (`GCodeProcessor.cpp:5951-6008`).
//! A block keeps the `move_id` it got at creation (`:4004`, `:5617`); only
//! the actual-speed insertion renumbers it (`:6003-6008`). `set()` stores
//! `moves.size() - 1` and nothing renumbers it, so a flush between `set()`
//! and `update()` leaves the index pointing at an earlier element.
//! `update()` copies and erases whatever sits there, and every queued block
//! at or after it resolves to the next element: its time (`:472`), its
//! actual-speed vertices (`:480-573`) and its `g1_times_cache` entry
//! (`:484-488`, `:574`) follow that element.

use crate::gcode_processor::MoveVertex;
use crate::project_slice::gcode_emit::processor::motion::Trapezoid;
use crate::project_slice::gcode_emit::processor::moves::actual_speed::{Pass, is_timed};

/// The block of a `VG1`, which stores no vertex.
const NO_VERTEX: u32 = u32::MAX;

/// Blocks queued, vertices stored and planner events pushed so far.
#[derive(Clone, Copy)]
struct Clock {
    blocks: usize,
    vertices: usize,
    events: usize,
}

enum Call {
    Set,
    /// `update(m_height)` (`GCodeProcessor.cpp:3929`).
    Update(f32),
}

/// The stored vertex count, the first vertex each block stored and the
/// corrector calls, in creation order.
pub(in crate::project_slice::gcode_emit::processor) struct Log {
    vertices: usize,
    blocks: Vec<u32>,
    events: usize,
    calls: Vec<(Clock, Call)>,
}

/// The planned blocks. `pushed[i]` is the processed-block count once block
/// `i` is queued and `flush_ends[e]` the count after planner event `e`.
pub(in crate::project_slice::gcode_emit::processor) struct Scheduled<'a> {
    pub(in crate::project_slice::gcode_emit::processor) times: &'a [f64],
    pub(in crate::project_slice::gcode_emit::processor) trapezoids: &'a [Trapezoid],
    pub(in crate::project_slice::gcode_emit::processor) pushed: &'a [usize],
    pub(in crate::project_slice::gcode_emit::processor) flush_ends: &'a [usize],
}

impl Scheduled<'_> {
    fn processed(&self, clock: Clock) -> usize {
        let refreshed = clock.blocks.checked_sub(1).map_or(0, |i| self.pushed[i]);
        let flushed = clock
            .events
            .checked_sub(1)
            .map_or(0, |e| self.flush_ends[e]);
        refreshed.max(flushed)
    }
}

impl Log {
    /// The origin vertex `GCodeProcessorResult::moves` starts with.
    pub(in crate::project_slice::gcode_emit::processor) fn new() -> Self {
        Self {
            vertices: 1,
            blocks: Vec::new(),
            events: 0,
            calls: Vec::new(),
        }
    }

    /// The planner events pushed before the current line.
    pub(in crate::project_slice::gcode_emit::processor) fn tick(&mut self, events: usize) {
        self.events = events;
    }

    pub(in crate::project_slice::gcode_emit::processor) fn next_vertex(&self) -> u32 {
        self.vertices as u32
    }

    /// Queues the block of a command whose vertices start at `first`.
    pub(in crate::project_slice::gcode_emit::processor) fn block(&mut self, first: u32) {
        self.blocks.push(first);
    }

    pub(in crate::project_slice::gcode_emit::processor) fn virtual_block(&mut self) {
        self.blocks.push(NO_VERTEX);
    }

    /// A `store_move_vertex` (`:5630`).
    pub(in crate::project_slice::gcode_emit::processor) fn vertex(&mut self) {
        self.vertices += 1;
    }

    fn call(&mut self, call: Call) {
        let clock = Clock {
            blocks: self.blocks.len(),
            vertices: self.vertices,
            events: self.events,
        };
        self.calls.push((clock, call));
    }

    /// `OptionsZCorrector::set` (`GCodeProcessor.hpp:688-691`).
    pub(in crate::project_slice::gcode_emit::processor) fn set(&mut self) {
        self.call(Call::Set);
    }

    /// `OptionsZCorrector::update` (`GCodeProcessor.hpp:693-705`) after a
    /// `set()`.
    pub(in crate::project_slice::gcode_emit::processor) fn update(&mut self, height: f32) {
        self.call(Call::Update(height));
    }

    /// Rebuilds `GCodeProcessorResult::moves` from the `recorded` vertices
    /// (origin first) with the flushes interleaved where Native ran them, and
    /// corrects `cache_eligible` of the blocks an `update()` shifted.
    pub(in crate::project_slice::gcode_emit::processor) fn replay(
        &self,
        recorded: Vec<MoveVertex>,
        scheduled: &Scheduled<'_>,
        cache_eligible: &mut [bool],
    ) -> Vec<MoveVertex> {
        debug_assert_eq!(self.blocks.len(), cache_eligible.len());
        let mut replay = Replay {
            scheduled,
            cache_eligible,
            moves: Vec::with_capacity(recorded.len() * 2),
            recorded: recorded.into_iter(),
            stored: 0,
            queue: Vec::with_capacity(self.blocks.len()),
            done: 0,
            pass: Pass::default(),
        };
        let mut stale = None;
        for &(clock, ref call) in &self.calls {
            replay.advance(clock.vertices, &self.blocks[..clock.blocks]);
            replay.flush(scheduled.processed(clock));
            match (call, stale.take()) {
                // `m_move_id = moves.size() - 1`.
                (Call::Set, _) => stale = Some(replay.moves.len() - 1),
                (&Call::Update(height), Some(index)) => replay.update(index, height),
                (Call::Update(_), None) => {}
            }
        }
        replay.advance(self.vertices, &self.blocks);
        replay.flush(self.blocks.len());
        replay.moves
    }
}

#[derive(Clone, Copy)]
struct Queued {
    /// `TimeBlock::move_id`; `None` for a `VG1`, which keeps `0`.
    move_id: Option<usize>,
    /// An `update()` erased an element at or before `move_id`.
    shifted: bool,
}

/// Native's move list and block queue while the recorded vertices replay.
struct Replay<'a, 'b> {
    scheduled: &'a Scheduled<'b>,
    cache_eligible: &'a mut [bool],
    moves: Vec<MoveVertex>,
    recorded: std::vec::IntoIter<MoveVertex>,
    stored: usize,
    queue: Vec<Queued>,
    done: usize,
    pass: Pass,
}

impl Replay<'_, '_> {
    /// Stores the recorded vertices up to `vertices` and queues `blocks`.
    /// Nothing flushes in between, so a block's `move_id` is its first
    /// vertex offset by the elements inserted so far.
    fn advance(&mut self, vertices: usize, blocks: &[u32]) {
        let offset = self.moves.len() - self.stored;
        self.moves
            .extend(self.recorded.by_ref().take(vertices - self.stored));
        self.stored = vertices;
        self.queue
            .extend(blocks[self.queue.len()..].iter().map(|&first| Queued {
                move_id: (first != NO_VERTEX).then_some(first as usize + offset),
                shifted: false,
            }));
    }

    /// `GCodeProcessor::calculate_time` (`:5951-6008`) up to `target`
    /// processed blocks.
    fn flush(&mut self, target: usize) {
        if target <= self.done {
            return;
        }
        for block in self.done..target {
            let Queued { move_id, shifted } = self.queue[block];
            let time = self.scheduled.times[block];
            let Some(id) = move_id else {
                self.moves[0].time = time as f32;
                continue;
            };
            if shifted {
                self.cache_eligible[block] = is_timed(self.moves[id].move_type);
            }
            let trapezoid = &self.scheduled.trapezoids[block];
            self.pass.block(&mut self.moves, id, time, trapezoid);
        }
        self.done = target;
        let (map, inserted) = self.pass.insert(&mut self.moves);
        for id in self.queue[target..]
            .iter_mut()
            .filter_map(|q| q.move_id.as_mut())
        {
            *id = match map.binary_search_by_key(id, |&(old, _)| old) {
                Ok(found) => map[found].1,
                Err(_) => *id + inserted,
            };
        }
    }

    /// `OptionsZCorrector::update` (`GCodeProcessor.hpp:693-705`) on the
    /// stale `index`; queued blocks keep their `move_id`.
    fn update(&mut self, index: usize, height: f32) {
        let mut copy = self.moves[index];
        copy.position = self.moves[self.moves.len() - 1].position;
        copy.height = height;
        self.moves.push(copy);
        self.moves.remove(index);
        for queued in &mut self.queue[self.done..] {
            if queued.move_id.is_some_and(|id| id >= index) {
                queued.shifted = true;
            }
        }
    }
}
