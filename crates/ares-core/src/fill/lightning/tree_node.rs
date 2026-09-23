//! `FillLightning::Node` (`src/libslic3r/Fill/Lightning/TreeNode.cpp`): the
//! lightning-tree vertex — a `shared_ptr`/`weak_ptr` parent-children tree with
//! `deepCopy`/`reroot`/`realign`/`straighten`/`prune`/`convertToPolylines`/
//! `removeJunctionOverlap`. Ported to `Rc<RefCell<Node>>` + `Weak` parents.
//! `convertToPolylines` picks the first child via glibc `rand()` (byte parity).

use std::cell::RefCell;
use std::rc::{Rc, Weak};

use crate::geometry::point_in_polygon;
use crate::geometry::{BoundingBox, Coord, EdgeGrid, Point, Polygon, Polyline};

use super::{glibc_rand::GlibcRand, segment_distance_squared};

pub(super) type NodeSPtr = Rc<RefCell<Node>>;

/// `FillLightning::Node`.
pub(super) struct Node {
    pub p: Point,
    pub is_root: bool,
    pub last_grounding_location: Option<Point>,
    pub parent: Weak<RefCell<Node>>,
    pub children: Vec<NodeSPtr>,
}

impl Node {
    pub(super) fn create(p: Point) -> NodeSPtr {
        Rc::new(RefCell::new(Self {
            p,
            is_root: true,
            last_grounding_location: None,
            parent: Weak::new(),
            children: Vec::new(),
        }))
    }

    /// `Node(p, last_grounding_location)` ctor — root with grounding location.
    pub(super) fn create_grounded(p: Point, last_grounding: Option<Point>) -> NodeSPtr {
        let n = Self::create(p);
        let mut b = n.borrow_mut();
        b.is_root = true;
        b.last_grounding_location = last_grounding;
        drop(b);
        n
    }

    /// `getWeightedDistance` — euclid dist minus a valence boost (TreeNode.cpp).
    pub(super) fn get_weighted_distance(
        &self,
        unsupported: Point,
        supporting_radius: Coord,
    ) -> Coord {
        const VALENCE_BOOST_MULT: Coord = 4;
        let valence = (!self.is_root) as usize + self.children.len();
        let valence_boost = if valence > 0 && valence < 4 {
            VALENCE_BOOST_MULT * supporting_radius
        } else {
            0
        };
        let dist = (((self.p.x() - unsupported.x()) as f64).powi(2)
            + ((self.p.y() - unsupported.y()) as f64).powi(2))
        .sqrt() as Coord;
        dist - valence_boost
    }

    /// `hasOffspring` — is `to_be_checked` this node or a descendant.
    pub(super) fn has_offspring(this: &NodeSPtr, to_be_checked: &NodeSPtr) -> bool {
        if Rc::ptr_eq(this, to_be_checked) {
            return true;
        }
        this.borrow()
            .children
            .iter()
            .any(|c| Self::has_offspring(c, to_be_checked))
    }

    /// `addChild(p)` — create+attach a child.
    pub(super) fn add_child(this: &NodeSPtr, p: Point) -> NodeSPtr {
        let child = Self::create(p);
        Self::add_child_node(this, child)
    }

    /// `addChild(node)` — attach an existing node as a child.
    pub(super) fn add_child_node(this: &NodeSPtr, child: NodeSPtr) -> NodeSPtr {
        {
            let mut c = child.borrow_mut();
            c.parent = Rc::downgrade(this);
            c.is_root = false;
        }
        this.borrow_mut().children.push(child.clone());
        child
    }

    /// `deepCopy` — clone the subtree, preserving root/last_grounding.
    pub(super) fn deep_copy(this: &NodeSPtr) -> NodeSPtr {
        let (p, is_root, last_grounding, children) = {
            let n = this.borrow();
            (
                n.p,
                n.is_root,
                n.last_grounding_location,
                n.children.clone(),
            )
        };
        let local_root = Self::create(p);
        {
            let mut lr = local_root.borrow_mut();
            lr.is_root = is_root;
            if is_root {
                lr.last_grounding_location = Some(last_grounding.unwrap_or(p));
            }
            lr.children.reserve(children.len());
        }
        for child in &children {
            let copy = Self::deep_copy(child);
            copy.borrow_mut().parent = Rc::downgrade(&local_root);
            local_root.borrow_mut().children.push(copy);
        }
        local_root
    }

