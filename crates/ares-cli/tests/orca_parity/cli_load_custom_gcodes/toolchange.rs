use std::{ffi::OsString, fs, path::Path};

use serde_json::{Value, json};

use super::{OrcaRunner, artifacts, gcode, inventory, result_comparison_bytes, run};

#[test]
fn multiasingle_toolchange_round_trip_uses_all_roles() {
    compare_toolchange_fixture(
        ("MultiAsSingle", false),
        true,
        None,
        &[(1.0, 2), (2.0, 1), (2.4, 2), (2.8, 3)],
        Some(&[0, 1, 0, 1, 0]),
    );
}

#[test]
fn multiasingle_toolchange_keeps_separate_support_body_filament() {
    compare_support_toolchange((2, 0));
}

#[test]
fn multiasingle_toolchange_keeps_separate_support_interface_filament() {
    compare_support_toolchange((2, 2));
}

#[test]
fn multiasingle_toolchange_overrides_object_when_support_uses_its_original_tool() {
    compare_support_toolchange((1, 1));
}

fn compare_support_toolchange(support_tools: (u8, u8)) {
    compare_toolchange_fixture(
        ("MultiAsSingle", false),
        true,
        Some(support_tools),
        &[(1.0, 2), (2.0, 1)],
        None,
    );
}

pub(super) fn compare_multiasingle_toolchange(
    mode: &str,
    single_filament: bool,
    events: &[(f64, u32)],
    expected_tools: &[u32],
) {
    compare_toolchange_fixture(
        (mode, single_filament),
        false,
        None,
        events,
        Some(expected_tools),
    );
}

fn compare_toolchange_fixture(
    (mode, single_filament): (&str, bool),
    rich_roles: bool,
    support_tools: Option<(u8, u8)>,
    events: &[(f64, u32)],
    expected_tools: Option<&[u32]>,
) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = tempfile::Builder::new()
        .prefix("cli-load-custom-gcodes-toolchange-")
        .tempdir_in(artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    write_toolchange_fixture(&root, single_filament, rich_roles, support_tools);
    let events_path = root.join("events.json");
    let gcodes = events
        .iter()
        .map(|&(print_z, extruder)| {
            json!({
                "type": "ToolChange", "print_z": print_z, "color": "",
                "extruder": extruder, "extra": ""
            })
        })
        .collect::<Vec<_>>();
    fs::write(
        &events_path,
        serde_json::to_vec(&json!({"mode": mode, "gcodes": gcodes})).unwrap(),
    )
    .unwrap();
    let filaments = if single_filament {
        root.join("filament-a.json").display().to_string()
    } else {
        format!(
            "{};{}",
            root.join("filament-a.json").display(),
            root.join("filament-b.json").display()
        )
    };
    let argv = [
        "--debug",
        "0",
        "--outputdir",
        "out",
        "--slice",
        "1",
        "--load-settings",
        &format!(
            "{};{}",
            root.join("machine.json").display(),
            root.join("process.json").display()
        ),
        "--load-filaments",
        &filaments,
        "--load-custom-gcodes",
        events_path.to_str().unwrap(),
        root.join("block.stl").to_str().unwrap(),
    ]
    .map(OsString::from);
    fs::write(root.join("argv.json"), serde_json::to_vec(&argv).unwrap()).unwrap();
    let native = run(runner.bin(), &root, "native", &argv);
    let ares = run(Path::new(env!("CARGO_BIN_EXE_ares")), &root, "ares", &argv);
    let native_cwd = root.join("native/cwd");
    let ares_cwd = root.join("ares/cwd");
    assert_eq!(native.status.code(), Some(0), "{root:?}");
    assert_eq!(ares.status.code(), native.status.code(), "{root:?}");
    assert_eq!(ares.stderr, native.stderr, "{root:?}");
    assert_eq!(ares.stdout, native.stdout, "{root:?}");
    assert_eq!(inventory(&ares_cwd), inventory(&native_cwd), "{root:?}");
    let native_report = fs::read(native_cwd.join("out/result.json")).unwrap();
    let actual = fs::read(ares_cwd.join("out/result.json")).unwrap();
    assert!(
        result_comparison_bytes(&actual) == result_comparison_bytes(&native_report),
        "complete report differs: {root:?}"
    );
    let native_gcode = gcode(&native_cwd);
    let ares_gcode = gcode(&ares_cwd);
    assert!(
        ares_gcode == native_gcode,
        "complete G-code differs: {root:?}"
    );
    let actual_tools = native_gcode
        .split(|byte| *byte == b'\n')
        .filter_map(|line| line.strip_prefix(b"T"))
        .filter_map(|tool| std::str::from_utf8(tool).ok()?.parse::<u32>().ok())
        .collect::<Vec<_>>();
    if let Some(expected_tools) = expected_tools {
        assert_eq!(actual_tools, expected_tools, "{root:?}");
    }
    if rich_roles {
        assert_role_tools(&native_gcode, support_tools);
    }
}

