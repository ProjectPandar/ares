// Copyright (c) Angus Johnson 2010-2025.
// BSL-1.0. Rust seed by MatterHackers.
// Derived from clipper2-rust 2f5e9d4249bc4d624773094ff6164335d9ff6ae7.
// See ../LICENSE-Boost-1.0 and ../SOURCE.md for pinned source authority.

use super::*;

/// Check if the AEL ordering is valid between two active edges
/// Direct port from clipper.engine.cpp IsValidAelOrder (line 1119)
pub fn is_valid_ael_order(
    resident: &Active,
    newcomer: &Active,
    vertex_arena: &[Vertex],
    minima_list: &[LocalMinima],
) -> bool {
    if newcomer.curr_x != resident.curr_x {
        return newcomer.curr_x > resident.curr_x;
    }

    // get the turning direction: resident.top, newcomer.bot, newcomer.top
    let i = cross_product_sign(
        vertex_arena[resident.vertex_top].pt,
        newcomer.bot,
        vertex_arena[newcomer.vertex_top].pt,
    );
    if i != 0 {
        return i < 0;
    }

    // edges must be collinear to get here
    // for starting open paths, place them according to
    // the direction they're about to turn
    if !is_maxima_active(resident, vertex_arena)
        && vertex_arena[resident.vertex_top].pt.y > vertex_arena[newcomer.vertex_top].pt.y
    {
        let nv = next_vertex(resident, vertex_arena);
        return cross_product_sign(
            newcomer.bot,
            vertex_arena[resident.vertex_top].pt,
            vertex_arena[nv].pt,
        ) <= 0;
    } else if !is_maxima_active(newcomer, vertex_arena)
        && vertex_arena[newcomer.vertex_top].pt.y > vertex_arena[resident.vertex_top].pt.y
    {
        let nv = next_vertex(newcomer, vertex_arena);
        return cross_product_sign(
            newcomer.bot,
            vertex_arena[newcomer.vertex_top].pt,
            vertex_arena[nv].pt,
        ) >= 0;
    }

    let y = newcomer.bot.y;
    let newcomer_is_left = newcomer.is_left_bound;

    if resident.bot.y != y || vertex_arena[minima_list[resident.local_min].vertex].pt.y != y {
        newcomer.is_left_bound
    } else if resident.is_left_bound != newcomer_is_left {
        newcomer_is_left
    } else if is_collinear(
        vertex_arena[prev_prev_vertex(resident, vertex_arena)].pt,
        resident.bot,
        vertex_arena[resident.vertex_top].pt,
    ) {
        true
    } else {
        // compare turning direction of the alternate bound
        let ppv_r = prev_prev_vertex(resident, vertex_arena);
        let ppv_n = prev_prev_vertex(newcomer, vertex_arena);
        (cross_product_sign(vertex_arena[ppv_r].pt, newcomer.bot, vertex_arena[ppv_n].pt) > 0)
            == newcomer_is_left
    }
}

/// Build a Path64 from an OutRec
/// Direct port from clipper.engine.cpp GetCleanPath equivalent
pub fn get_clean_path(op_start: usize, outpt_arena: &[OutPt]) -> Path64 {
    let mut result = Path64::new();
    let mut op = op_start;
    loop {
        let prev = outpt_arena[op].prev;
        let next = outpt_arena[op].next;
        if outpt_arena[op].pt == outpt_arena[prev].pt
            || is_collinear(
                outpt_arena[prev].pt,
                outpt_arena[op].pt,
                outpt_arena[next].pt,
            )
        {
            op = next;
            if op == op_start {
                break;
            }
            continue;
        }
        result.push(outpt_arena[op].pt);
        op = next;
        if op == op_start {
            break;
        }
    }
    result
}

