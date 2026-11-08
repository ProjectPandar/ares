//! `FillLightning::Node` (`src/libslic3r/Fill/Lightning/TreeNode.cpp`): the
//! lightning-tree vertex — a `shared_ptr`/`weak_ptr` parent-children tree with
//! `deepCopy`/`reroot`/`realign`/`straighten`/`prune`/`convertToPolylines`/
//! `removeJunctionOverlap`. Ported to `Rc<RefCell<Node>>` + `Weak` parents.
//! `convertToPolylines` picks the first child from a generator seeded per tree.

use std::cell::RefCell;
use std::rc::{Rc, Weak};

use crate::geometry::{Coord, EdgeGrid, Point, Polygon};

use super::tree_geom::dist;

pub(super) type NodeSPtr = Rc<RefCell<Node>>;

pub(super) struct PropagationSettings {
    pub(super) prune_distance: Coord,
    pub(super) smooth_magnitude: Coord,
    pub(super) max_remove_colinear_dist: Coord,
}

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
        let (is_root, parent) = {
            let n = this.borrow();
            (n.is_root, n.parent.upgrade())
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
        settings: &PropagationSettings,
    ) {
        let &PropagationSettings {
            prune_distance,
            smooth_magnitude,
            max_remove_colinear_dist,
        } = settings;
        let tree_below = Self::deep_copy(this);
        Self::prune(&tree_below, prune_distance);
        Self::straighten(&tree_below, smooth_magnitude, max_remove_colinear_dist);
        let ok = Self::realign(&tree_below, next_outlines, outline_locator, next_trees);
        if ok {
            next_trees.push(tree_below);
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
                continue;
            }
            let a = this.borrow().p;
            let b = child.borrow().p;
            let ab_len = dist(a, b);
            if dist_pruned_child + ab_len <= pruning_distance {
                // prune the whole child
                max_pruned = max_pruned.max(dist_pruned_child + ab_len);
                // don't keep
                continue;
            }
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
                // `n = b + (ba.normalized()*remain).cast<coord_t>()`
                // (TreeNode.cpp:338): the displacement vector is
                // componentwise-truncated, THEN added to `b` — not
                // `b + disp` rounded.
                // `b + (disp.cast<coord_t>())` — coord_t add.
                Point::new(
                    b.x() + (dx / nlen * remain as f64) as Coord,
                    b.y() + (dy / nlen * remain as f64) as Coord,
                )
            };
            child.borrow_mut().p = n;
            max_pruned = max_pruned.max(pruning_distance);
            keep.push(child.clone());
        }
        this.borrow_mut().children = keep;
        max_pruned
    }
}
