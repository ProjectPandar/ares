//! Same-height bottom contact and top interface through public slicing streams.

#[test]
fn grid_interface_retains_connector_on_x1c() {
    check(
        "x1c-grid-interface",
        include_bytes!("../../../../tests/parity/support-contact-overlap/x1c-grid-interface.3mf"),
        "51ee28a8b08f5de32219f96a4e411ed42ecffd17d0f5184ade5536a43a9ded05",
    );
}

#[test]
fn two_layer_raft_retains_support_island_hole_on_x1c() {
    check(
        "x1c-two-layer-raft",
        include_bytes!("../../../../tests/parity/support-contact-overlap/x1c-two-layer-raft.3mf"),
        "6da4a987e7739d9ddd14b9da9605c0ebf2abf6a2028ad909c941b92277ae38cc",
    );
}

#[test]
fn rectilinear_grid_base_retains_support_island_hole_on_k1() {
    check(
        "k1-rectilinear-grid-base",
        include_bytes!(
            "../../../../tests/parity/support-contact-overlap/k1-rectilinear-grid-base.3mf"
        ),
        "f9ac3bfb51c097552633e3da64eccae53d8d39d6206ca1edb79fbebb00fab057",
    );
}

fn check(label: &str, project: &[u8], sha: &str) {
    crate::runner::native_stream_case::check(
        (
            "support-contact-overlap",
            &["Support/SupportCommon.cpp:1805-1812::generate_support_toolpaths"],
        ),
        label,
        project,
        sha,
        (false, |_, _| {}),
    );
}
