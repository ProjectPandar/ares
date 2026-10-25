//! Thin-gap-j1 IDEX nearest-seam embedding layer gate witnesses.

#[test]
fn wall_arachne_matches_native() {
    check(
        "wall-arachne",
        include_bytes!("../../../../tests/parity/thin-gap-j1-seam-embedding/wall-arachne.3mf"),
        "5f822b0761a325deb15795f82dedb9a7d03a8c40304ed389edaf4e4543b351c7",
    );
}

#[test]
fn wall_classic_matches_native() {
    check(
        "wall-classic",
        include_bytes!("../../../../tests/parity/thin-gap-j1-seam-embedding/wall-classic.3mf"),
        "482eb2e6a19b43a56cc13553dc9c8795ca9d4ccf9510743d45d236b547cab01d",
    );
}

fn check(label: &str, project: &[u8], expected_sha: &str) {
    crate::runner::native_stream_case::check(
        (
            "thin-gap-j1-seam-embedding",
            &[
                "GCode/SeamPlacer.cpp:1071-1077 regions_with_perimeter counts region->perimeters.entities.size()>0",
                "GCode/SeamPlacer.cpp:1095-1097 populate_embedded_distance runs only on counted layers",
                "GCode/SeamPlacer.cpp:770-775 SeamComparator hidden embedded gate",
                "LayerRegion.cpp:82-84,119-122 perimeters filled only by the generator",
                "PerimeterGenerator.cpp:1568-1569,229-232,1613-1621,2128,2471-2476 append boundary",
            ],
        ),
        label,
        project,
        expected_sha,
        (true, |_, _| {}),
    );
}
