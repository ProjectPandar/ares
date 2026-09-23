//! `FillLightning::Layer` grounding/reconnect methods + geometry helpers
//! (`Layer.cpp`), split from `layer.rs` for the <400 LOC rule.

use std::cell::RefCell;
use std::rc::{Rc, Weak};

use crate::geometry::{BoundingBox, Coord, EdgeGrid, Point, Polygon};

use super::layer::{GroundingLocation, LOCATOR_CELL_SIZE, Layer, SparseNodeGrid, to_grid_point};
use super::tree_geom::{inside, line_segment_polygons_intersection};
use super::tree_node::Node;
use super::tree_node::NodeSPtr;

impl Layer {
    pub(super) fn get_best_grounding_location(
        &self,
        unsupported_location: Point,
        current_outlines: &[Polygon],
        current_outlines_bbox: BoundingBox,
        outline_locator: &EdgeGrid,
        supporting_radius: Coord,
        wall_supporting_radius: Coord,
        tree_node_locator: &SparseNodeGrid,
        exclude_tree: Option<&NodeSPtr>,
    ) -> GroundingLocation {
        // Closest point on current_outlines to the unsupported location.
        let mut node_location = Point::new(0, 0);
        let mut d2 = f64::MAX;
        for contour in current_outlines {
            let pts = contour.points();
            if pts.len() > 2 {
                let mut prev = *pts.last().unwrap();
                for &p2 in pts {
                    let (d, cp) = segment_closest(prev, p2, unsupported_location);
                    if d < d2 {
                        d2 = d;
                        node_location = cp;
                    }
                    prev = p2;
                }
            }
        }
        let within_dist = dist(node_location, unsupported_location);
        let mut current_dist = get_weighted_distance_boundary(node_location, unsupported_location);
        let mut sub_tree: Option<NodeSPtr> = None;

        if current_dist >= wall_supporting_radius {
            let search_radius = current_dist.min(within_dist);
            let rmin = to_grid_point(
                Point::new(
                    unsupported_location.x() - search_radius,
                    unsupported_location.y() - search_radius,
                ),
                current_outlines_bbox,
            );
            let rmax = to_grid_point(
                Point::new(
                    unsupported_location.x() + search_radius + LOCATOR_CELL_SIZE,
                    unsupported_location.y() + search_radius + LOCATOR_CELL_SIZE,
                ),
                current_outlines_bbox,
            );
            // Serial scan (upstream runs a deterministic-order parallel_for
            // that resolves ties by grid address — the serial scan is the same
            // order).
            let mut best_addr = (i64::MAX, i64::MAX);
            for gy in rmin.y()..rmax.y() {
                for gx in rmin.x()..rmax.x() {
                    let addr = Point::new(gx, gy);
                    for weak in tree_node_locator.equal_range(addr) {
                        let Some(candidate) = weak.upgrade() else {
                            continue;
                        };
                        if let Some(ex) = exclude_tree {
                            if Rc::ptr_eq(&candidate, ex) || Node::has_offspring(ex, &candidate) {
                                continue;
                            }
                        }
                        if polygon_collides_with_line(
                            unsupported_location,
                            candidate.borrow().p,
                            outline_locator,
                        ) {
                            continue;
                        }
                        let cd = candidate
                            .borrow()
                            .get_weighted_distance(unsupported_location, supporting_radius);
                        if cd < current_dist || (cd == current_dist && (gy, gx) < best_addr) {
                            current_dist = cd;
                            sub_tree = Some(candidate);
                            best_addr = (gy, gx);
                        }
                    }
                }
            }
        }
        match sub_tree {
            None => GroundingLocation {
                tree_node: None,
                boundary_location: Some(node_location),
            },
            Some(t) => GroundingLocation {
                tree_node: Some(t),
                boundary_location: None,
            },
        }
    }

    /// `attach` — attach the unsupported point: new root on the boundary, or a
    /// child on an existing tree (Layer.cpp:201).
    pub(super) fn attach(
        &mut self,
        unsupported_location: Point,
        grounding: &GroundingLocation,
    ) -> (NodeSPtr, Option<NodeSPtr>) {
        if let Some(boundary) = grounding.boundary_location {
            let new_root = Node::create_grounded(boundary, Some(boundary));
            let new_child = Node::add_child(&new_root, unsupported_location);
            self.tree_roots.push(new_root.clone());
            (new_child, Some(new_root))
        } else {
            let tree = grounding.tree_node.clone().unwrap();
            let new_child = Node::add_child(&tree, unsupported_location);
            (new_child, None)
        }
    }

