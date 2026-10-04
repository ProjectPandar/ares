//! `FillLightning::Layer` grounding/reconnect methods + geometry helpers
//! (`Layer.cpp`), split from `layer.rs` for the <400 LOC rule.

use std::rc::Rc;

use crate::geometry::{BoundingBox, Coord, EdgeGrid, Point, Polygon};

use super::layer::{GroundingLocation, Layer, SparseNodeGrid, to_grid_point};
use super::tree_node::Node;
use super::tree_node::NodeSPtr;

pub(super) struct GroundingContext<'a> {
    pub(super) outlines: &'a [Polygon],
    pub(super) bbox: BoundingBox,
    pub(super) locator: &'a EdgeGrid,
    pub(super) supporting_radius: Coord,
    pub(super) wall_supporting_radius: Coord,
}

pub(super) fn get_best_grounding_location(
    unsupported_location: Point,
    context: &GroundingContext<'_>,
    tree_node_locator: &SparseNodeGrid,
    exclude_tree: Option<&NodeSPtr>,
    locator_cell_size: Coord,
) -> GroundingLocation {
    let &GroundingContext {
        outlines: current_outlines,
        bbox: current_outlines_bbox,
        locator: outline_locator,
        supporting_radius,
        wall_supporting_radius,
    } = context;
    // Closest point on current_outlines to the unsupported location.
    let mut node_location = Point::new(0, 0);
    let mut d2 = f64::MAX;
    for contour in current_outlines {
        let pts = contour.points();
        if pts.len() <= 2 {
            continue;
        }
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
    let within_dist = dist(node_location, unsupported_location);
    let mut current_dist = get_weighted_distance_boundary(node_location, unsupported_location);
    let mut sub_tree: Option<NodeSPtr> = None;

    if current_dist < wall_supporting_radius {
        return GroundingLocation {
            tree_node: None,
            boundary_location: Some(node_location),
        };
    }
    let search_radius = current_dist.min(within_dist);
    let cell = locator_cell_size;
    let rmin = to_grid_point(
        Point::new(
            unsupported_location.x() - search_radius,
            unsupported_location.y() - search_radius,
        ),
        current_outlines_bbox,
        cell,
    );
    let rmax = to_grid_point(
        Point::new(
            unsupported_location.x() + search_radius + cell,
            unsupported_location.y() + search_radius + cell,
        ),
        current_outlines_bbox,
        cell,
    );
    // Serial scan (upstream runs a deterministic-order parallel_for
    // that resolves ties by grid address — the serial scan is the same
    // order).
    let mut best_addr = (i64::MAX, i64::MAX);
    for (gy, gx) in (rmin.y()..rmax.y()).flat_map(|gy| (rmin.x()..rmax.x()).map(move |gx| (gy, gx)))
    {
        let addr = Point::new(gx, gy);
        for weak in tree_node_locator.equal_range(addr) {
            let Some(candidate) = weak.upgrade() else {
                continue;
            };
            if let Some(ex) = exclude_tree
                && (Rc::ptr_eq(&candidate, ex) || Node::has_offspring(ex, &candidate))
            {
                continue;
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
            // Upstream `candidate_dist < local_current_dist`
            // (Layer.cpp:171) is STRICT: a tree tied with the
            // initial boundary distance never wins; the grid
            // tie-break only ranks already-winning candidates.
            if cd < current_dist
                || (cd == current_dist && sub_tree.is_some() && (gy, gx) < best_addr)
            {
                current_dist = cd;
                sub_tree = Some(candidate);
                best_addr = (gy, gx);
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

impl Layer {
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
        context: &GroundingContext<'_>,
    ) {
        let mut tree_node_locator = SparseNodeGrid::new();
        self.fill_locator(&mut tree_node_locator, context.bbox);
        let within_max_dist = context.locator.resolution() * 2;
        for root_ptr in to_be_reconnected {
            self.reconnect_root(root_ptr, context, &mut tree_node_locator, within_max_dist);
        }
    }

    fn reconnect_root(
        &mut self,
        root_ptr: NodeSPtr,
        context: &GroundingContext<'_>,
        tree_node_locator: &mut SparseNodeGrid,
        within_max_dist: Coord,
    ) {
        const TREE_CONNECTING_IGNORE_OFFSET: Coord = 100;
        let &GroundingContext {
            bbox: current_outlines_bbox,
            locator: outline_locator,
            wall_supporting_radius,
            ..
        } = context;
        let old_root_pos = self
            .tree_roots
            .iter()
            .position(|r| Rc::ptr_eq(r, &root_ptr));

        // Copy the root's point + last-grounding into locals BEFORE any
        // `borrow_mut` (upstream `root_ptr->getLocation()`/`getLastGroundingLocation()`
        // are plain reads; holding a `Ref` across `add_child_node`'s
        // `root_ptr.borrow_mut()` would panic).
        let (root_p, last_grounding) = {
            let r = root_ptr.borrow();
            (r.p, r.last_grounding_location)
        };
        if let Some(ground_loc) = last_grounding
            && ground_loc != root_p
            && let Some(new_root_pt) = super::tree_geom::line_segment_polygons_intersection(
                root_p,
                ground_loc,
                outline_locator,
                within_max_dist,
            )
        {
            let new_root = Node::create_grounded(new_root_pt, Some(new_root_pt));
            // `root_ptr->addChild(new_root); new_root->reroot();`
            // (Layer.cpp:248-250): new_root becomes a CHILD of the
            // old root, then `reroot(nullptr)` flips the edge —
            // new_root ends up the ROOT with the entire old tree
            // beneath it (depth +1, subtree preserved).
            Node::add_child_node(&root_ptr, new_root.clone());
            Node::reroot(&new_root, None);
            tree_node_locator.insert(
                to_grid_point(
                    new_root.borrow().p,
                    current_outlines_bbox,
                    self.locator_cell_size,
                ),
                &new_root,
            );
            if let Some(pos) = old_root_pos {
                self.tree_roots[pos] = new_root;
            }
            return;
        }

        let tree_connecting_ignore_width = wall_supporting_radius - TREE_CONNECTING_IGNORE_OFFSET;
        let ground = get_best_grounding_location(
            root_ptr.borrow().p,
            &GroundingContext {
                wall_supporting_radius: tree_connecting_ignore_width,
                ..*context
            },
            tree_node_locator,
            Some(&root_ptr),
            self.locator_cell_size,
        );
        if let Some(boundary) = ground.boundary_location {
            if boundary == root_ptr.borrow().p {
                return;
            }
            let new_root = Node::create_grounded(boundary, Some(boundary));
            let attach_ptr = Node::closest_node(&root_ptr, new_root.borrow().p);
            Node::reroot(&attach_ptr, None);
            Node::add_child_node(&new_root, attach_ptr);
            tree_node_locator.insert(
                to_grid_point(
                    new_root.borrow().p,
                    current_outlines_bbox,
                    self.locator_cell_size,
                ),
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

mod geometry;

use geometry::{dist, get_weighted_distance_boundary, polygon_collides_with_line, segment_closest};
