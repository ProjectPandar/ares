// Copyright (c) Angus Johnson 2010-2025.
// BSL-1.0. Rust seed by MatterHackers.
// Derived from clipper2-rust 2f5e9d4249bc4d624773094ff6164335d9ff6ae7.
// See ../LICENSE-Boost-1.0 and ../SOURCE.md for pinned source authority.

use super::*;

/// Check if path1 (as OutPt list) is contained within path2 (as OutPt list)
/// Direct port from clipper.engine.cpp Path2ContainsPath1 (line 576)
pub fn path2_contains_path1_outpt(
    op1_start: usize,
    op2_start: usize,
    outpt_arena: &[OutPt],
) -> bool {
    let mut pip = PointInPolygonResult::IsOn;
    let mut op = op1_start;
    loop {
        match point_in_op_polygon(outpt_arena[op].pt, op2_start, outpt_arena) {
            PointInPolygonResult::IsOutside => {
                if pip == PointInPolygonResult::IsOutside {
                    return false;
                }
                pip = PointInPolygonResult::IsOutside;
            }
            PointInPolygonResult::IsInside => {
                if pip == PointInPolygonResult::IsInside {
                    return true;
                }
                pip = PointInPolygonResult::IsInside;
            }
            _ => {}
        }
        op = outpt_arena[op].next;
        if op == op1_start {
            break;
        }
    }
    // result unclear, try using cleaned paths
    let clean1 = get_clean_path(op1_start, outpt_arena);
    let clean2 = get_clean_path(op2_start, outpt_arena);
    path2_contains_path1(&clean1, &clean2)
}

/// Check if path1 is contained within path2 using Path64 vectors
/// Direct port from clipper.h line 717 - template Path2ContainsPath1<T>
/// precondition: paths must not intersect, except for transient micro intersections
pub fn path2_contains_path1(path1: &Path64, path2: &Path64) -> bool {
    let mut pip = PointInPolygonResult::IsOn;
    for pt in path1 {
        match point_in_polygon(*pt, path2) {
            PointInPolygonResult::IsOutside => {
                if pip == PointInPolygonResult::IsOutside {
                    return false;
                }
                pip = PointInPolygonResult::IsOutside;
            }
            PointInPolygonResult::IsInside => {
                if pip == PointInPolygonResult::IsInside {
                    return true;
                }
                pip = PointInPolygonResult::IsInside;
            }
            _ => {}
        }
    }
    if pip != PointInPolygonResult::IsInside {
        return false;
    }
    // result is likely true but check midpoint
    let mp1 = get_bounds_path(path1).mid_point();
    point_in_polygon(mp1, path2) == PointInPolygonResult::IsInside
}

/// `clipper.engine.cpp:2917–2954`: preserve the original walk and output admission.
pub fn build_path64_from_outpt(
    op_start: usize,
    reverse: bool,
    is_open: bool,
    outpt_arena: &[OutPt],
) -> Option<Path64> {
    let next = outpt_arena[op_start].next;
    if next == op_start || (!is_open && next == outpt_arena[op_start].prev) {
        return None;
    }
    let start = if reverse { op_start } else { next };
    let mut last = outpt_arena[start].pt;
    let mut current = if reverse {
        outpt_arena[start].prev
    } else {
        outpt_arena[start].next
    };
    let mut path = vec![last];
    while current != start {
        if outpt_arena[current].pt != last {
            last = outpt_arena[current].pt;
            path.push(last);
        }
        current = if reverse {
            outpt_arena[current].prev
        } else {
            outpt_arena[current].next
        };
    }
    if !is_open && path.len() == 3 && is_very_small_triangle(current, outpt_arena) {
        None
    } else {
        Some(path)
    }
}