    /// `reconnectRoots` — re-ground each propagated root (Layer.cpp:220).
    pub(super) fn reconnect_roots(
        &mut self,
        to_be_reconnected: Vec<NodeSPtr>,
        current_outlines: &[Polygon],
        current_outlines_bbox: BoundingBox,
        outline_locator: &EdgeGrid,
        supporting_radius: Coord,
        wall_supporting_radius: Coord,
    ) {
        const TREE_CONNECTING_IGNORE_OFFSET: Coord = 100;
        let mut tree_node_locator = SparseNodeGrid::new();
        self.fill_locator(&mut tree_node_locator, current_outlines_bbox);
        let within_max_dist = outline_locator.resolution() * 2;

        if std::env::var("ARES_DEBUG_LIGHTNING").is_ok() {
            eprintln!(
                "RECONNECT entry count={} existing_roots={}",
                to_be_reconnected.len(),
                self.tree_roots.len()
            );
        }
        for root_ptr in to_be_reconnected {
            let old_root_pos = self
                .tree_roots
                .iter()
                .position(|r| Rc::ptr_eq(r, &root_ptr));

            let last_grounding = root_ptr.borrow().last_grounding_location;
            if let Some(ground_loc) = last_grounding {
                if ground_loc != root_ptr.borrow().p {
                    if let Some(new_root_pt) = super::tree_geom::line_segment_polygons_intersection(
                        root_ptr.borrow().p,
                        ground_loc,
                        outline_locator,
                        within_max_dist,
                    ) {
                        let new_root = Node::create_grounded(new_root_pt, Some(new_root_pt));
                        // `root_ptr->addChild(new_root); new_root->reroot();`
                        // (Layer.cpp:248-250): new_root becomes a CHILD of the
                        // old root, then `reroot(nullptr)` flips the edge —
                        // new_root ends up the ROOT with the entire old tree
                        // beneath it (depth +1, subtree preserved).
                        Node::add_child_node(&root_ptr, new_root.clone());
                        Node::reroot(&new_root, None);
                        tree_node_locator.insert(
                            to_grid_point(new_root.borrow().p, current_outlines_bbox),
                            &new_root,
                        );
                        if let Some(pos) = old_root_pos {
                            self.tree_roots[pos] = new_root;
                        }
                        continue;
                    }
                }
            }

            let tree_connecting_ignore_width =
                wall_supporting_radius - TREE_CONNECTING_IGNORE_OFFSET;
            let ground = self.get_best_grounding_location(
                root_ptr.borrow().p,
                current_outlines,
                current_outlines_bbox,
                outline_locator,
                supporting_radius,
                tree_connecting_ignore_width,
                &tree_node_locator,
                Some(&root_ptr),
            );
            if std::env::var("ARES_DEBUG_LIGHTNING").is_ok() {
                eprintln!(
                    "RECONNECT pos={:?} boundary={:?} tree={} ground_loc={:?} p={:?}",
                    old_root_pos,
                    ground.boundary_location.is_some(),
                    ground.tree_node.is_some(),
                    last_grounding.map(|g| (g.x(), g.y())),
                    (root_ptr.borrow().p.x(), root_ptr.borrow().p.y())
                );
            }
            if let Some(boundary) = ground.boundary_location {
                if boundary == root_ptr.borrow().p {
                    continue;
                }
                let new_root = Node::create_grounded(boundary, Some(boundary));
                let attach_ptr = Node::closest_node(&root_ptr, new_root.borrow().p);
                Node::reroot(&attach_ptr, None);
                Node::add_child_node(&new_root, attach_ptr);
                tree_node_locator.insert(
                    to_grid_point(new_root.borrow().p, current_outlines_bbox),
                    &new_root,
                );
                if let Some(pos) = old_root_pos {
                    self.tree_roots[pos] = new_root;
                }
            } else {
                let ground_tree = ground.tree_node.clone().unwrap();
                let attach_ptr = Node::closest_node(&root_ptr, ground_tree.borrow().p);
                Node::reroot(&attach_ptr, None);
                Node::add_child_node(&ground_tree, attach_ptr);
                // remove old root: replace with back + pop
                if let Some(pos) = old_root_pos {
                    self.tree_roots.swap_remove(pos);
                }
            }
        }
    }
}

