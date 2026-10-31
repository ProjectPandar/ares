// Copyright (c) Angus Johnson 2010-2025.
// BSL-1.0. Rust seed by MatterHackers.
// Derived from clipper2-rust 2f5e9d4249bc4d624773094ff6164335d9ff6ae7.
// See ../LICENSE-Boost-1.0 and ../SOURCE.md for pinned source authority.

use super::*;

/// Check if a value is odd
/// Direct port from clipper.engine.cpp line 61
#[inline]
pub fn is_odd(val: i32) -> bool {
    val & 1 != 0
}

/// Check if an active edge is "hot" (has an output record)
/// Direct port from clipper.engine.cpp line 67
#[inline]
pub fn is_hot_edge(e: &Active) -> bool {
    e.outrec.is_some()
}

/// Check if an active edge is part of an open path
/// Direct port from clipper.engine.cpp line 73
#[inline]
pub fn is_open_active(e: &Active, minima_list: &[LocalMinima]) -> bool {
    minima_list[e.local_min].is_open
}

/// Check if a vertex is an open end
/// Direct port from clipper.engine.cpp line 79
#[inline]
pub fn is_open_end_vertex(v: &Vertex) -> bool {
    (v.flags & (VertexFlags::OPEN_START | VertexFlags::OPEN_END)) != VertexFlags::EMPTY
}

/// Calculate slope dx for an edge defined by two points
/// Direct port from clipper.engine.cpp line 116
#[inline]
pub fn get_dx(pt1: Point64, pt2: Point64) -> f64 {
    let dy = (pt2.y - pt1.y) as f64;
    if dy != 0.0 {
        (pt2.x - pt1.x) as f64 / dy
    } else if pt2.x > pt1.x {
        -f64::MAX
    } else {
        f64::MAX
    }
}

/// Get the x coordinate of an active edge at a given y
/// Direct port from clipper.engine.cpp line 127
#[inline]
pub fn top_x(ae: &Active, current_y: i64) -> i64 {
    if current_y == ae.top.y || ae.top.x == ae.bot.x {
        ae.top.x
    } else if current_y == ae.bot.y {
        ae.bot.x
    } else {
        ae.bot.x
            + crate::geometry::clipper2::core::nearbyint_f64(ae.dx * (current_y - ae.bot.y) as f64)
                as i64
    }
}

/// Check if a horizontal edge is heading right
/// Direct port from clipper.engine.cpp line 143
#[inline]
pub fn is_heading_right_horz(e: &Active) -> bool {
    e.dx == -f64::MAX
}

/// Check if a horizontal edge is heading left
/// Direct port from clipper.engine.cpp line 149
#[inline]
pub fn is_heading_left_horz(e: &Active) -> bool {
    e.dx == f64::MAX
}

/// Set dx for an active edge from its bot/top points
/// Direct port from clipper.engine.cpp line 172
#[inline]
pub fn set_dx(e: &mut Active) {
    e.dx = get_dx(e.bot, e.top);
}

/// Get next vertex for an active edge (depends on winding direction)
/// Direct port from clipper.engine.cpp line 177
#[inline]
pub fn next_vertex(e: &Active, vertex_arena: &[Vertex]) -> usize {
    if e.wind_dx > 0 {
        vertex_arena[e.vertex_top].next
    } else {
        vertex_arena[e.vertex_top].prev
    }
}

/// Get the vertex two steps back (useful for alternate edge top)
/// Direct port from clipper.engine.cpp line 187
#[inline]
pub fn prev_prev_vertex(ae: &Active, vertex_arena: &[Vertex]) -> usize {
    if ae.wind_dx > 0 {
        let prev = vertex_arena[ae.vertex_top].prev;
        vertex_arena[prev].prev
    } else {
        let next = vertex_arena[ae.vertex_top].next;
        vertex_arena[next].next
    }
}

/// Check if a vertex is a local maximum
/// Direct port from clipper.engine.cpp line 215
#[inline]
pub fn is_maxima_vertex(v: &Vertex) -> bool {
    (v.flags & VertexFlags::LOCAL_MAX) != VertexFlags::EMPTY
}

