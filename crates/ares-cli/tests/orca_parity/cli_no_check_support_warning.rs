#![cfg(target_os = "linux")]

use std::{ffi::OsString, fs, path::Path};

use serde_json::Value;

use super::cli_no_check::{gcode, inventory, run};
use crate::{
    artifacts, load_settings_cli::preset_files, native_wall_clock::result_comparison_bytes,
    runner::OrcaRunner,
};

#[test]
fn cli_no_check_skips_support_warning_only_for_a_following_slice() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = tempfile::Builder::new()
        .prefix("cli-no-check-support-warning-")
        .tempdir_in(artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let presets = preset_files(&root, "Creality", "Creality Ender-3 0.4 nozzle");
    let mut process: Value = serde_json::from_slice(&fs::read(&presets[1]).unwrap()).unwrap();
    process["enable_support"] = "0".into();
    fs::write(&presets[1], serde_json::to_vec(&process).unwrap()).unwrap();
    let settings = presets[..2].join(";");
    let filaments = presets[2..].join(";");
    let input = root.join("shelf.stl");
    fs::write(&input, cantilever_stl()).unwrap();

    let cases = [
        ("absent", &["--slice", "1"][..], true),
        ("after", &["--slice", "1", "--no-check"][..], true),
        ("before", &["--no-check", "--slice", "1"][..], false),
    ]
    .map(|(order, actions, expect_warning)| {
        let dir = root.join(order);
        fs::create_dir(&dir).unwrap();
        let argv: Vec<OsString> = ["--debug", "0", "--outputdir", "out"]
            .into_iter()
            .chain(actions.iter().copied())
            .chain([
                "--arrange",
                "0",
                "--load-settings",
                &settings,
                "--load-filaments",
                &filaments,
                input.to_str().unwrap(),
            ])
            .map(OsString::from)
            .collect();
        fs::write(dir.join("argv.json"), serde_json::to_vec(&argv).unwrap()).unwrap();
        let native = run(runner.bin(), &dir, "native", &argv);
        let cwd = dir.join("native/cwd");
        assert_eq!(native.status.code(), Some(0), "{dir:?}");
        assert!(native.stderr.is_empty(), "{dir:?}");
        assert_eq!(
            inventory(&cwd),
            ["out/", "out/plate_1.gcode", "out/result.json"]
        );
        let report = fs::read(cwd.join("out/result.json")).unwrap();
        let value: Value = serde_json::from_slice(&report).unwrap();
        let warnings = value["warnings"].as_array().unwrap();
        let plate_warning = &value["sliced_plates"][0]["warning_message"];
        assert_eq!(warnings.len(), usize::from(expect_warning), "{dir:?}");
        if expect_warning {
            let warning = &warnings[0];
            assert_eq!(warning["class"], "slicing_warning_non_critical");
            assert_eq!(warning["plate_id"], 1);
            assert!(
                warning["text"]
                    .as_str()
                    .unwrap()
                    .contains("floating cantilever")
            );
            assert_eq!(*plate_warning, warning["text"]);
        } else {
            assert_eq!(*plate_warning, "");
        }
        (dir, argv, native, report, gcode(&cwd))
    });
    // Establish from all three Native invocations that the flag does not change this export.
    assert!(
        cases[0].4 == cases[1].4 && cases[0].4 == cases[2].4,
        "Native G-code changed: {root:?}"
    );

    for (dir, argv, native, report, native_gcode) in cases {
        let ares = run(Path::new(env!("CARGO_BIN_EXE_ares")), &dir, "ares", &argv);
        let cwd = dir.join("ares/cwd");
        assert_eq!(ares.status.code(), native.status.code(), "{dir:?}");
        assert_eq!(ares.stderr, native.stderr, "{dir:?}");
        assert_eq!(ares.stdout, native.stdout, "{dir:?}");
        assert_eq!(
            inventory(&cwd),
            inventory(&dir.join("native/cwd")),
            "{dir:?}"
        );
        let actual = fs::read(cwd.join("out/result.json")).unwrap();
        assert!(
            result_comparison_bytes(&actual) == result_comparison_bytes(&report),
            "complete report differs: {dir:?}"
        );
        assert!(
            gcode(&cwd) == native_gcode,
            "complete G-code differs: {dir:?}"
        );
    }
}

fn cantilever_stl() -> Vec<u8> {
    // A 10 mm shelf projects from a 10 mm stem. Emit only the exposed cell faces.
    let coords = [
        [95.0_f32, 105.0, 115.0],
        [100.0, 110.0, 110.0],
        [0.0, 0.4, 0.8],
    ];
    let cells = [[0, 0, 0], [0, 0, 1], [1, 0, 1]];
    let mut bytes = vec![0; 84];
    let mut triangle_count = 0_u32;
    for cell in cells {
        for axis in 0..3 {
            let (u, v) = ((axis + 1) % 3, (axis + 2) % 3);
            for sign in [-1, 1] {
                let mut neighbor = cell;
                neighbor[axis] += sign;
                if cells.contains(&neighbor) {
                    continue;
                }
                let corners = [(0, 0), (1, 0), (1, 1), (0, 1)].map(|(a, b)| {
                    let mut p = cell;
                    p[axis] += i32::from(sign > 0);
                    p[u] += a;
                    p[v] += b;
                    std::array::from_fn::<_, 3, _>(|i| coords[i][p[i] as usize])
                });
                let mut normal = [0.0_f32; 3];
                normal[axis] = sign as f32;
                for face in if sign > 0 {
                    [[0, 1, 2], [0, 2, 3]]
                } else {
                    [[0, 2, 1], [0, 3, 2]]
                } {
                    for value in normal
                        .into_iter()
                        .chain(face.into_iter().flat_map(|i| corners[i]))
                    {
                        bytes.extend(value.to_le_bytes());
                    }
                    bytes.extend([0; 2]);
                    triangle_count += 1;
                }
            }
        }
    }
    bytes[80..84].copy_from_slice(&triangle_count.to_le_bytes());
    bytes
}