    /// `reroot` — re-root this subtree onto `new_parent` (or make root).
    pub(super) fn reroot(this: &NodeSPtr, new_parent: Option<NodeSPtr>) {
        let (is_root, parent, children) = {
            let n = this.borrow();
            (n.is_root, n.parent.upgrade(), n.children.clone())
        };
        if !is_root {
            let old_parent = parent.expect("non-root has a parent");
            Self::reroot(&old_parent, Some(this.clone()));
            this.borrow_mut().children.push(old_parent);
        }
        if let Some(np) = new_parent {
            this.borrow_mut().children.retain(|c| !Rc::ptr_eq(c, &np));
            let mut n = this.borrow_mut();
            n.is_root = false;
            n.parent = Rc::downgrade(&np);
        } else {
            let mut n = this.borrow_mut();
            n.is_root = true;
            n.parent = Weak::new();
        }
    }

    /// `closestNode` — the subtree node nearest `loc` (euclidean).
    pub(super) fn closest_node(this: &NodeSPtr, loc: Point) -> NodeSPtr {
        let mut result = this.clone();
        let mut closest = dist(this.borrow().p, loc);
        for child in &this.borrow().children {
            let cand = Self::closest_node(child, loc);
            let d = dist(cand.borrow().p, loc);
            if d < closest {
                closest = d;
                result = cand;
            }
        }
        result
    }

    /// `propagateToNextLayer` — deepCopy, prune, straighten, realign; push if
    /// the realigned root survives inside `next_outlines`.
    pub(super) fn propagate_to_next_layer(
        this: &NodeSPtr,
        next_trees: &mut Vec<NodeSPtr>,
        next_outlines: &[Polygon],
        outline_locator: &EdgeGrid,
        prune_distance: Coord,
        smooth_magnitude: Coord,
        max_remove_colinear_dist: Coord,
    ) {
        let tree_below = Self::deep_copy(this);
        Self::prune(&tree_below, prune_distance);
        Self::straighten(&tree_below, smooth_magnitude, max_remove_colinear_dist);
        if Self::realign(&tree_below, next_outlines, outline_locator, next_trees) {
            next_trees.push(tree_below);
        }
    }

    /// `visitBranches` — DFS over branches, calling `visitor(parent_p, child_p)`.
    pub(super) fn visit_branches(this: &NodeSPtr, visitor: &mut dyn FnMut(Point, Point)) {
        let children = this.borrow().children.clone();
        let p = this.borrow().p;
        for child in &children {
            visitor(p, child.borrow().p);
            Self::visit_branches(child, visitor);
        }
    }

    /// `visitNodes` — DFS pre-order over nodes.
    pub(super) fn visit_nodes(this: &NodeSPtr, visitor: &mut dyn FnMut(&NodeSPtr)) {
        visitor(this);
        let children = this.borrow().children.clone();
        for child in &children {
            Self::visit_nodes(child, visitor);
        }
    }