fn assert_role_tools(gcode: &[u8], support_tools: Option<(u8, u8)>) {
    let mut tool = 0;
    let mut z = b"".as_slice();
    let mut roles = Vec::new();
    for line in gcode.split(|&byte| byte == b'\n') {
        match line {
            b"T0" => tool = 1,
            b"T1" => tool = 2,
            _ => {}
        }
        if let Some(height) = line.strip_prefix(b";Z:") {
            z = height;
        }
        if let Some(role) = line.strip_prefix(b";TYPE:") {
            roles.push((z, tool, role));
        }
    }
    for (z, tool) in [(b"1".as_slice(), 2), (b"2".as_slice(), 1)] {
        assert!(roles.contains(&(z, tool, b"Outer wall")));
    }
    for role in [
        b"Inner wall".as_slice(),
        b"Sparse infill",
        b"Internal solid infill",
    ] {
        assert!(roles.iter().any(|&(_, _, actual)| actual == role));
    }
    if support_tools.is_none() {
        assert!(roles.contains(&(b"1", 2, b"Internal solid infill")));
        assert!(roles.contains(&(b"2", 1, b"Sparse infill")));
        assert!(roles.contains(&(b"2.4", 2, b"Outer wall")));
        assert!(roles.contains(&(b"2.8", 1, b"Outer wall")));
    }
    if let Some((body, interface)) = support_tools {
        assert!(
            roles
                .iter()
                .any(|&(_, tool, role)| tool == body && role == b"Support")
        );
        if interface != 0 {
            assert!(
                roles
                    .iter()
                    .any(|&(_, tool, role)| tool == interface && role == b"Support interface")
            );
        }
    }
}

fn write_toolchange_fixture(
    root: &Path,
    single_filament: bool,
    rich_roles: bool,
    support_tools: Option<(u8, u8)>,
) {
    let write = |name: &str, value: Value| {
        fs::write(root.join(name), serde_json::to_vec(&value).unwrap()).unwrap();
    };
    write(
        "machine.json",
        json!({
            "type": "machine", "name": "Synthetic dual-filament machine", "from": "system",
            "printer_technology": "FFF", "printable_height": "200",
            "printable_area": ["0x0", "200x0", "200x200", "0x200"],
            "nozzle_diameter": ["0.4"], "printer_extruder_id": ["1"],
            "printer_extruder_variant": ["Direct Drive Standard"],
            "filament_diameter": if single_filament {
                json!(["1.75"])
            } else {
                json!(["1.75", "1.75"])
            },
            "layer_change_gcode": "G92 E0"
        }),
    );
    let mut process = json!({
        "type": "process", "name": "Synthetic single-object process", "from": "system",
        "compatible_printers": ["Synthetic dual-filament machine"], "layer_height": "0.2",
        "initial_layer_print_height": "0.2",
        "sparse_infill_density": if rich_roles { "20%" } else { "0%" },
        "wall_loops": if rich_roles { "2" } else { "1" },
        "top_shell_layers": if rich_roles { "2" } else { "0" },
        "bottom_shell_layers": if !rich_roles { "0" } else if support_tools.is_some() { "2" } else { "6" },
        "enable_prime_tower": "0"
    });
    if let Some((body, interface)) = support_tools {
        process["enable_support"] = json!("1");
        process["support_type"] = json!("normal(auto)");
        process["support_filament"] = json!(body.to_string());
        process["support_interface_filament"] = json!(interface.to_string());
        if body == 1 {
            // Exercise both tools at the object's switch height.
            process["independent_support_layer_height"] = json!("0");
        }
    }
    write("process.json", process);
    for (name, id, density, temperature) in [
        ("filament-a.json", "SYNTH_A", "1.11", "205"),
        ("filament-b.json", "SYNTH_B", "1.33", "225"),
    ] {
        write(
            name,
            json!({
                "type": "filament", "name": name, "from": "system", "filament_id": id,
                "filament_diameter": ["1.75"], "filament_density": [density],
                "filament_is_support": ["0"], "nozzle_temperature": [temperature],
                "filament_extruder_variant": ["Direct Drive Standard"],
                "filament_start_gcode": ["; synthetic filament"]
            }),
        );
    }
    let model = if support_tools.is_some() {
        include_bytes!("../../../../../tests/parity/overhang.stl").to_vec()
    } else {
        super::block_stl(108.0)
    };
    fs::write(root.join("block.stl"), model).unwrap();
}