/// Get the last output point for a hot edge
/// Direct port from clipper.engine.cpp GetLastOp (line 2496)
#[inline]
pub fn get_last_op(
    hot_edge_idx: usize,
    active_arena: &[Active],
    outrec_list: &[OutRec],
    outpt_arena: &[OutPt],
) -> Option<usize> {
    if let Some(or_idx) = active_arena[hot_edge_idx].outrec {
        let result = outrec_list[or_idx].pts?;
        if outrec_list[or_idx].front_edge != Some(hot_edge_idx) {
            Some(outpt_arena[result].next)
        } else {
            Some(result)
        }
    } else {
        None
    }
}

/// Fix all OutPt outrec references in an OutRec
/// Direct port from clipper.engine.cpp FixOutRecPts (line 2158)
pub fn fix_outrec_pts(outrec_idx: usize, outrec_list: &[OutRec], outpt_arena: &mut [OutPt]) {
    if let Some(pts) = outrec_list[outrec_idx].pts {
        let mut op = pts;
        loop {
            outpt_arena[op].outrec = outrec_idx;
            op = outpt_arena[op].next;
            if op == pts {
                break;
            }
        }
    }
}

/// Update all OutPt outrec references in an OutRec
/// Direct port from clipper.engine.cpp UpdateOutrecOwner (line 1684)
pub fn update_outrec_owner(outrec_idx: usize, outrec_list: &[OutRec], outpt_arena: &mut [OutPt]) {
    fix_outrec_pts(outrec_idx, outrec_list, outpt_arena);
}

/// `clipper.engine.cpp:2294–2303`: exclude self and retain the cleared source list.
pub fn move_splits(outrec_list: &mut [OutRec], from_or: usize, to_or: usize) {
    let mut from_splits = outrec_list[from_or].splits.take().unwrap();
    let to_splits = outrec_list[to_or].splits.get_or_insert_with(Vec::new);
    to_splits.extend(from_splits.iter().copied().filter(|split| *split != to_or));
    from_splits.clear();
    outrec_list[from_or].splits = Some(from_splits);
}

/// Uncouple an OutRec from its edges
/// Direct port from clipper.engine.cpp UncoupleOutRec (line 425)
pub fn uncouple_outrec(e_idx: usize, active_arena: &mut [Active], outrec_list: &mut [OutRec]) {
    if let Some(or_idx) = active_arena[e_idx].outrec {
        if let Some(fe) = outrec_list[or_idx].front_edge {
            active_arena[fe].outrec = None;
        }
        if let Some(be) = outrec_list[or_idx].back_edge {
            active_arena[be].outrec = None;
        }
        outrec_list[or_idx].front_edge = None;
        outrec_list[or_idx].back_edge = None;
    }
}

/// Extract an active from the SEL (sorted edge list)
/// Direct port from clipper.engine.cpp ExtractFromSEL
pub fn extract_from_sel(e_idx: usize, active_arena: &mut [Active]) -> Option<usize> {
    let next = active_arena[e_idx].next_in_sel;
    if let Some(next_idx) = next {
        active_arena[next_idx].prev_in_sel = active_arena[e_idx].prev_in_sel;
    }
    if let Some(prev_idx) = active_arena[e_idx].prev_in_sel {
        active_arena[prev_idx].next_in_sel = next;
    }
    active_arena[e_idx].prev_in_sel = None;
    active_arena[e_idx].next_in_sel = None;
    next
}

/// Insert tmp before left in SEL
/// Direct port from clipper.engine.cpp Insert1Before2InSEL
pub fn insert1_before2_in_sel(tmp_idx: usize, left_idx: usize, active_arena: &mut [Active]) {
    let prev = active_arena[left_idx].prev_in_sel;
    active_arena[tmp_idx].prev_in_sel = prev;
    if let Some(prev_idx) = prev {
        active_arena[prev_idx].next_in_sel = Some(tmp_idx);
    }
    active_arena[tmp_idx].next_in_sel = Some(left_idx);
    active_arena[left_idx].prev_in_sel = Some(tmp_idx);
}