    /// `prune` — prune leaf branches up to `pruning_distance`; returns the max
    /// pruned distance (TreeNode.cpp). Returns 0 if `pruning_distance <= 0`.
    pub(super) fn prune(this: &NodeSPtr, pruning_distance: Coord) -> Coord {
        if pruning_distance <= 0 {
            return 0;
        }
        let mut max_pruned = 0;
        let children = this.borrow().children.clone();
        let mut keep: Vec<NodeSPtr> = Vec::new();
        for child in &children {
            let dist_pruned_child = Self::prune(child, pruning_distance);
            if dist_pruned_child >= pruning_distance {
                max_pruned = max_pruned.max(dist_pruned_child);
                keep.push(child.clone());
            } else {
                let a = this.borrow().p;
                let b = child.borrow().p;
                let ab_len = dist(a, b);
                if dist_pruned_child + ab_len <= pruning_distance {
                    // prune the whole child
                    max_pruned = max_pruned.max(dist_pruned_child + ab_len);
                    // don't keep
                } else {
                    // prune part-way: n = b + normalize(a-b)*remain (move the
                    // leaf toward the parent by the remaining distance).
                    let remain = pruning_distance - dist_pruned_child;
                    let (bx, by) = (b.x() as f64, b.y() as f64);
                    let (ax, ay) = (a.x() as f64, a.y() as f64);
                    let (dx, dy) = (ax - bx, ay - by); // ba = a - b
                    let nlen = (dx * dx + dy * dy).sqrt();
                    let n = if nlen == 0.0 {
                        b
                    } else {
                        Point::new(
                            (bx + dx / nlen * remain as f64).round() as Coord,
                            (by + dy / nlen * remain as f64).round() as Coord,
                        )
                    };
                    child.borrow_mut().p = n;
                    max_pruned = max_pruned.max(pruning_distance);
                    keep.push(child.clone());
                }
            }
        }
        this.borrow_mut().children = keep;
        max_pruned
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
            let mut reground_me = false;
            let children = this.borrow().children.clone();
            let mut kept = Vec::new();
            for child in children {
                let mut connect_branch =
                    Self::realign(&child, outlines, outline_locator, rerooted_parts);
                let child_p = child.borrow().p;
                if connect_branch
                    && line_segment_polygons_intersection(
                        child_p,
                        p,
                        outline_locator,
                        outline_locator.resolution() * 2,
                    )
                    .is_some()
                {
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

    /// `convertToPolylines` — depth-first branch order, first child picked by
    /// `rand()` (glibc), other children get new polylines (TreeNode.cpp:355).
    pub(super) fn convert_to_polylines(
        this: &NodeSPtr,
        line_overlap: Coord,
        rng: &mut GlibcRand,
    ) -> Vec<Polyline> {
        let mut result: Vec<Polyline> = vec![Polyline::new(Vec::new())];
        Self::convert_rec(this, 0, &mut result, rng);
        Self::remove_junction_overlap(&mut result, line_overlap);
        result
    }

    fn convert_rec(
        this: &NodeSPtr,
        long_line_idx: usize,
        output: &mut Vec<Polyline>,
        rng: &mut GlibcRand,
    ) {
        let (children, p) = {
            let n = this.borrow();
            (n.children.clone(), n.p)
        };
        if children.is_empty() {
            output[long_line_idx].points_mut().push(p);
            return;
        }
        let first_child_idx = (rng.next() as usize) % children.len();
        Self::convert_rec(&children[first_child_idx], long_line_idx, output, rng);
        output[long_line_idx].points_mut().push(p);
        for idx_offset in 1..children.len() {
            let child_idx = (first_child_idx + idx_offset) % children.len();
            let child = children[child_idx].clone();
            output.push(Polyline::new(Vec::new()));
            let child_line_idx = output.len() - 1;
            Self::convert_rec(&child, child_line_idx, output, rng);
            output[child_line_idx].points_mut().push(p);
        }
    }

    /// `removeJunctionOverlap` — reduce each polyline's tail by `line_overlap`.
    fn remove_junction_overlap(result_lines: &mut Vec<Polyline>, line_overlap: Coord) {
        let reduction = line_overlap;
        let mut i = 0;
        while i < result_lines.len() {
            if result_lines[i].points().len() <= 1 {
                result_lines.swap_remove(i);
                continue;
            }
            let mut to_be_reduced = reduction;
            let pts = result_lines[i].points().to_vec();
            let mut a = *pts.last().unwrap();
            let mut new_pts = pts.clone();
            let mut idx = pts.len() as i64 - 2;
            while idx >= 0 {
                let b = pts[idx as usize];
                let ab_len = dist(a, b);
                if ab_len >= to_be_reduced {
                    // move the tail point toward `b` by the remaining reduction
                    let (ax, ay) = (a.x() as f64, a.y() as f64);
                    let (bx, by) = (b.x() as f64, b.y() as f64);
                    let (dx, dy) = (bx - ax, by - ay);
                    let t = to_be_reduced as f64 / ab_len as f64;
                    *new_pts.last_mut().unwrap() = Point::new(
                        (ax + dx * t).round() as Coord,
                        (ay + dy * t).round() as Coord,
                    );
                    break;
                } else {
                    to_be_reduced -= ab_len;
                    new_pts.pop();
                }
                a = b;
                idx -= 1;
            }
            if new_pts.len() <= 1 {
                result_lines.swap_remove(i);
            } else {
                result_lines[i] = Polyline::new(new_pts);
                i += 1;
            }
        }
    }
}

/// `RectilinearJunction` — (total_recti_dist, junction_loc) for `straighten`.
struct RectilinearJunction {
    total_recti_dist: Coord,
    junction_loc: Point,
}

/// `lineSegmentPolygonsIntersection` (TreeNode.cpp) — walk the outline locator
/// along `a→b`, find the nearest segment intersection within `within_max_dist`
/// of `b`. Returns the intersection `Point`.
pub(super) fn line_segment_polygons_intersection(
    a: Point,
    b: Point,
    outline_locator: &EdgeGrid,
    within_max_dist: Coord,
) -> Option<Point> {
    let mut d2min = f64::MAX;
    let mut result: Option<(f64, f64)> = None;
    let _ = outline_locator.visit_cells_intersecting_line(a, b, |_row, _col, edges| {
        for &edge in edges {
            let (s1, s2) = outline_locator.segment(edge);
            if let Some(ip) = seg_seg_intersection(
                (a.x() as f64, a.y() as f64),
                (b.x() as f64 - a.x() as f64, b.y() as f64 - a.y() as f64),
                (s1.x() as f64, s1.y() as f64),
                (s2.x() as f64 - s1.x() as f64, s2.y() as f64 - s1.y() as f64),
            ) {
                let d = (ip.0 - b.x() as f64).powi(2) + (ip.1 - b.y() as f64).powi(2);
                if d < d2min {
                    d2min = d;
                    result = Some(ip);
                }
            }
        }
        true
    });
    if d2min < (within_max_dist as f64) * (within_max_dist as f64) {
        let (x, y) = result?;
        Some(Point::new(x as Coord, y as Coord))
    } else {
        None
    }
}

/// `Geometry::segment_segment_intersection` (Geometry.hpp:92) — f64 segments.
fn seg_seg_intersection(
    p1: (f64, f64),
    v1: (f64, f64),
    p2: (f64, f64),
    v2: (f64, f64),
) -> Option<(f64, f64)> {
    let mut denom = v1.0 * v2.1 - v2.0 * v1.1;
    if denom.abs() < 1e-6 {
        return None;
    }
    let s12 = (p1.0 - p2.0, p1.1 - p2.1);
    let mut s_numer = v1.0 * s12.1 - v1.1 * s12.0;
    let denom_pos = denom < 0.0;
    if denom_pos {
        denom = -denom;
        s_numer = -s_numer;
    }
    if s_numer < 0.0 {
        return None;
    }
    let mut t_numer = v2.0 * s12.1 - v2.1 * s12.0;
    if !denom_pos {
        t_numer = -t_numer;
    }
    if t_numer < 0.0 || s_numer > denom || t_numer > denom {
        return None;
    }
    let t = t_numer / denom;
    Some((p1.0 + t * v1.0, p1.1 + t * v1.1))
}

fn dist(a: Point, b: Point) -> Coord {
    (((a.x() - b.x()) as f64).powi(2) + ((a.y() - b.y()) as f64).powi(2)).sqrt() as Coord
}

/// `inside` (TreeNode.cpp) — `PointInPolygon` parity over all polys, odd count.
fn inside(polygons: &[Polygon], p: Point) -> bool {
    let mut count = 0;
    for poly in polygons {
        let r = point_in_polygon(p, poly.points());
        if r == -1 {
            return true;
        }
        count += r;
    }
    count % 2 == 1
}
