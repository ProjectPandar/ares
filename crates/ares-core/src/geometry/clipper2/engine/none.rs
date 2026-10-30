// Copyright (c) Angus Johnson 2010-2025.
// BSL-1.0. Rust seed by MatterHackers.
// Derived from clipper2-rust 2f5e9d4249bc4d624773094ff6164335d9ff6ae7.
// See ../LICENSE-Boost-1.0 and ../SOURCE.md for pinned source authority.

use super::*;

pub const NONE: usize = usize::MAX;

/// Type of clipping operation
/// Direct port from clipper.engine.h line 29
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(C)]
pub enum ClipType {
    #[default]
    NoClip,
    Intersection,
    Union,
    Difference,
    Xor,
}

/// Whether a path is Subject or Clip
/// Direct port from clipper.engine.h line 31
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum PathType {
    Subject,
    Clip,
}

/// How an edge joins with another (for horizontal processing)
/// Direct port from clipper.engine.h line 32
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(C)]
pub enum JoinWith {
    #[default]
    NoJoin,
    Left,
    Right,
}

/// Vertex flags (bitflags)
/// Direct port from clipper.engine.h line 34
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct VertexFlags(pub(crate) u32);

impl VertexFlags {
    pub const EMPTY: VertexFlags = VertexFlags(0);
}

impl VertexFlags {
    pub const OPEN_START: VertexFlags = VertexFlags(1);
}

impl VertexFlags {
    pub const OPEN_END: VertexFlags = VertexFlags(2);
}

impl VertexFlags {
    pub const LOCAL_MAX: VertexFlags = VertexFlags(4);
}

impl VertexFlags {
    pub const LOCAL_MIN: VertexFlags = VertexFlags(8);
}

impl std::ops::BitAnd for VertexFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self {
        VertexFlags(self.0 & rhs.0)
    }
}

impl std::ops::BitOr for VertexFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        VertexFlags(self.0 | rhs.0)
    }
}

