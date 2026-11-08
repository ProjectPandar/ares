//! OrcaSlicer 2.4.2 `Fill/FillAdaptive.cpp` port (adaptivecubic /
//! supportcubic sparse infill).

pub(crate) mod filler;
pub(crate) mod hooks;
mod multiline;
pub(crate) mod octree;

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub(crate) enum OctreeKind {
    Adaptive,
    Support,
}

/// Per-slice octree cache keyed by (traversal object index, line-spacing
/// bits, source tree kind). PrintObject.cpp:1006-1007 owns separate trees.
/// Lives on the slice context (`PreparedPostClassicTraversal`), so
/// every slice API call works with its own trees — repeated calls with
/// multiple projects never share or leak cache entries across calls, and
/// nothing needs a global reset. Interior mutability: the fill phases all
/// borrow the traversal shared on one thread.
pub(crate) struct AdaptiveOctreeCache(
    RefCell<HashMap<(usize, u64, OctreeKind), Arc<octree::Octree>>>,
);

impl AdaptiveOctreeCache {
    pub(crate) fn new() -> Self {
        Self(RefCell::new(HashMap::new()))
    }

    pub(crate) fn get_or_build(
        &self,
        key: (usize, u64, OctreeKind),
        build: impl FnOnce() -> octree::Octree,
    ) -> Arc<octree::Octree> {
        let mut cache = self.0.borrow_mut();
        cache
            .entry(key)
            .or_insert_with(|| Arc::new(build()))
            .clone()
    }

    pub(crate) fn get_or_try_build<E>(
        &self,
        key: (usize, u64, OctreeKind),
        build: impl FnOnce() -> Result<octree::Octree, E>,
    ) -> Result<Arc<octree::Octree>, E> {
        use std::collections::hash_map::Entry;

        match self.0.borrow_mut().entry(key) {
            Entry::Occupied(entry) => Ok(entry.get().clone()),
            Entry::Vacant(entry) => Ok(entry.insert(Arc::new(build()?)).clone()),
        }
    }

    pub(crate) fn get(&self, key: (usize, u64, OctreeKind)) -> Arc<octree::Octree> {
        self.0.borrow()[&key].clone()
    }
}
