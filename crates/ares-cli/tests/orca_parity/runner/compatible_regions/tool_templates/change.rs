use super::outputs;
use serde_json::json;

#[test]
fn custom_toolchange_renders_runtime_context_and_selects_tool_once() {
    if !crate::runner::reference_available() {
        return;
    }
    let source = "; custom change {previous_extruder} {next_extruder} {toolchange_count} {layer_num} {relative_e_axis}\n T[next_extruder]\nG1 Z{toolchange_z + 2}";
    let (reference, actual) = outputs(
        "; custom start {filament_extruder_id} {layer_z}",
        "; custom end {filament_extruder_id}",
        false,
        &[("change_filament_gcode", json!(source))],
    );
    let blocks = |bytes: &[u8]| {
        let output = std::str::from_utf8(bytes).unwrap();
        let executable = &output[output.find("; EXECUTABLE_BLOCK_START").unwrap()
            ..output.find("; EXECUTABLE_BLOCK_END").unwrap()];
        executable
            .split("; custom change ")
            .skip(1)
            .map(|block| {
                let end = block.find("; custom start ").unwrap();
                let start = block[end..].lines().next().unwrap();
                let commands = block[..end]
                    .lines()
                    .filter(|line| line.trim_start().starts_with('T'))
                    .map(str::to_owned)
                    .collect::<Vec<_>>();
                let forced = block[..end].contains(";_FORCE_RESUME_FAN_SPEED\n");
                (
                    block.lines().next().unwrap().to_owned(),
                    commands,
                    forced,
                    start.to_owned(),
                )
            })
            .collect::<Vec<_>>()
    };
    let expected = blocks(&reference);
    assert_eq!(
        expected.len(),
        17,
        "native: initial selection plus16 changes"
    );
    assert_eq!(expected[0].0, "0 0 1 -1 true");
    assert_eq!(expected[1].0, "0 1 2 0 true");
    assert_eq!(expected[1].3, "; custom start 1 2.2");
    assert!(expected[0].2, "native: initial resume fan marker");
    for (index, (_, commands, _, _)) in expected.iter().enumerate() {
        assert_eq!(
            commands.len(),
            1,
            "native transition{index}: no duplicate T"
        );
    }
    assert_eq!(
        blocks(&actual),
        expected,
        "Ares: live custom toolchange blocks"
    );
}