/// Get the real (non-disposed) OutRec by following owner chain
/// Direct port from clipper.engine.cpp GetRealOutRec (line 412)
pub fn get_real_outrec(outrec_list: &[OutRec], idx: usize) -> Option<usize> {
    let mut i = Some(idx);
    while let Some(cur) = i {
        if outrec_list[cur].pts.is_some() {
            return Some(cur);
        }
        i = outrec_list[cur].owner;
    }
    None
}

/// Check if an owner is valid (not circular)
/// Direct port from clipper.engine.cpp IsValidOwner (line 418)
pub fn is_valid_owner(outrec_list: &[OutRec], outrec_idx: usize, test_owner_idx: usize) -> bool {
    let mut tmp = Some(test_owner_idx);
    while let Some(idx) = tmp {
        if idx == outrec_idx {
            return false;
        }
        tmp = outrec_list[idx].owner;
    }
    true
}

/// Check if two points are really close (within 2 units)
/// Direct port from clipper.engine.cpp PtsReallyClose (line 436)
#[inline]
pub fn pts_really_close(pt1: Point64, pt2: Point64) -> bool {
    (pt1.x - pt2.x).abs() < 2 && (pt1.y - pt2.y).abs() < 2
}

/// Check if an OutPt triangle is very small
/// Direct port from clipper.engine.cpp IsVerySmallTriangle (line 441)
pub fn is_very_small_triangle(op_idx: usize, outpt_arena: &[OutPt]) -> bool {
    let next = outpt_arena[op_idx].next;
    let prev = outpt_arena[op_idx].prev;
    if outpt_arena[next].next != prev {
        return false;
    }
    pts_really_close(outpt_arena[prev].pt, outpt_arena[next].pt)
        || pts_really_close(outpt_arena[op_idx].pt, outpt_arena[next].pt)
        || pts_really_close(outpt_arena[op_idx].pt, outpt_arena[prev].pt)
}

/// Check if a closed path is valid
/// Direct port from clipper.engine.cpp IsValidClosedPath (line 449)
pub fn is_valid_closed_path(op: Option<usize>, outpt_arena: &[OutPt]) -> bool {
    match op {
        None => false,
        Some(idx) => {
            let next = outpt_arena[idx].next;
            if next == idx {
                return false;
            }
            let prev = outpt_arena[idx].prev;
            if next == prev {
                return false;
            }
            !is_very_small_triangle(idx, outpt_arena)
        }
    }
}

/// Check if the hot edge is ascending (front edge of its outrec)
/// Direct port from clipper.engine.cpp OutrecIsAscending (line 455)
#[inline]
pub fn outrec_is_ascending(
    hot_edge_idx: usize,
    outrec_list: &[OutRec],
    active_arena: &[Active],
) -> bool {
    if let Some(or_idx) = active_arena[hot_edge_idx].outrec {
        outrec_list[or_idx].front_edge == Some(hot_edge_idx)
    } else {
        false
    }
}

/// Swap front and back sides of an OutRec
/// Direct port from clipper.engine.cpp SwapFrontBackSides (line 460)
pub fn swap_front_back_sides(outrec_idx: usize, outrec_list: &mut [OutRec], outpt_arena: &[OutPt]) {
    std::mem::swap(
        &mut outrec_list[outrec_idx].front_edge,
        &mut outrec_list[outrec_idx].back_edge,
    );
    if let Some(pts) = outrec_list[outrec_idx].pts {
        outrec_list[outrec_idx].pts = Some(outpt_arena[pts].next);
    }
}

/// Check if intersect node edges are adjacent in AEL
/// Direct port from clipper.engine.cpp EdgesAdjacentInAEL (line 468)
#[inline]
pub fn edges_adjacent_in_ael(inode: &IntersectNode, active_arena: &[Active]) -> bool {
    active_arena[inode.edge1].next_in_ael == Some(inode.edge2)
        || active_arena[inode.edge1].prev_in_ael == Some(inode.edge2)
}

/// Check if an edge is joined
/// Direct port from clipper.engine.cpp IsJoined (line 473)
#[inline]
pub fn is_joined(e: &Active) -> bool {
    e.join_with != JoinWith::NoJoin
}

