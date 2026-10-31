// AABBTreeIndirect.hpp:129–218,228–242, as instantiated by
// ClipperUtils.cpp:c2_attach_stray_holes. Preserve leaf traversal and equal-area
// candidate order rather than substituting Rust's nth-element partition.
use super::c2;
use crate::geometry::CoordinateScale;

struct Item {
    bounds: c2::Rect64,
    index: usize,
}
impl Item {
    fn centroid(&self, axis: usize) -> i64 {
        if axis == 0 {
            (self.bounds.left + self.bounds.right) / 2
        } else {
            (self.bounds.top + self.bounds.bottom) / 2
        }
    }
}
enum Node {
    Leaf {
        bounds: c2::Rect64,
        index: usize,
    },
    Branch {
        bounds: c2::Rect64,
        children: [usize; 2],
    },
}
impl Node {
    fn bounds(&self) -> c2::Rect64 {
        match self {
            Self::Leaf { bounds, .. } | Self::Branch { bounds, .. } => *bounds,
        }
    }
}
pub(super) struct Index {
    nodes: Vec<Node>,
    root: Option<usize>,
}
impl Index {
    pub(super) fn new(bounds: &[c2::Rect64], scale: CoordinateScale) -> Self {
        let mut tree = Self {
            nodes: Vec::with_capacity(bounds.len() * 2),
            root: None,
        };
        let epsilon = match scale {
            CoordinateScale::Normal => 100,
            CoordinateScale::LargeBed => 10,
        };
        let mut items: Vec<_> = bounds
            .iter()
            .copied()
            .enumerate()
            .map(|(index, mut bounds)| {
                // BoundingBoxWrapper uses coord_t(SCALED_EPSILON), not exact bounds.
                bounds.left -= epsilon;
                bounds.top -= epsilon;
                bounds.right += epsilon;
                bounds.bottom += epsilon;
                Item { index, bounds }
            })
            .collect();
        if !items.is_empty() {
            tree.root = Some(tree.build(&mut items));
        }
        tree
    }
    fn build(&mut self, items: &mut [Item]) -> usize {
        let node = if items.len() == 1 {
            Node::Leaf {
                bounds: items[0].bounds,
                index: items[0].index,
            }
        } else {
            let bounds = union_bounds(items);
            let axis = usize::from(bounds.bottom - bounds.top > bounds.right - bounds.left);
            let middle = (items.len() - 1) / 2;
            partition(items, axis, middle);
            let (left, right) = items.split_at_mut(middle + 1);
            Node::Branch {
                bounds,
                children: [self.build(left), self.build(right)],
            }
        };
        let index = self.nodes.len();
        self.nodes.push(node);
        index
    }
    pub(super) fn candidates(&self, bounds: c2::Rect64, output: &mut Vec<usize>) {
        if let Some(root) = self.root {
            self.visit(root, bounds, output);
        }
    }
    fn visit(&self, node: usize, bounds: c2::Rect64, output: &mut Vec<usize>) {
        if !self.nodes[node].bounds().contains_rect(&bounds) {
            return;
        }
        match self.nodes[node] {
            Node::Leaf { index, .. } => output.push(index),
            Node::Branch { children, .. } => {
                self.visit(children[0], bounds, output);
                self.visit(children[1], bounds, output);
            }
        }
    }
}
fn union_bounds(items: &[Item]) -> c2::Rect64 {
    let mut bounds = items[0].bounds;
    for item in &items[1..] {
        let b = item.bounds;
        bounds.left = bounds.left.min(b.left);
        bounds.top = bounds.top.min(b.top);
        bounds.right = bounds.right.max(b.right);
        bounds.bottom = bounds.bottom.max(b.bottom);
    }
    bounds
}

// AABBTreeIndirect.hpp:162–218. Median-of-three and pivot restoration are
// separate functions only to bound Rust nesting; swaps/comparisons retain order.
fn partition(items: &mut [Item], axis: usize, k: usize) {
    let (mut left, mut right) = (0, items.len() - 1);
    while left < right {
        let center = (left + right) / 2;
        let pivot = median(items, axis, left, center, right);
        if right <= left + 2 {
            break;
        }
        let i = split(items, axis, left, right, pivot);
        if k < i {
            right = i - 1;
        } else if k == i {
            break;
        } else {
            left = i + 1;
        }
    }
}
fn median(items: &mut [Item], axis: usize, left: usize, center: usize, right: usize) -> i64 {
    let mut left_value = items[left].centroid(axis);
    let mut center_value = items[center].centroid(axis);
    let mut right_value = items[right].centroid(axis);
    if left_value > center_value {
        items.swap(left, center);
        std::mem::swap(&mut left_value, &mut center_value);
    }
    if left_value > right_value {
        items.swap(left, right);
        right_value = left_value;
    }
    if center_value > right_value {
        items.swap(center, right);
        center_value = right_value;
    }
    center_value
}
fn split(items: &mut [Item], axis: usize, left: usize, right: usize, pivot: i64) -> usize {
    let center = (left + right) / 2;
    let (mut i, mut j) = (left, right - 1);
    items.swap(center, j);
    loop {
        i += 1;
        while items[i].centroid(axis) < pivot {
            i += 1;
        }
        j -= 1;
        while items[j].centroid(axis) > pivot && i < j {
            j -= 1;
        }
        if i >= j {
            break;
        }
        items.swap(i, j);
    }
    items.swap(i, right - 1);
    i
}