/// Check if an active edge is at a local maximum
/// Direct port from clipper.engine.cpp line 221
#[inline]
pub fn is_maxima_active(e: &Active, vertex_arena: &[Vertex]) -> bool {
    is_maxima_vertex(&vertex_arena[e.vertex_top])
}

/// Calculate area of an OutPt circular list
/// Direct port from clipper.engine.cpp line 366
pub fn area_outpt(op_start: usize, outpt_arena: &[OutPt]) -> f64 {
    let mut result = 0.0;
    let mut op2 = op_start;
    loop {
        let prev_idx = outpt_arena[op2].prev;
        result += (outpt_arena[prev_idx].pt.y + outpt_arena[op2].pt.y) as f64
            * (outpt_arena[prev_idx].pt.x - outpt_arena[op2].pt.x) as f64;
        op2 = outpt_arena[op2].next;
        if op2 == op_start {
            break;
        }
    }
    result * 0.5
}

/// Calculate area of a triangle formed by three points
/// Direct port from clipper.engine.cpp line 380
#[inline]
pub fn area_triangle(pt1: Point64, pt2: Point64, pt3: Point64) -> f64 {
    (pt3.y + pt1.y) as f64 * (pt3.x - pt1.x) as f64
        + (pt1.y + pt2.y) as f64 * (pt1.x - pt2.x) as f64
        + (pt2.y + pt3.y) as f64 * (pt2.x - pt3.x) as f64
}

/// Sort comparator for IntersectNodes
/// Direct port from clipper.engine.cpp line 322
pub fn intersect_list_sort(a: &IntersectNode, b: &IntersectNode) -> std::cmp::Ordering {
    if a.pt.y == b.pt.y {
        a.pt.x.cmp(&b.pt.x)
    } else {
        b.pt.y.cmp(&a.pt.y)
    }
}

/// Get the maxima pair for an active edge
/// Direct port from clipper.engine.cpp line 254
pub fn get_maxima_pair(e: &Active, active_arena: &[Active]) -> Option<usize> {
    let mut e2_idx = e.next_in_ael;
    while let Some(idx) = e2_idx {
        if active_arena[idx].vertex_top == e.vertex_top {
            return Some(idx);
        }
        e2_idx = active_arena[idx].next_in_ael;
    }
    None
}

/// Get current Y maxima vertex for open paths
/// Direct port from clipper.engine.cpp line 226
pub fn get_curr_y_maxima_vertex_open(e: &Active, vertex_arena: &[Vertex]) -> Option<usize> {
    let mut result = e.vertex_top;
    if e.wind_dx > 0 {
        while vertex_arena[vertex_arena[result].next].pt.y == vertex_arena[result].pt.y
            && (vertex_arena[result].flags & (VertexFlags::OPEN_END | VertexFlags::LOCAL_MAX))
                == VertexFlags::EMPTY
        {
            result = vertex_arena[result].next;
        }
    } else {
        while vertex_arena[vertex_arena[result].prev].pt.y == vertex_arena[result].pt.y
            && (vertex_arena[result].flags & (VertexFlags::OPEN_END | VertexFlags::LOCAL_MAX))
                == VertexFlags::EMPTY
        {
            result = vertex_arena[result].prev;
        }
    }
    if is_maxima_vertex(&vertex_arena[result]) {
        Some(result)
    } else {
        None
    }
}

/// Get current Y maxima vertex for closed paths
/// Direct port from clipper.engine.cpp line 243
pub fn get_curr_y_maxima_vertex(e: &Active, vertex_arena: &[Vertex]) -> Option<usize> {
    let mut result = e.vertex_top;
    if e.wind_dx > 0 {
        while vertex_arena[vertex_arena[result].next].pt.y == vertex_arena[result].pt.y {
            result = vertex_arena[result].next;
        }
    } else {
        while vertex_arena[vertex_arena[result].prev].pt.y == vertex_arena[result].pt.y {
            result = vertex_arena[result].prev;
        }
    }
    if is_maxima_vertex(&vertex_arena[result]) {
        Some(result)
    } else {
        None
    }
}
