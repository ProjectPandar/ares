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
            let mut reground_me = false;
            let children = this.borrow().children.clone();
            let mut kept = Vec::new();
            for child in children {
                let mut connect_branch =
                    Self::realign(&child, outlines, outline_locator, rerooted_parts);
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
                if std::env::var("ARES_DEBUG_LIGHTNING").is_ok() {
                    eprintln!(
                        "REALIGN root=({},{}) child=({},{}) cb={} cross={:?}",
                        p.x(),
                        p.y(),
                        child_p.x(),
                        child_p.y(),
                        connect_branch,
                        crossing.map(|q| (q.x(), q.y()))
                    );
                }
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
            return true;
        }
        // Outside the outlines: lift descendants out of this tree.
        let children = this.borrow().children.clone();
        for child in &children {
            if Self::realign(child, outlines, outline_locator, rerooted_parts) {
                {
                    let mut c = child.borrow_mut();
                    c.last_grounding_location = Some(p);
                    c.parent = Weak::new();
                    c.is_root = true;
                }
                rerooted_parts.push(child.clone());
            }
        }
        this.borrow_mut().children.clear();
        false
    }

    /// `straighten` (two-arg) — entry: magnitude + max_remove_colinear_dist.
    pub(super) fn straighten(this: &NodeSPtr, magnitude: Coord, max_remove_colinear_dist: Coord) {
        let p = this.borrow().p;
        let m2 = (max_remove_colinear_dist as i64) * (max_remove_colinear_dist as i64);
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
        let (children, p, is_root, parent) = {
            let n = this.borrow();
            (n.children.clone(), n.p, n.is_root, n.parent.upgrade())
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
                let (ax, ay) = (a.x() as i64, a.y() as i64);
                let (bx, by) = (b.x() as i64, b.y() as i64);
                let (abx, aby) = (bx - ax, by - ay);
                let ad = accumulated_dist as i64;
                let td = total_dist.max(1) as i64;
                let dest = Point::new((ax + abx * ad / td) as Coord, (ay + aby * ad / td) as Coord);
                let dm = dest.x() - p.x();
                let dn = dest.y() - p.y();
                let dm2 = (dm as i64) * (dm as i64) + (dn as i64) * (dn as i64);
                let mag2 = (magnitude as i64) * (magnitude as i64);
                let mut n = this.borrow_mut();
                if dm2 <= mag2 {
                    n.p = dest;
                } else {
                    // move magnitude toward dest (normalized)
                    let dlen = (dm2 as f64).sqrt();
                    n.p = Point::new(
                        (p.x() as f64 + dm as f64 / dlen * magnitude as f64).round() as Coord,
                        (p.y() as f64 + dn as f64 / dlen * magnitude as f64).round() as Coord,
                    );
                }
            }
            // remove colinear nodes
            const CLOSE_ENOUGH: i64 = 10;
            let child_p = this.borrow().children.first().cloned();
            if let (Some(child_p), Some(parent_node)) = (child_p, parent.clone()) {
                let cp = child_p.borrow().p;
                let pp = parent_node.borrow().p;
                let d = cp.x() - pp.x();
                let e = cp.y() - pp.y();
                let dd = (d as i64) * (d as i64) + (e as i64) * (e as i64);
                if dd < max_remove_colinear_dist2
                    && segment_distance_squared(p, pp, cp) < (CLOSE_ENOUGH * CLOSE_ENOUGH) as f64
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
            junction_below
        } else {
            const WEIGHT: f64 = 1000.0;
            // junction_moving_dir = sum of normalized dirs * weight
            let (jx, jy) = (
                junction_above.x() as f64 - p.x() as f64,
                junction_above.y() as f64 - p.y() as f64,
            );
            let jlen = (jx * jx + jy * jy).sqrt();
            let mut mdx = if jlen == 0.0 { 0.0 } else { jx / jlen * WEIGHT };
            let mut mdy = if jlen == 0.0 { 0.0 } else { jy / jlen * WEIGHT };
            let mut prevent_moving = false;
            for child_p in &children {
                let child_dist = dist(p, child_p.borrow().p);
                let below = Self::straighten_rec(
                    child_p,
                    magnitude,
                    p,
                    child_dist,
                    max_remove_colinear_dist2,
                );
                let (bx, by) = (
                    below.junction_loc.x() as f64 - p.x() as f64,
                    below.junction_loc.y() as f64 - p.y() as f64,
                );
                let blen = (bx * bx + by * by).sqrt();
                if blen != 0.0 {
                    mdx += bx / blen * WEIGHT;
                    mdy += by / blen * WEIGHT;
                }
                if below.total_recti_dist < magnitude {
                    prevent_moving = true;
                }
            }
            let (has_children, is_root) = {
                let n = this.borrow();
                (!n.children.is_empty(), n.is_root)
            };
            if (mdx != 0.0 || mdy != 0.0) && has_children && !is_root && !prevent_moving {
                let mlen = (mdx * mdx + mdy * mdy).sqrt();
                let (mx, my) = if mlen > junction_magnitude as f64 {
                    (
                        mdx * junction_magnitude as f64 / mlen,
                        mdy * junction_magnitude as f64 / mlen,
                    )
                } else {
                    (mdx, mdy)
                };
                let mut n = this.borrow_mut();
                n.p = Point::new(
                    (p.x() as f64 + mx).round() as Coord,
                    (p.y() as f64 + my).round() as Coord,
                );
            }
            let cur_p = this.borrow().p;
            RectilinearJunction {
                total_recti_dist: accumulated_dist,
                junction_loc: cur_p,
            }
        }
    }
}
