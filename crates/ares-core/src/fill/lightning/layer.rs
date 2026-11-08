//! `FillLightning::Layer` (`src/libslic3r/Fill/Lightning/Layer.cpp`): a layer
//! of lightning trees. `generateNewTrees` seeds branches from the
//! `DistanceField` unsupported cells; `reconnectRoots` re-grounds propagated
//! roots; `convertToLines` emits the tree polylines clipped to the outline.
//! The node locator is a `SparseNodeGrid` (`unordered_multimap` keyed by the
//! grid point) — ported as `HashMap<Point, Vec<Weak<Node>>>`.

use std::collections::HashMap;
use std::rc::{Rc, Weak};

use crate::geometry::{
    BoundingBox, ClipperError, Coord, Point, Polygon, Polyline, intersection_open_polylines,
};

use super::distance_field::DistanceField;
use super::layer_geom::{GroundingContext, get_best_grounding_location};
use super::tree_node::{Node, NodeSPtr};

/// `locator_cell_size` — `scaled<coord_t>(4.)` (`TreeNode.hpp:21`), derived
/// from the coordinate scale: `4 / NORMAL_SCALE` = 4,000,000 at the default
/// scale, `4 / LARGE_BED_SCALE` = 400,000 on large beds.
pub(super) fn locator_cell_size(scale: crate::geometry::CoordinateScale) -> Coord {
    scale.checked_scale(4.0).unwrap_or(4_000_000)
}

/// `SparseNodeGrid` — grid-point → node weak refs (unordered_multimap port).
pub(super) struct SparseNodeGrid {
    map: HashMap<(Coord, Coord), Vec<Weak<std::cell::RefCell<Node>>>>,
}

impl SparseNodeGrid {
    pub(super) fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }
    pub(super) fn insert(&mut self, grid_addr: Point, node: &NodeSPtr) {
        self.map
            .entry((grid_addr.x(), grid_addr.y()))
            .or_default()
            .push(Rc::downgrade(node));
    }
    /// No-hint `unordered_multimap` insertion prepends equivalent keys.
    pub(super) fn equal_range(
        &self,
        grid_addr: Point,
    ) -> impl Iterator<Item = &Weak<std::cell::RefCell<Node>>> {
        self.map
            .get(&(grid_addr.x(), grid_addr.y()))
            .map(Vec::as_slice)
            .unwrap_or(&[])
            .iter()
            .rev()
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
    pub(super) fn p(&self) -> Point {
        match &self.tree_node {
            Some(n) => n.borrow().p,
            None => self.boundary_location.unwrap(),
        }
    }
}

/// `to_grid_point` — `(point - bbox.min) / locator_cell_size`.
pub(super) fn to_grid_point(p: Point, bbox: BoundingBox, cell_size: Coord) -> Point {
    Point::new(
        (p.x() - bbox.min().x()) / cell_size,
        (p.y() - bbox.min().y()) / cell_size,
    )
}

/// `FillLightning::Layer`.
pub(super) struct Layer {
    pub tree_roots: Vec<NodeSPtr>,
    /// `locator_cell_size()` — this layer's SparseNodeGrid cell size.
    pub(super) locator_cell_size: Coord,
}

impl Layer {
    pub(super) fn new(locator_cell_size: Coord) -> Self {
        Self {
            tree_roots: Vec::new(),
            locator_cell_size,
        }
    }

    /// `fillLocator` — populate the node grid with all tree nodes.
    pub(super) fn fill_locator(&self, locator: &mut SparseNodeGrid, bbox: BoundingBox) {
        for tree in &self.tree_roots {
            let tree = tree.clone();
            Node::visit_nodes(&tree, &mut |node| {
                locator.insert(
                    to_grid_point(node.borrow().p, bbox, self.locator_cell_size),
                    node,
                );
            });
        }
    }

    /// `generateNewTrees` — grow branches to support every unsupported cell.
    pub(super) fn generate_new_trees(
        &mut self,
        current_overhang: &[Polygon],
        context: &GroundingContext<'_>,
    ) -> Result<(), ClipperError> {
        let &GroundingContext {
            bbox: current_outlines_bbox,
            supporting_radius,
            ..
        } = context;
        let mut distance_field =
            DistanceField::new(supporting_radius, current_outlines_bbox, current_overhang)?;

        let mut tree_node_locator = SparseNodeGrid::new();
        self.fill_locator(&mut tree_node_locator, current_outlines_bbox);

        let mut unsupported_cell_idx = 0usize;
        while let Some((cell_idx, unsupported_location)) =
            distance_field.try_get_next_point(unsupported_cell_idx)
        {
            unsupported_cell_idx = cell_idx;
            let grounding = get_best_grounding_location(
                unsupported_location,
                context,
                &tree_node_locator,
                None,
                self.locator_cell_size,
            );
            let (new_child, new_parent) = self.attach(unsupported_location, &grounding);
            tree_node_locator.insert(
                to_grid_point(
                    new_child.borrow().p,
                    current_outlines_bbox,
                    self.locator_cell_size,
                ),
                &new_child,
            );
            if let Some(np) = new_parent {
                tree_node_locator.insert(
                    to_grid_point(np.borrow().p, current_outlines_bbox, self.locator_cell_size),
                    &np,
                );
            }
            distance_field.update(grounding.p(), unsupported_location);
        }
        Ok(())
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
        let mut result_lines = Vec::new();
        for tree in &self.tree_roots {
            result_lines.extend(Node::convert_to_polylines(tree, line_overlap));
        }
        intersection_open_polylines(&result_lines, limit_to_outline)
    }
}
