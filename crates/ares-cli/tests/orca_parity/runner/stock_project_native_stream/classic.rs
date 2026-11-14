//! Runtime classic records for GCodeWriter / GCode::do_export comparator contracts.
//! All eleven body anchors and 33 change/delete/reorder assertions stay unchanged.

use super::producer;
use crate::{
    golden::ordered_bytes_tests::mutations,
    runner::{ExportedCase, cache_identity, repo_root},
};
use std::fs;

#[test]
fn gcode_writer_and_do_export_real_records_reject_33_mutations() {
    let Some(runner) = producer::runner(
        "runner::stock_project_native_stream::classic::gcode_writer_and_do_export_real_records_reject_33_mutations",
    ) else {
        return;
    };
    let source = repo_root().join("tests/parity/replay/ender3-classic.3mf");
    let project = fs::read(&source).unwrap();
    assert_eq!(
        cache_identity::raw_digest(&project),
        "abd8882f86e789f727fe362a11e3b57eb52e846c712b7a0b102b0a4fe3d05bc3",
        "{:?}",
        runner.work,
    );
    let project_path = runner.work.join(source.file_name().unwrap());
    fs::write(&project_path, &project).unwrap();
    let output_dir = runner.work.join("native");
    fs::create_dir(&output_dir).unwrap();
    let case = runner
        .slice_case(&ExportedCase {
            label: "ksr/classic-records".to_owned(),
            project,
            project_path,
            output_dir,
        })
        .unwrap();
    fs::write(runner.work.join("orca.gcode"), &case.reference).unwrap();
    let classic = String::from_utf8(case.reference).unwrap();
    for (label, first, second, changed) in [
        (
            "XY travel",
            "G1 X2 Y10 F3000\n",
            "G1 X2.3 F5000\n",
            "G1 X2 Y10.001 F3000\n",
        ),
        ("Z", "G1 Z.6\n", "G1 Z.2\n", "G1 Z.601\n"),
        (
            "E retract",
            "G1 E-4 F3600\n",
            "G1 E4 F2400\n",
            "G1 E-4.001 F3600\n",
        ),
        ("F", "G1 F3000\n", "G1 F1800\n", "G1 F3001\n"),
        (
            "acceleration",
            "M201 X500 Y500 Z100 E5000\n",
            "M204 P500 R1000 T500\n",
            "M201 X501 Y500 Z100 E5000\n",
        ),
        (
            "temperature",
            "M190 S60 ; wait for bed temp to stabilize\n",
            "M109 S220 ; wait for nozzle temp to stabilize\n",
            "M140 S60 ; wait for bed temp to stabilize\n",
        ),
        ("fan", "M106 S0\n", "M106 S255\n", "M106 S1\n"),
        (
            "lifecycle",
            "G90 ; use absolute coordinates\n",
            "M83 ; extruder relative mode\n",
            "G91 ; use absolute coordinates\n",
        ),
        ("M73", "M73 P4 R8\n", "M73 P6 R8\n", "M73 P2 R8\n"),
        (
            "config",
            // OrcaSlicer 40ecc66f7 PrintConfig.cpp:256-261,4426-4444.
            "; enable_arc_fitting = disabled\n",
            "; enable_extra_bridge_layer = disabled\n",
            "; enable_arc_fitting = enabled\n",
        ),
        (
            "statistics",
            "; filament used [mm] = 262.34\n",
            "; estimated printing time (normal mode) = 8m 55s\n",
            "; filament used [mm] = 262.35\n",
        ),
    ] {
        mutations(&classic, first, second, changed, label);
    }
}
