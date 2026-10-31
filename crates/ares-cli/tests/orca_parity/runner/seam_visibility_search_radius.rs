//! `SeamPlacer.cpp:685` search radius promotion via public profile slicing.

#[test]
fn seam_visibility_search_radius_complete_output() {
    super::seam_visibility_eigen_distance::check(
        "seam-visibility-search-radius",
        "first-layer-hole",
        true,
    );
}
