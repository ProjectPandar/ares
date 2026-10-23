//! Print-object toolpath sharing (`Print.cpp:2200-2238`).

impl super::ProjectObject {
    /// Print.cpp:2200-2238 `is_print_object_the_same`: objects with
    /// identical volume meshes (loader pointer identity), volume
    /// transforms, painted facets, and overrides slice once and share the
    /// layer entities (`Print.cpp:2247` `set_shared_object`, `:2103`
    /// `copy_layers_from_shared_object`).
    pub(crate) fn shares_print_toolpaths(&self, other: &Self) -> bool {
        self.layer_height_profile == other.layer_height_profile
            && self.object_overrides == other.object_overrides
            && self.volumes.len() == other.volumes.len()
            && self
                .volumes
                .iter()
                .zip(&other.volumes)
                .all(|(left, right)| {
                    left.volume_type == right.volume_type
                        && left.mesh.geometry_identity() == right.mesh.geometry_identity()
                        && left.transform == right.transform
                        && left.mesh.supported_facets == right.mesh.supported_facets
                        && left.mesh.mmu_segmentation_facets == right.mesh.mmu_segmentation_facets
                        && left.mesh.fuzzy_skin_facets == right.mesh.fuzzy_skin_facets
                        && left.region_overrides == right.region_overrides
                })
    }
}
