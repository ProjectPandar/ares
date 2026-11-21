use crate::runner::native_stream_case;

#[test]
fn mk4_arachne_tower_with_disconnected_sampled_loop_matches_native() {
    native_stream_case::check(
        (
            "arachne-disconnected-loop",
            &["PerimeterGenerator.cpp:572-579::traverse_extrusions"],
        ),
        "tower",
        include_bytes!("../../../../tests/parity/arachne-disconnected-loop/tower.3mf"),
        "805cd40cfa306d482b111f31a8ff3fb93fbb25fa270665f9ba115d97b944f08e",
        (false, |_, _| {}),
    );
}
