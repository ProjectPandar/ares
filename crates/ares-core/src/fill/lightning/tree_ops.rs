//! `FillLightning::Node` realign/straighten (`TreeNode.cpp`), split from
//! `tree_node.rs` for the <400 LOC rule.

use std::rc::{Rc, Weak};

use crate::geometry::{Coord, EdgeGrid, Point, Polygon};

use super::segment_distance_squared;
use super::tree_geom::{dist, inside, line_segment_polygons_intersection};
use super::tree_node::{Node, NodeSPtr};

/// `RectilinearJunction` — (total_recti_dist, junction_loc) for `straighten`.
struct RectilinearJunction {
    total_recti_dist: Coord,
    junction_loc: Point,
}

fn realign_inside(
    this: &NodeSPtr,
    outlines: &[Polygon],
    outline_locator: &EdgeGrid,
    rerooted_parts: &mut Vec<NodeSPtr>,
    p: Point,
) {
    let mut reground_me = false;
    let children = this.borrow().children.clone();
    let mut kept = Vec::new();
    for child in children {
        let mut connect_branch = Node::realign(&child, outlines, outline_locator, rerooted_parts);
        let child_p = child.borrow().p;
        let crossing = connect_branch
            .then(|| {
                line_segment_polygons_intersection(
                    child_p,
                    p,
                    outline_locator,
                    outline_locator.resolution() * 2,
                )
            })
            .flatten();
        if connect_branch && crossing.is_some() {
            {
                let mut c = child.borrow_mut();
                c.last_grounding_location = None;
                c.parent = Weak::new();
                c.is_root = true;
            }
            rerooted_parts.push(child.clone());
            reground_me = true;
            connect_branch = false;
        }
        if connect_branch {
            kept.push(child);
        }
    }
    this.borrow_mut().children = kept;
    if reground_me {
        this.borrow_mut().last_grounding_location = None;
    }
}

fn move_toward(this: &NodeSPtr, p: Point, dest: Point, magnitude: Coord) {
    let dm = dest.x() - p.x();
    let dn = dest.y() - p.y();
    let dm2 = dm * dm + dn * dn;
    let mag2 = magnitude * magnitude;
    let mut n = this.borrow_mut();
    if dm2 <= mag2 {
        n.p = dest;
    } else {
        // `m_p += ((destination - m_p).cast<double>().normalized() * magnitude)
        // .cast<coord_t>()` (TreeNode.cpp:260): the normalized-direction
        // displacement is truncated per-component, THEN added — not the
        // destination computed in f64 then rounded.
        let dlen = (dm2 as f64).sqrt();
        let step_x = (dm as f64 / dlen * magnitude as f64) as Coord;
        let step_y = (dn as f64 / dlen * magnitude as f64) as Coord;
        n.p = Point::new(n.p.x() + step_x, n.p.y() + step_y);
    }
}

fn remove_collinear(this: &NodeSPtr, parent: Option<NodeSPtr>, max_remove_colinear_dist2: i64) {
    const CLOSE_ENOUGH: i64 = 10;
    let cur_mp = this.borrow().p;
    let child_p = this.borrow().children.first().cloned();
    let (Some(child_p), Some(parent_node)) = (child_p, parent) else {
        return;
    };
    let cp = child_p.borrow().p;
    let pp = parent_node.borrow().p;
    let d = cp.x() - pp.x();
    let e = cp.y() - pp.y();
    let dd = d * d + e * e;
    if dd < max_remove_colinear_dist2
        && segment_distance_squared(cur_mp, pp, cp) < (CLOSE_ENOUGH * CLOSE_ENOUGH) as f64
    {
        child_p.borrow_mut().parent = Rc::downgrade(&parent_node);
        let mut par = parent_node.borrow_mut();
        for sib in par.children.iter_mut() {
            if Rc::ptr_eq(sib, this) {
                *sib = child_p.clone();
                break;
            }
        }
    }
}

impl Node {
    /// `realign` — keep only children with an unbroken in-outline connection;
    /// boundary-crossing children become rerooted parts (TreeNode.cpp).
    pub(super) fn realign(
        this: &NodeSPtr,
        outlines: &[Polygon],
        outline_locator: &EdgeGrid,
        rerooted_parts: &mut Vec<NodeSPtr>,
    ) -> bool {
        if outlines.is_empty() {
            return false;
        }
        let p = this.borrow().p;
        if inside(outlines, p) {
            realign_inside(this, outlines, outline_locator, rerooted_parts, p);
            return true;
        }
        // Outside the outlines: lift descendants out of this tree.
        let children = this.borrow().children.clone();
        for child in &children {
            if !Self::realign(child, outlines, outline_locator, rerooted_parts) {
                continue;
            }
            {
                let mut c = child.borrow_mut();
                c.last_grounding_location = Some(p);
                c.parent = Weak::new();
                c.is_root = true;
            }
            rerooted_parts.push(child.clone());
        }
        this.borrow_mut().children.clear();
        false
    }

