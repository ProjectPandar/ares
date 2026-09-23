//! `FillLightning::Layer` (`src/libslic3r/Fill/Lightning/Layer.cpp`): a layer
//! of lightning trees. `generateNewTrees` seeds branches from the
//! `DistanceField` unsupported cells; `reconnectRoots` re-grounds propagated
//! roots; `convertToLines` emits the tree polylines clipped to the outline.
//! The node locator is a `SparseNodeGrid` (`unordered_multimap` keyed by the
//! grid point) — ported as `HashMap<Point, Vec<Weak<Node>>>`.

use std::collections::HashMap;
use std::rc::{Rc, Weak};

use crate::geometry::{
    BoundingBox, ClipperError, Coord, EdgeGrid, Point, Polygon, Polyline,
    intersection_open_polylines,
};

use super::distance_field::DistanceField;
use super::glibc_rand::GlibcRand;
use super::tree_node::{Node, NodeSPtr};

/// `locator_cell_size` — the SparseNodeGrid cell size (Layer.cpp `to_grid_point`
/// uses `locator_cell_size()`); upstream `Generator::locator_cell_size()`.
pub(super) const LOCATOR_CELL_SIZE: Coord = 4000; // ~4mm in coord_t

/// `SparseNodeGrid` — grid-point → node weak refs (unordered_multimap port).
pub(super) struct SparseNodeGrid {
    map: HashMap<(Coord, Coord), Vec<Weak<std::cell::RefCell<Node>>>>,
}

impl SparseNodeGrid {
    fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }
    fn insert(&mut self, grid_addr: Point, node: &NodeSPtr) {
        self.map
            .entry((grid_addr.x(), grid_addr.y()))
            .or_default()
            .push(Rc::downgrade(node));
    }
    /// `equal_range` — the candidate nodes at a grid address.
    fn equal_range(&self, grid_addr: Point) -> &[Weak<std::cell::RefCell<Node>>] {
        self.map
            .get(&(grid_addr.x(), grid_addr.y()))
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }
}

/// `GroundingLocation` (Layer.hpp) — where a new branch grounds: on an
/// existing tree node, or on the outline boundary.
pub(super) struct GroundingLocation {
    pub tree_node: Option<NodeSPtr>,
    pub boundary_location: Option<Point>,
}

impl GroundingLocation {
    /// `p()` — the grounding point (tree node loc or boundary loc).
    fn p(&self) -> Point {
        match &self.tree_node {
            Some(n) => n.borrow().p,
            None => self.boundary_location.unwrap(),
        }
    }
}

/// `to_grid_point` — `(point - bbox.min) / locator_cell_size`.
fn to_grid_point(p: Point, bbox: BoundingBox) -> Point {
    Point::new(
        (p.x() - bbox.min().x()) / LOCATOR_CELL_SIZE,
        (p.y() - bbox.min().y()) / LOCATOR_CELL_SIZE,
    )
}

/// `FillLightning::Layer`.
pub(super) struct Layer {
    pub tree_roots: Vec<NodeSPtr>,
}

impl Layer {
    pub(super) fn new() -> Self {
        Self {
            tree_roots: Vec::new(),
        }
    }

    /// `fillLocator` — populate the node grid with all tree nodes.
    fn fill_locator(&self, locator: &mut SparseNodeGrid, bbox: BoundingBox) {
        for tree in &self.tree_roots {
            let tree = tree.clone();
            Node::visit_nodes(&tree, &mut |node| {
                locator.insert(to_grid_point(node.borrow().p, bbox), node);
            });
        }
    }

    /// `generateNewTrees` — grow branches to support every unsupported cell.
    pub(super) fn generate_new_trees(
        &mut self,
        current_overhang: &[Polygon],
        current_outlines: &[Polygon],
        current_outlines_bbox: BoundingBox,
        outlines_locator: &EdgeGrid,
        supporting_radius: Coord,
        wall_supporting_radius: Coord,
    ) -> Result<(), ClipperError> {
        let mut distance_field = DistanceField::new(
            supporting_radius,
            current_outlines,
            current_outlines_bbox,
            current_overhang,
        )?;

        let mut tree_node_locator = SparseNodeGrid::new();
        self.fill_locator(&mut tree_node_locator, current_outlines_bbox);

        let mut unsupported_cell_idx = 0usize;
        while let Some((cell_idx, unsupported_location)) =
            distance_field.try_get_next_point(unsupported_cell_idx)
        {
            unsupported_cell_idx = cell_idx;
            let grounding = self.get_best_grounding_location(
                unsupported_location,
                current_outlines,
                current_outlines_bbox,
                outlines_locator,
                supporting_radius,
                wall_supporting_radius,
                &tree_node_locator,
                None,
            );
            let (new_child, new_parent) = self.attach(unsupported_location, &grounding);
            tree_node_locator.insert(
                to_grid_point(new_child.borrow().p, current_outlines_bbox),
                &new_child,
            );
            if let Some(np) = new_parent {
                tree_node_locator.insert(to_grid_point(np.borrow().p, current_outlines_bbox), &np);
            }
            distance_field.update(grounding.p(), unsupported_location);
        }
        Ok(())
    }

    /// `getBestGroundingLocation` — the closest boundary point or the best
    /// existing tree node to attach `unsupported_location` (Layer.cpp:111).
    fn get_best_grounding_location(
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
    fn attach(
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

        for root_ptr in to_be_reconnected {
            let old_root_pos = self
                .tree_roots
                .iter()
                .position(|r| Rc::ptr_eq(r, &root_ptr));

            let last_grounding = root_ptr.borrow().last_grounding_location;
            if let Some(ground_loc) = last_grounding {
                if ground_loc != root_ptr.borrow().p {
                    if let Some(new_root_pt) = super::tree_node::line_segment_polygons_intersection(
                        root_ptr.borrow().p,
                        ground_loc,
                        outline_locator,
                        within_max_dist,
                    ) {
                        let new_root = Node::create_grounded(new_root_pt, Some(new_root_pt));
                        // reroot new_root onto root_ptr
                        Node::reroot(&new_root, Some(root_ptr.clone()));
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

    /// `convertToLines` — emit the tree polylines clipped to the outline.
    pub(super) fn convert_to_lines(
        &self,
        limit_to_outline: &[Polygon],
        line_overlap: Coord,
    ) -> Result<Vec<Polyline>, ClipperError> {
        if self.tree_roots.is_empty() {
            return Ok(Vec::new());
        }
        let mut rng = GlibcRand::new();
        let mut result_lines = Vec::new();
        for tree in &self.tree_roots {
            result_lines.extend(Node::convert_to_polylines(tree, line_overlap, &mut rng));
        }
        intersection_open_polylines(&result_lines, limit_to_outline)
    }
}

/// `polygonCollidesWithLineSegment` (Layer.cpp:88) — does the segment `from→to`
/// cross any outline edge within the locator's traversed cells.
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

/// `Geometry::segments_intersect` — proper segment-segment crossing test.
fn segments_intersect(p1: Point, p2: Point, q1: Point, q2: Point) -> bool {
    let d = |o: Point, a: Point, b: Point| {
        (a.x() - o.x()) as i128 * (b.y() - o.y()) as i128
            - (a.y() - o.y()) as i128 * (b.x() - o.x()) as i128
    };
    let d1 = d(q1, q2, p1);
    let d2 = d(q1, q2, p2);
    let d3 = d(p1, p2, q1);
    let d4 = d(p1, p2, q2);
    ((d1 > 0) != (d2 > 0)) && ((d3 > 0) != (d4 > 0))
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