impl std::ops::BitAndAssign for VertexFlags {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl std::ops::BitOrAssign for VertexFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

/// Input polygon vertex (circular doubly-linked list via arena indices)
/// Direct port from clipper.engine.h line 48
#[derive(Debug, Clone)]
pub struct Vertex {
    pub pt: Point64,
    pub next: usize, // index into vertex arena
    pub prev: usize, // index into vertex arena
    pub flags: VertexFlags,
}

impl Vertex {
    pub fn new(pt: Point64) -> Self {
        Self {
            pt,
            next: NONE,
            prev: NONE,
            flags: VertexFlags::EMPTY,
        }
    }
}

/// Output point in the clipping result (circular doubly-linked list)
/// Direct port from clipper.engine.h line 55
#[derive(Debug, Clone)]
pub struct OutPt {
    pub pt: Point64,
    pub next: usize,         // index into outpt arena
    pub prev: usize,         // index into outpt arena
    pub outrec: usize,       // index into outrec list
    pub horz: Option<usize>, // index into horz_seg_list
}

impl OutPt {
    pub fn new(pt: Point64, outrec_idx: usize) -> Self {
        Self {
            pt,
            next: NONE,
            prev: NONE,
            outrec: outrec_idx,
            horz: None,
        }
    }
}

/// Output polygon record
/// Direct port from clipper.engine.h line 79
#[derive(Debug, Clone)]
pub struct OutRec {
    pub idx: usize,
    pub owner: Option<usize>,           // index into outrec list
    pub front_edge: Option<usize>,      // index into active arena
    pub back_edge: Option<usize>,       // index into active arena
    pub pts: Option<usize>,             // index into outpt arena
    pub polypath: Option<usize>,        // index into polypath list
    pub splits: Option<Vec<usize>>,     // indices of split outrecs
    pub recursive_split: Option<usize>, // index into outrec list
    pub bounds: Rect64,
    pub path: Path64,
    pub is_open: bool,
}

impl OutRec {
    pub fn new(idx: usize) -> Self {
        Self {
            idx,
            owner: None,
            front_edge: None,
            back_edge: None,
            pts: None,
            polypath: None,
            splits: None,
            recursive_split: None,
            bounds: Rect64::new(0, 0, 0, 0),
            path: Path64::new(),
            is_open: false,
        }
    }
}

/// Active edge in the sweep line
/// Direct port from clipper.engine.h line 104
#[derive(Debug, Clone)]
pub struct Active {
    pub bot: Point64,
    pub top: Point64,
    pub curr_x: i64,
    pub dx: f64,
    pub wind_dx: i32, // 1 or -1 depending on winding direction
    pub wind_cnt: i32,
    pub wind_cnt2: i32,        // winding count of the opposite polytype
    pub outrec: Option<usize>, // index into outrec list
    // AEL: active edge list
    pub prev_in_ael: Option<usize>, // index into active arena
    pub next_in_ael: Option<usize>, // index into active arena
    // SEL: sorted edge list
    pub prev_in_sel: Option<usize>, // index into active arena
    pub next_in_sel: Option<usize>, // index into active arena
    pub jump: Option<usize>,        // index into active arena
    pub vertex_top: usize,          // index into vertex arena
    pub local_min: usize,           // index into local_minima list
    pub is_left_bound: bool,
    pub join_with: JoinWith,
}

impl Active {
    pub fn new() -> Self {
        Self {
            bot: Point64::new(0, 0),
            top: Point64::new(0, 0),
            curr_x: 0,
            dx: 0.0,
            wind_dx: 1,
            wind_cnt: 0,
            wind_cnt2: 0,
            outrec: None,
            prev_in_ael: None,
            next_in_ael: None,
            prev_in_sel: None,
            next_in_sel: None,
            jump: None,
            vertex_top: NONE,
            local_min: NONE,
            is_left_bound: false,
            join_with: JoinWith::NoJoin,
        }
    }
}

impl Default for Active {
    fn default() -> Self {
        Self::new()
    }
}

/// Local minimum vertex where ascending and descending bounds meet
/// Direct port from clipper.engine.h line 131
#[derive(Debug, Clone)]
pub struct LocalMinima {
    pub vertex: usize, // index into vertex arena
    pub polytype: PathType,
    pub is_open: bool,
}

impl LocalMinima {
    pub fn new(vertex: usize, polytype: PathType, is_open: bool) -> Self {
        Self {
            vertex,
            polytype,
            is_open,
        }
    }
}

/// Intersection between two active edges
/// Direct port from clipper.engine.h line 139
#[derive(Debug, Clone)]
pub struct IntersectNode {
    pub pt: Point64,
    pub edge1: usize, // index into active arena
    pub edge2: usize, // index into active arena
}

impl IntersectNode {
    pub fn new() -> Self {
        Self {
            pt: Point64::new(0, 0),
            edge1: NONE,
            edge2: NONE,
        }
    }
}

impl IntersectNode {
    pub fn with_edges(edge1: usize, edge2: usize, pt: Point64) -> Self {
        Self { pt, edge1, edge2 }
    }
}

impl Default for IntersectNode {
    fn default() -> Self {
        Self::new()
    }
}

/// Horizontal segment for horizontal edge processing
/// Direct port from clipper.engine.h line 148
#[derive(Debug, Clone)]
pub struct HorzSegment {
    pub left_op: Option<usize>,  // index into outpt arena
    pub right_op: Option<usize>, // index into outpt arena
    pub left_to_right: bool,
}

impl HorzSegment {
    pub fn new() -> Self {
        Self {
            left_op: None,
            right_op: None,
            left_to_right: true,
        }
    }
}

impl HorzSegment {
    pub fn with_op(op_idx: usize) -> Self {
        Self {
            left_op: Some(op_idx),
            right_op: None,
            left_to_right: true,
        }
    }
}

impl Default for HorzSegment {
    fn default() -> Self {
        Self::new()
    }
}