    /// `straighten` (two-arg) — entry: magnitude + max_remove_colinear_dist.
    pub(super) fn straighten(this: &NodeSPtr, magnitude: Coord, max_remove_colinear_dist: Coord) {
        let p = this.borrow().p;
        let m2 = max_remove_colinear_dist * max_remove_colinear_dist;
        Self::straighten_rec(this, magnitude, p, 0, m2);
    }

    /// `straighten` recursive — returns the `RectilinearJunction` for the
    /// junction below this node (TreeNode.cpp:204+).
    fn straighten_rec(
        this: &NodeSPtr,
        magnitude: Coord,
        junction_above: Point,
        accumulated_dist: Coord,
        max_remove_colinear_dist2: i64,
    ) -> RectilinearJunction {
        const JUNCT_NUM: i64 = 3;
        const JUNCT_DEN: i64 = 4;
        let junction_magnitude = magnitude * JUNCT_NUM / JUNCT_DEN;
        let (children, p, parent) = {
            let n = this.borrow();
            (n.children.clone(), n.p, n.parent.upgrade())
        };
        if children.len() == 1 {
            let child_p = children[0].clone();
            let child_dist = dist(p, child_p.borrow().p);
            let junction_below = Self::straighten_rec(
                &child_p,
                magnitude,
                junction_above,
                accumulated_dist + child_dist,
                max_remove_colinear_dist2,
            );
            let total_dist = junction_below.total_recti_dist;
            let a = junction_above;
            let b = junction_below.junction_loc;
            if a != b {
                // destination = a + (b-a) * accumulated_dist / total_dist
                let (ax, ay) = (a.x(), a.y());
                let (bx, by) = (b.x() as i64, b.y() as i64);
                let (abx, aby) = (bx - ax, by - ay);
                let ad = accumulated_dist;
                let td = total_dist.max(1) as i64;
                let dest = Point::new((ax + abx * ad / td) as Coord, (ay + aby * ad / td) as Coord);
                move_toward(this, p, dest, magnitude);
            }
            // remove colinear nodes — `Line::distance_to_squared(m_p, ...)` uses the
            // LIVE `m_p` (post-move), not the pre-move `p` (TreeNode.cpp:269).
            remove_collinear(this, parent.clone(), max_remove_colinear_dist2);
            return junction_below;
        }
        // `Point junction_moving_dir` — an INTEGER point accumulated from
        // per-child truncated direction vectors (TreeNode.cpp:282-300), not an
        // f64 vector rounded at the end.
        const WEIGHT: f64 = 1000.0;
        let trunc_dir = |from: Point, to: Point| -> Point {
            let dx = to.x() as f64 - from.x() as f64;
            let dy = to.y() as f64 - from.y() as f64;
            let len = (dx * dx + dy * dy).sqrt();
            if len == 0.0 {
                Point::new(0, 0)
            } else {
                Point::new((dx / len * WEIGHT) as Coord, (dy / len * WEIGHT) as Coord)
            }
        };
        let mut junction_moving_dir = trunc_dir(p, junction_above);
        let mut prevent_moving = false;
        for child_p in &children {
            let child_dist = dist(p, child_p.borrow().p);
            let below =
                Self::straighten_rec(child_p, magnitude, p, child_dist, max_remove_colinear_dist2);
            junction_moving_dir = Point::new(
                junction_moving_dir.x() + trunc_dir(p, below.junction_loc).x(),
                junction_moving_dir.y() + trunc_dir(p, below.junction_loc).y(),
            );
            if below.total_recti_dist < magnitude {
                prevent_moving = true;
            }
        }
        let (has_children, is_root) = {
            let n = this.borrow();
            (!n.children.is_empty(), n.is_root)
        };
        if (junction_moving_dir.x() != 0 || junction_moving_dir.y() != 0)
            && has_children
            && !is_root
            && !prevent_moving
        {
            // `coord_t(junction_moving_dir.norm())` — the integer-point norm.
            let mlen = ((junction_moving_dir.x() as f64 * junction_moving_dir.x() as f64
                + junction_moving_dir.y() as f64 * junction_moving_dir.y() as f64)
                .sqrt()) as Coord;
            if mlen > junction_magnitude {
                junction_moving_dir = Point::new(
                    junction_moving_dir.x() * junction_magnitude / mlen,
                    junction_moving_dir.y() * junction_magnitude / mlen,
                );
            }
            let mut n = this.borrow_mut();
            n.p = Point::new(
                n.p.x() + junction_moving_dir.x(),
                n.p.y() + junction_moving_dir.y(),
            );
        }
        let cur_p = this.borrow().p;
        RectilinearJunction {
            total_recti_dist: accumulated_dist,
            junction_loc: cur_p,
        }
    }
}
