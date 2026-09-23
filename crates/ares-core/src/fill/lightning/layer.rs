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
/// uses `locator_cell_size()`); upstream `TreeNode.hpp:21`
/// `locator_cell_size() = scaled<coord_t>(4.)` = `4 / NORMAL_SCALE` = 4,000,000
/// at the default 1e6 coordinate scale.
pub(super) const LOCATOR_CELL_SIZE: Coord = 4_000_000;

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
    /// `equal_range` — the candidate nodes at a grid address.
    pub(super) fn equal_range(&self, grid_addr: Point) -> &[Weak<std::cell::RefCell<Node>>] {
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
    pub(super) fn p(&self) -> Point {
        match &self.tree_node {
            Some(n) => n.borrow().p,
            None => self.boundary_location.unwrap(),
        }
    }
}

/// `to_grid_point` — `(point - bbox.min) / locator_cell_size`.
pub(super) fn to_grid_point(p: Point, bbox: BoundingBox) -> Point {
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
    pub(super) fn fill_locator(&self, locator: &mut SparseNodeGrid, bbox: BoundingBox) {
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
            if std::env::var("ARES_DEBUG_LIGHTNING").is_ok() {
                eprintln!(
                    "ATTACH unsupported=({},{}) boundary={} tree={}",
                    unsupported_location.x(),
                    unsupported_location.y(),
                    grounding.boundary_location.is_some(),
                    grounding.tree_node.is_some()
                );
            }
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

    /// `convertToLines` — emit the tree polylines clipped to the outline.
    pub(super) fn convert_to_lines(
        &self,
        limit_to_outline: &[Polygon],
        line_overlap: Coord,
        rng: &mut GlibcRand,
    ) -> Result<Vec<Polyline>, ClipperError> {
        if self.tree_roots.is_empty() {
            return Ok(Vec::new());
        }
        let mut result_lines = Vec::new();
        for tree in &self.tree_roots {
            if std::env::var("ARES_DEBUG_LIGHTNING").is_ok() {
                let (tp, ch): ((i64, i64), Vec<(i64, i64)>) = {
                    let n = tree.borrow();
                    (
                        (n.p.x(), n.p.y()),
                        n.children
                            .iter()
                            .map(|c| {
                                let b = c.borrow();
                                (b.p.x(), b.p.y())
                            })
                            .collect(),
                    )
                };
                eprintln!(
                    "C2LT root=({},{}) children={:?} overlap={}",
                    tp.0, tp.1, ch, line_overlap
                );
            }
            result_lines.extend(Node::convert_to_polylines(tree, line_overlap, rng));
        }
        if std::env::var("ARES_DEBUG_LIGHTNING").is_ok() {
            let pts: Vec<(i64, i64)> = result_lines
                .iter()
                .flat_map(|l| l.points().iter().map(|p| (p.x(), p.y())))
                .collect();
            let (mut minx, mut miny, mut maxx, mut maxy) = (i64::MAX, i64::MAX, i64::MIN, i64::MIN);
            for (x, y) in &pts {
                minx = minx.min(*x);
                miny = miny.min(*y);
                maxx = maxx.max(*x);
                maxy = maxy.max(*y);
            }
            let obb = limit_to_outline
                .iter()
                .flat_map(|p| p.points().iter().map(|pt| (pt.x(), pt.y())))
                .fold((i64::MAX, i64::MAX, i64::MIN, i64::MIN), |a, (x, y)| {
                    (a.0.min(x), a.1.min(y), a.2.max(x), a.3.max(y))
                });
            eprintln!(
                "C2LP raw_lines={} pts={} tree_bbox=({},{})-({},{}) outline_bbox=({},{})-({},{})",
                result_lines.len(),
                pts.len(),
                minx,
                miny,
                maxx,
                maxy,
                obb.0,
                obb.1,
                obb.2,
                obb.3
            );
        }
        intersection_open_polylines(&result_lines, limit_to_outline)
    }
}
