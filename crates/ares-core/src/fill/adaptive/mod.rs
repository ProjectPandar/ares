//! OrcaSlicer 2.4.2 `Fill/FillAdaptive.cpp` port (adaptivecubic /
//! supportcubic sparse infill).

pub(crate) mod filler;
pub(crate) mod hooks;
pub(crate) mod octree;

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

/// Per-slice octree cache keyed by (traversal object index, line-spacing
/// bits). Lives on the slice context (`PreparedPostClassicTraversal`), so
/// every slice API call works with its own trees — repeated calls with
/// multiple projects never share or leak cache entries across calls, and
/// nothing needs a global reset. Interior mutability: the fill phases all
/// borrow the traversal shared on one thread.
pub(crate) struct AdaptiveOctreeCache(RefCell<HashMap<(usize, u64), Arc<octree::Octree>>>);

impl AdaptiveOctreeCache {
    pub(crate) fn new() -> Self {
        Self(RefCell::new(HashMap::new()))
    }

    pub(crate) fn get_or_build(
        &self,
        key: (usize, u64),
        build: impl FnOnce() -> octree::Octree,
    ) -> Arc<octree::Octree> {
        let mut cache = self.0.borrow_mut();
        cache
            .entry(key)
            .or_insert_with(|| Arc::new(build()))
            .clone()
    }
}
