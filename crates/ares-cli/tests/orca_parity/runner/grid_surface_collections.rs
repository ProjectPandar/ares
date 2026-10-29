//! Fill.cpp:1326-1357 / FillBase.cpp:161-182: Grid surface collection identity.

#[test]
fn k1_bridge_target() {
    check(
        "k1-bridge",
        include_bytes!("../../../../../tests/parity/grid-surface-collections/k1-bridge.3mf"),
        "3750e02942d25b6a96866da9990a6c4bd8bc30bfb0eb990ed64c4f109ab9badb",
    );
}

#[test]
fn mk4_bridge_control() {
    check(
        "mk4-bridge",
        include_bytes!("../../../../../tests/parity/grid-surface-collections/mk4-bridge.3mf"),
        "999f4fc56915b6591b653bf4e1909c85eef6319da306947a73cb1d89aaeeee3b",
    );
}

fn check(label: &str, project: &[u8], expected_sha: &str) {
    crate::runner::native_stream_case::check(
        (
            "grid-surface-collections",
            &[
                "Fill.cpp:1326-1357::Layer::make_fills per-expolygon calls",
                "FillBase.cpp:161-182::Fill::fill_surface_extrusion collection creation",
                "GCode.cpp:5010-5068::whole-collection island assignment",
            ],
        ),
        label,
        project,
        expected_sha,
        (false, |_, _| {}),
    );
}
