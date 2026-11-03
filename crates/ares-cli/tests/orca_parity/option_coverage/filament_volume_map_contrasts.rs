//! Native contrast witnesses for static ToolOrdering volume-map state.

#[test]
fn volume_map_high_flow_labelled_section() {
    super::filament_volume_map::check_labelled(
        "high-flow",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/parity/filament-volume-map/high-flow.3mf"
        )),
        "6c61b5ca2432a4cd8793fe7c8288b85243a7c303310b5328634337ac0e302a06",
        [
            ("; filament_map = ", "1,1"),
            ("; filament_volume_map = ", "0,1"),
            ("; filament_nozzle_map = ", "0,0"),
        ],
    );
}

#[test]
fn volume_map_unused_assignment_labelled_section() {
    super::filament_volume_map::check_labelled(
        "unused-assignment",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/parity/filament-volume-map/unused-assignment.3mf"
        )),
        "1f001db69a4803b6f6f37f12d79dea42a23d156c4fe089e10bada97492089768",
        [
            ("; filament_map = ", "1,1"),
            ("; filament_volume_map = ", "1,0"),
            ("; filament_nozzle_map = ", "0,0"),
        ],
    );
}
