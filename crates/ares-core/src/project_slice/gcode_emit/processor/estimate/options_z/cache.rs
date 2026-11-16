//! Official marker identity for export-cache membership, without copying public moves.

use super::super::super::motion::Trapezoid;
use super::super::super::moves::{actual_speed, insert};
use crate::gcode_processor::{EMoveType, MoveVertex};

#[derive(Clone, Copy)]
pub(super) struct CacheMove {
    pub(super) position: [f32; 3],
    pub(super) kind: EMoveType,
}

impl From<&MoveVertex> for CacheMove {
    fn from(vertex: &MoveVertex) -> Self {
        Self {
            position: vertex.position,
            kind: vertex.move_type,
        }
    }
}

#[derive(Default)]
pub(super) struct Pass {
    groups: Vec<(usize, usize)>,
    splits: Vec<CacheMove>,
}

impl Pass {
    pub(super) fn block(&mut self, moves: &[CacheMove], id: usize, trapezoid: &Trapezoid) {
        let current = moves[id];
        if !actual_speed::is_timed(current.kind) {
            return;
        }
        let before = self.splits.len();
        self.splits
            .extend(actual_speed::split_fractions(trapezoid).filter_map(|t| {
                actual_speed::split_position(moves[id - 1].position, current.position, t).map(
                    |position| CacheMove {
                        position,
                        ..current
                    },
                )
            }));
        self.groups.push((id, self.splits.len() - before));
    }

    pub(super) fn insert(&mut self, moves: &mut Vec<CacheMove>) -> (Vec<(usize, usize)>, usize) {
        insert::insert(moves, self.groups.drain(..), &mut self.splits)
    }
}