/// Set the owner of an OutRec
/// Direct port from clipper.engine.cpp SetOwner (line 478)
pub fn set_owner(outrec_list: &mut [OutRec], outrec_idx: usize, new_owner_idx: usize) {
    // precondition: new_owner_idx is valid
    // Direct port from C++: new_owner->owner = GetRealOutRec(new_owner->owner);
    // When owner is None, GetRealOutRec returns None (C++ handles nullptr correctly)
    outrec_list[new_owner_idx].owner = match outrec_list[new_owner_idx].owner {
        Some(owner_idx) => get_real_outrec(outrec_list, owner_idx),
        None => None,
    };
    let mut tmp = Some(new_owner_idx);
    while let Some(t) = tmp {
        if t == outrec_idx {
            outrec_list[new_owner_idx].owner = outrec_list[outrec_idx].owner;
            break;
        }
        tmp = outrec_list[t].owner;
    }
    outrec_list[outrec_idx].owner = Some(new_owner_idx);
}

/// Point in polygon test for OutPt-based polygons
/// Direct port from clipper.engine.cpp PointInOpPolygon (line 488)
pub fn point_in_op_polygon(
    pt: Point64,
    op_start: usize,
    outpt_arena: &[OutPt],
) -> PointInPolygonResult {
    let next = outpt_arena[op_start].next;
    if next == op_start || outpt_arena[op_start].prev == next {
        return PointInPolygonResult::IsOutside;
    }

    let mut op = op_start;
    loop {
        if outpt_arena[op].pt.y != pt.y {
            break;
        }
        op = outpt_arena[op].next;
        if op == op_start {
            break;
        }
    }
    if outpt_arena[op].pt.y == pt.y {
        return PointInPolygonResult::IsOutside;
    }

    let mut is_above = outpt_arena[op].pt.y < pt.y;
    let starting_above = is_above;
    let mut val = 0;
    let mut op2 = outpt_arena[op].next;

    while op2 != op {
        if is_above {
            while op2 != op && outpt_arena[op2].pt.y < pt.y {
                op2 = outpt_arena[op2].next;
            }
        } else {
            while op2 != op && outpt_arena[op2].pt.y > pt.y {
                op2 = outpt_arena[op2].next;
            }
        }
        if op2 == op {
            break;
        }

        if outpt_arena[op2].pt.y == pt.y {
            let prev = outpt_arena[op2].prev;
            if outpt_arena[op2].pt.x == pt.x
                || (outpt_arena[op2].pt.y == outpt_arena[prev].pt.y
                    && (pt.x < outpt_arena[prev].pt.x) != (pt.x < outpt_arena[op2].pt.x))
            {
                return PointInPolygonResult::IsOn;
            }
            op2 = outpt_arena[op2].next;
            if op2 == op {
                break;
            }
            continue;
        }

        let prev = outpt_arena[op2].prev;
        if pt.x < outpt_arena[op2].pt.x && pt.x < outpt_arena[prev].pt.x {
            // do nothing
        } else if pt.x > outpt_arena[prev].pt.x && pt.x > outpt_arena[op2].pt.x {
            val = 1 - val;
        } else {
            let i = cross_product_sign(outpt_arena[prev].pt, outpt_arena[op2].pt, pt);
            if i == 0 {
                return PointInPolygonResult::IsOn;
            }
            if (i < 0) == is_above {
                val = 1 - val;
            }
        }
        is_above = !is_above;
        op2 = outpt_arena[op2].next;
    }

    if is_above != starting_above {
        let prev = outpt_arena[op2].prev;
        let i = cross_product_sign(outpt_arena[prev].pt, outpt_arena[op2].pt, pt);
        if i == 0 {
            return PointInPolygonResult::IsOn;
        }
        if (i < 0) == is_above {
            val = 1 - val;
        }
    }

    if val == 0 {
        PointInPolygonResult::IsOutside
    } else {
        PointInPolygonResult::IsInside
    }
}
