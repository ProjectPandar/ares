//! Incremental OptionsZ correction and actual-speed insertion.
//! Public marker identity uses the stable correction; the compact
//! cache projection preserves official export-cache membership.

mod cache;
use cache::CacheMove;
use std::collections::VecDeque;

use super::super::motion::Trapezoid;
use super::super::moves::actual_speed::{Pass, is_timed};
use crate::gcode_processor::MoveVertex;

#[derive(Clone, Copy)]
pub(in crate::project_slice::gcode_emit::processor) struct MoveIds {
    public: usize,
    cache: usize,
}

struct Queued {
    ids: Option<MoveIds>,
    shifted: bool,
}

pub(in crate::project_slice::gcode_emit::processor) struct Corrector {
    cache_moves: Vec<CacheMove>,
    queue: VecDeque<Queued>,
    pending_public: Option<usize>,
    pending_cache: Option<usize>,
    pass: Pass,
    cache_pass: cache::Pass,
}

impl Corrector {
    pub(in crate::project_slice::gcode_emit::processor) fn new() -> Self {
        Self {
            cache_moves: vec![CacheMove::from(&MoveVertex::default())],
            queue: VecDeque::new(),
            pending_public: None,
            pending_cache: None,
            pass: Pass::default(),
            cache_pass: cache::Pass::default(),
        }
    }

    pub(in crate::project_slice::gcode_emit::processor) fn next_vertex(
        &self,
        public: usize,
    ) -> MoveIds {
        MoveIds {
            public,
            cache: self.cache_moves.len(),
        }
    }

    pub(in crate::project_slice::gcode_emit::processor) fn block(&mut self, ids: MoveIds) {
        self.queue.push_back(Queued {
            ids: Some(ids),
            shifted: false,
        });
    }

    pub(in crate::project_slice::gcode_emit::processor) fn virtual_block(&mut self) {
        self.queue.push_back(Queued {
            ids: None,
            shifted: false,
        });
    }

    pub(in crate::project_slice::gcode_emit::processor) fn vertex(&mut self, vertex: &MoveVertex) {
        self.cache_moves.push(CacheMove::from(vertex));
    }

    pub(in crate::project_slice::gcode_emit::processor) fn set(&mut self, public: usize) {
        self.pending_public = Some(public);
        self.pending_cache = Some(self.cache_moves.len() - 1);
    }

    /// `OptionsZCorrector::update`: relocate the pending marker and renumber
    /// pending blocks after the erased vertex.
    pub(in crate::project_slice::gcode_emit::processor) fn update(
        &mut self,
        moves: &mut crate::gcode_processor::ChunkedMoves,
        height: f32,
    ) {
        let public = self.pending_public.take().unwrap();
        let cache = self.pending_cache.take().unwrap();
        let mut copy = moves[public];
        copy.position = moves.last().unwrap().position;
        copy.height = height;
        moves.push(copy);
        moves.remove(public);
        let mut copy = self.cache_moves[cache];
        copy.position = self.cache_moves.last().unwrap().position;
        self.cache_moves.push(copy);
        self.cache_moves.remove(cache);
        for queued in &mut self.queue {
            let Some(ids) = &mut queued.ids else {
                continue;
            };
            if ids.public > public {
                ids.public -= 1;
            }
            if ids.cache > cache {
                ids.cache -= 1;
                queued.shifted = true;
            }
        }
    }

    /// `calculate_time` inserts only this flush's actual-speed vertices in
    /// the existing buffers (`GCodeProcessor.cpp:8103-8192`).
    pub(in crate::project_slice::gcode_emit::processor) fn flush(
        &mut self,
        moves: &mut crate::gcode_processor::ChunkedMoves,
        batch: &[(f64, Trapezoid)],
    ) -> Vec<Option<bool>> {
        let mut eligible = Vec::with_capacity(batch.len());
        for &(time, trapezoid) in batch {
            let queued = self.queue.pop_front().unwrap();
            if let Some(ids) = queued.ids {
                eligible.push(
                    queued
                        .shifted
                        .then(|| is_timed(self.cache_moves[ids.cache].kind)),
                );
                self.cache_pass
                    .block(&self.cache_moves, ids.cache, &trapezoid);
                self.pass.block(moves, ids.public, time, &trapezoid);
            } else {
                moves[0].time = time as f32;
                eligible.push(None);
            }
        }
        let (public_map, public_inserted) = self.pass.insert(moves);
        let (cache_map, cache_inserted) = self.cache_pass.insert(&mut self.cache_moves);
        for queued in &mut self.queue {
            let Some(ids) = &mut queued.ids else {
                continue;
            };
            ids.public = remap(&public_map, public_inserted, ids.public);
            ids.cache = remap(&cache_map, cache_inserted, ids.cache);
        }
        if let Some(index) = &mut self.pending_public {
            let upper = public_map.partition_point(|&(old, _)| old <= *index);
            if let Some(&(old, new)) = public_map[..upper].last() {
                *index += new - old;
            }
        }
        eligible
    }
}

fn remap(map: &[(usize, usize)], inserted: usize, id: usize) -> usize {
    match map.binary_search_by_key(&id, |&(old, _)| old) {
        Ok(found) => map[found].1,
        Err(_) => id + inserted,
    }
}
