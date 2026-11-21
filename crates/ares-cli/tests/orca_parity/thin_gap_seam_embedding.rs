//! Thin-gap-j1 IDEX nearest-seam embedding witnesses.

const CLASSIC: &[u8] =
    include_bytes!("../../../../tests/parity/thin-gap-j1-seam-embedding/wall-classic.3mf");

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
        CLASSIC,
        "482eb2e6a19b43a56cc13553dc9c8795ca9d4ccf9510743d45d236b547cab01d",
    );
}

#[test]
fn wall_classic_with_thin_wall_detection_matches_native() {
    use std::io::{Cursor, Read, Write};

    const SETTINGS: &str = "Metadata/project_settings.config";
    const DISABLED: &str = "\"detect_thin_wall\": \"0\"";
    const ENABLED: &str = "\"detect_thin_wall\": \"1\"";
    let mut source = zip::ZipArchive::new(Cursor::new(CLASSIC)).unwrap();
    let mut project = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..source.len() {
        let mut member = source.by_index(index).unwrap();
        if member.name() == SETTINGS {
            let mut settings = String::new();
            member.read_to_string(&mut settings).unwrap();
            assert_eq!(settings.matches(DISABLED).count(), 1);
            project
                .start_file(SETTINGS, zip::write::SimpleFileOptions::default())
                .unwrap();
            project
                .write_all(settings.replacen(DISABLED, ENABLED, 1).as_bytes())
                .unwrap();
        } else {
            project.raw_copy_file(member).unwrap();
        }
    }
    let project = project.finish().unwrap().into_inner();
    crate::runner::native_stream_case::check(
        (
            "classic-thin-wall",
            &[
                "PerimeterGenerator.cpp:258-278 converts thin walls before variable-width extrusion",
                "Arachne/utils/ExtrusionLine.hpp:212-230 expands junction widths into segment widths",
            ],
        ),
        "detect-thin-wall",
        &project,
        &crate::runner::native_api_cli::sha(&project),
        (true, |_, _| {}),
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
