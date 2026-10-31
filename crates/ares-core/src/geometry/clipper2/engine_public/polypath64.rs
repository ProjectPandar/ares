// Copyright (c) Angus Johnson 2010-2025.
// BSL-1.0. Rust seed by MatterHackers.
// Derived from clipper2-rust 2f5e9d4249bc4d624773094ff6164335d9ff6ae7.
// See ../LICENSE-Boost-1.0 and ../SOURCE.md for pinned source authority.

use super::*;

/// PolyPath64 - Integer coordinate polytree node
/// Direct port from clipper.engine.h line 334
#[derive(Debug, Clone)]
pub struct PolyPath64 {
    pub(crate) parent: Option<usize>, // index into polytree arena
    pub(crate) children: Vec<usize>,  // indices into polytree arena
    pub(crate) polygon: Path64,
}

impl PolyPath64 {
    pub fn new() -> Self {
        Self {
            parent: None,
            children: Vec::new(),
            polygon: Path64::new(),
        }
    }
}

impl PolyPath64 {
    pub fn with_parent(parent_idx: usize) -> Self {
        Self {
            parent: Some(parent_idx),
            children: Vec::new(),
            polygon: Path64::new(),
        }
    }
}

impl PolyPath64 {
    pub fn with_path(parent_idx: usize, path: Path64) -> Self {
        Self {
            parent: Some(parent_idx),
            children: Vec::new(),
            polygon: path,
        }
    }
}

impl PolyPath64 {
    pub fn polygon(&self) -> &Path64 {
        &self.polygon
    }
}

impl PolyPath64 {
    pub fn count(&self) -> usize {
        self.children.len()
    }
}

impl PolyPath64 {
    pub fn parent(&self) -> Option<usize> {
        self.parent
    }
}

impl PolyPath64 {
    pub fn children(&self) -> &[usize] {
        &self.children
    }
}

impl Default for PolyPath64 {
    fn default() -> Self {
        Self::new()
    }
}

/// PolyTree64 arena - stores all PolyPath64 nodes
/// Direct port from clipper.engine.h PolyTree64
pub struct PolyTree64 {
    pub nodes: Vec<PolyPath64>,
}

impl PolyTree64 {
    pub fn new() -> Self {
        // Root node is always at index 0
        let root = PolyPath64::new();
        Self { nodes: vec![root] }
    }
}

impl PolyTree64 {
    /// Add a child path to a parent node
    pub fn add_child(&mut self, parent_idx: usize, path: Path64) -> usize {
        let child_idx = self.nodes.len();
        self.nodes.push(PolyPath64::with_path(parent_idx, path));
        self.nodes[parent_idx].children.push(child_idx);
        child_idx
    }
}

impl PolyTree64 {
    /// Get the root node
    pub fn root(&self) -> &PolyPath64 {
        &self.nodes[0]
    }
}

impl PolyTree64 {
    /// Clear the tree
    pub fn clear(&mut self) {
        self.nodes.clear();
        self.nodes.push(PolyPath64::new());
    }
}

impl PolyTree64 {
    /// Get level of a node
    pub fn level(&self, node_idx: usize) -> u32 {
        let mut result = 0u32;
        let mut p = self.nodes[node_idx].parent;
        while let Some(parent_idx) = p {
            result += 1;
            p = self.nodes[parent_idx].parent;
        }
        result
    }
}

impl PolyTree64 {
    /// Check if a node is a hole
    pub fn is_hole(&self, node_idx: usize) -> bool {
        let lvl = self.level(node_idx);
        lvl > 0 && (lvl & 1) == 0
    }
}

impl PolyTree64 {
    /// Get total area including children
    pub fn area_of(&self, node_idx: usize) -> f64 {
        let mut result = area(&self.nodes[node_idx].polygon);
        for &child_idx in &self.nodes[node_idx].children {
            result += self.area_of(child_idx);
        }
        result
    }
}

impl Default for PolyTree64 {
    fn default() -> Self {
        Self::new()
    }
}