fn polygon_collides_with_line(from: Point, to: Point, locator: &EdgeGrid) -> bool {
    let mut hit = false;
    let _ = locator.visit_cells_intersecting_line(from, to, |_r, _c, edges| {
        for &edge in edges {
            let (s1, s2) = locator.segment(edge);
            if segments_intersect(s1, s2, from, to) {
                hit = true;
                return false;
            }
        }
        true
    });
    hit
}

/// `Geometry::segments_intersect` (Geometry.hpp:123) — sign-product crossing
/// test including the collinear-overlap case (a strict `>0` orientation test
/// misses collinear overlapping segments).
fn segments_intersect(ip1: Point, ip2: Point, jp1: Point, jp2: Point) -> bool {
    let cross2 = |ax: i128, ay: i128, bx: i128, by: i128| ax * by - ay * bx;
    let signs = |a1: Point, a2: Point, b1: Point, b2: Point| -> (i32, i32) {
        let ivx = (a2.x() - a1.x()) as i128;
        let ivy = (a2.y() - a1.y()) as i128;
        let v1x = (b1.x() - a1.x()) as i128;
        let v1y = (b1.y() - a1.y()) as i128;
        let v2x = (b2.x() - a1.x()) as i128;
        let v2y = (b2.y() - a1.y()) as i128;
        let t1 = cross2(ivx, ivy, v1x, v1y);
        let t2 = cross2(ivx, ivy, v2x, v2y);
        let s = |t: i128| {
            if t > 0 {
                1
            } else if t < 0 {
                -1
            } else {
                0
            }
        };
        (s(t1), s(t2))
    };
    let sign1 = signs(ip1, ip2, jp1, jp2);
    let sign2 = signs(jp1, jp2, ip1, ip2);
    let test1 = sign1.0 * sign1.1;
    let test2 = sign2.0 * sign2.1;
    if test1 <= 0 && test2 <= 0 {
        if test1 != 0 || test2 != 0 {
            return true; // not collinear — they intersect
        }
        if sign1.0 == 0 && sign1.1 == 0 {
            // Collinear — check overlap on the dominant axis.
            let vi_x = (ip2.x() - ip1.x()).abs();
            let vi_y = (ip2.y() - ip1.y()).abs();
            let axis = if vi_x > vi_y { 0 } else { 1 };
            let coord = |p: Point| if axis == 0 { p.x() } else { p.y() };
            let (mut i, mut j, mut k, mut l) = (coord(ip1), coord(ip2), coord(jp1), coord(jp2));
            if i > j {
                std::mem::swap(&mut i, &mut j);
            }
            if k > l {
                std::mem::swap(&mut k, &mut l);
            }
            return (k >= i && k <= j) || (i >= k && i <= l);
        }
    }
    false
}

/// `line_alg::distance_to_squared` with the projected `closest_point` out-arg
/// (Layer.cpp:130) — returns (d², closest point on segment).
fn segment_closest(a: Point, b: Point, p: Point) -> (f64, Point) {
    let (ax, ay) = (a.x() as f64, a.y() as f64);
    let (bx, by) = (b.x() as f64, b.y() as f64);
    let (px, py) = (p.x() as f64, p.y() as f64);
    let (dx, dy) = (bx - ax, by - ay);
    let len2 = dx * dx + dy * dy;
    let t = if len2 == 0.0 {
        0.0
    } else {
        (((px - ax) * dx + (py - ay) * dy) / len2).clamp(0.0, 1.0)
    };
    let (cx, cy) = (ax + t * dx, ay + t * dy);
    let d2 = (px - cx).powi(2) + (py - cy).powi(2);
    (d2, Point::new(cx as Coord, cy as Coord))
}

fn dist(a: Point, b: Point) -> Coord {
    (((a.x() - b.x()) as f64).powi(2) + ((a.y() - b.y()) as f64).powi(2)).sqrt() as Coord
}

/// `getWeightedDistance` for a boundary location (no valence — boundary points
/// get no boost): just the euclidean distance (Layer.cpp uses the same fn on a
/// boundary Point; for boundary it's the plain distance).
fn get_weighted_distance_boundary(boundary: Point, unsupported: Point) -> Coord {
    dist(boundary, unsupported)
}
