// Copyright (c) Angus Johnson 2010-2025.
// BSL-1.0. Rust seed by MatterHackers.
// Derived from clipper2-rust 2f5e9d4249bc4d624773094ff6164335d9ff6ae7.
// See ../LICENSE-Boost-1.0 and ../SOURCE.md for pinned source authority.

use super::*;

/// Horizontal join between two output points
/// Direct port from clipper.engine.h line 156
#[derive(Debug, Clone)]
pub struct HorzJoin {
    pub op1: Option<usize>, // index into outpt arena
    pub op2: Option<usize>, // index into outpt arena
}

impl HorzJoin {
    pub fn new() -> Self {
        Self {
            op1: None,
            op2: None,
        }
    }
}

impl HorzJoin {
    pub fn with_ops(ltr: usize, rtl: usize) -> Self {
        Self {
            op1: Some(ltr),
            op2: Some(rtl),
        }
    }
}

impl Default for HorzJoin {
    fn default() -> Self {
        Self::new()
    }
}

/// Callback invoked at each intersection so callers can assign the z value:
/// (e1bot, e1top, e2bot, e2top, intersect_pt)
/// Direct port from clipper.engine.h line 163 (ZCallback64 std::function)
pub trait ZCallback64: FnMut(&Point64, &Point64, &Point64, &Point64, &mut Point64) {}

impl<T: FnMut(&Point64, &Point64, &Point64, &Point64, &mut Point64)> ZCallback64 for T {}

/// Main clipping engine. Manages all arenas and the sweep-line algorithm state.
/// Direct port from clipper.engine.h line 192
pub struct ClipperBase {
    // Configuration
    pub cliptype: ClipType,
    pub fillrule: FillRule,
    pub preserve_collinear: bool,
    pub reverse_solution: bool,
    pub error_code: i32,
    pub has_open_paths: bool,
    pub succeeded: bool,
    pub(crate) using_polytree: bool,

    // Sweep-line state
    pub(crate) bot_y: i64,
    pub(crate) minima_list_sorted: bool,

    // Arenas - all nodes are stored here, referenced by indices
    pub vertex_arena: Vec<Vertex>,
    pub active_arena: Vec<Active>,
    pub outpt_arena: Vec<OutPt>,

    // Lists referencing arena indices
    pub outrec_list: Vec<OutRec>,
    pub minima_list: Vec<LocalMinima>,
    pub(crate) current_locmin_idx: usize,

    // Active edge list head
    pub actives: Option<usize>, // index into active_arena
    pub sel: Option<usize>,     // index into active_arena

    // Scanline priority queue (min-heap using Reverse)
    pub(crate) scanline_list: BinaryHeap<i64>,

    // Intersection and horizontal processing
    pub intersect_nodes: Vec<IntersectNode>,
    pub horz_seg_list: Vec<HorzSegment>,
    pub horz_join_list: Vec<HorzJoin>,

    // 'static boxed FnMut mirrors C++ std::function, which has no borrow
    // semantics either; callers capture external state via move closures.
    pub(crate) z_callback: Option<Box<dyn ZCallback64 + 'static>>,

    /// z assigned to intersection points that match no input vertex
    /// Direct port from clipper.engine.h line 287 (DefaultZ)
    pub default_z: i64,
}

impl ClipperBase {
    pub fn new() -> Self {
        Self {
            cliptype: ClipType::NoClip,
            fillrule: FillRule::EvenOdd,
            preserve_collinear: true,
            reverse_solution: false,
            error_code: 0,
            has_open_paths: false,
            succeeded: true,
            using_polytree: false,
            bot_y: 0,
            minima_list_sorted: false,
            vertex_arena: Vec::new(),
            active_arena: Vec::new(),
            outpt_arena: Vec::new(),
            outrec_list: Vec::new(),
            minima_list: Vec::new(),
            current_locmin_idx: 0,
            actives: None,
            sel: None,
            scanline_list: BinaryHeap::new(),
            intersect_nodes: Vec::new(),
            horz_seg_list: Vec::new(),
            horz_join_list: Vec::new(),

            z_callback: None,

            default_z: 0,
        }
    }
}

impl ClipperBase {
    /// Clear all data
    /// Direct port from clipper.engine.h line 284
    pub fn clear(&mut self) {
        self.vertex_arena.clear();
        self.active_arena.clear();
        self.outpt_arena.clear();
        self.outrec_list.clear();
        self.minima_list.clear();
        self.current_locmin_idx = 0;
        self.actives = None;
        self.sel = None;
        self.scanline_list.clear();
        self.intersect_nodes.clear();
        self.horz_seg_list.clear();
        self.horz_join_list.clear();
        self.minima_list_sorted = false;
        self.has_open_paths = false;
        self.succeeded = true;
    }
}

impl ClipperBase {
    /// Insert a scanline y-value
    /// Direct port from clipper.engine.cpp InsertScanline
    #[inline]
    pub fn insert_scanline(&mut self, y: i64) {
        self.scanline_list.push(y);
    }
}

impl ClipperBase {
    /// Pop the next scanline y-value
    /// Direct port from clipper.engine.cpp PopScanline
    #[inline]
    pub fn pop_scanline(&mut self) -> Option<i64> {
        if self.scanline_list.is_empty() {
            return None;
        }
        let y = self.scanline_list.pop().unwrap();
        // Skip duplicates
        while !self.scanline_list.is_empty() && *self.scanline_list.peek().unwrap() == y {
            self.scanline_list.pop();
        }
        Some(y)
    }
}

impl ClipperBase {
    /// Pop a local minimum at a given y-value
    /// Direct port from clipper.engine.cpp PopLocalMinima
    #[inline]
    pub fn pop_local_minima(&mut self, y: i64) -> Option<usize> {
        if self.current_locmin_idx < self.minima_list.len() {
            let vertex_idx = self.minima_list[self.current_locmin_idx].vertex;
            if self.vertex_arena[vertex_idx].pt.y == y {
                let idx = self.current_locmin_idx;
                self.current_locmin_idx += 1;
                return Some(idx);
            }
        }
        None
    }
}

impl ClipperBase {
    /// Create a new OutRec
    /// Direct port from clipper.engine.cpp NewOutRec
    pub fn new_out_rec(&mut self) -> usize {
        let idx = self.outrec_list.len();
        self.outrec_list.push(OutRec::new(idx));
        idx
    }
}

impl ClipperBase {
    /// Create a new OutPt in the arena
    pub fn new_out_pt(&mut self, pt: Point64, outrec_idx: usize) -> usize {
        let idx = self.outpt_arena.len();
        let mut op = OutPt::new(pt, outrec_idx);
        op.next = idx;
        op.prev = idx;
        self.outpt_arena.push(op);
        idx
    }
}

impl ClipperBase {
    /// Create a new Active in the arena
    pub fn new_active(&mut self) -> usize {
        let idx = self.active_arena.len();
        self.active_arena.push(Active::new());
        idx
    }
}

impl ClipperBase {
    /// Add a local minimum
    /// Direct port from clipper.engine.cpp AddLocMin
    pub fn add_loc_min(&mut self, vert_idx: usize, polytype: PathType, is_open: bool) {
        // clipper.engine.cpp:925–932: register each vertex only once.
        if self.vertex_arena[vert_idx].flags & VertexFlags::LOCAL_MIN != VertexFlags::EMPTY {
            return;
        }
        self.vertex_arena[vert_idx].flags |= VertexFlags::LOCAL_MIN;
        self.minima_list
            .push(LocalMinima::new(vert_idx, polytype, is_open));
    }
}
