use ares_core::SliceOptions;
use serde_json::json;

// PrintConfig.cpp:1449–1467: the registered current-bed default is Cool Plate.
#[tokio::test]
async fn byte_slice_uses_registered_cool_plate_for_a_missing_current_bed() {
    for (current, hint, initial, ordinary) in [
        (None, "Textured PEI Plate", 35, 31),
        (None, "4", 35, 31),
        (None, "", 35, 31),
        (None, "invalid", 35, 31),
        (Some("Cool Plate"), "Textured PEI Plate", 35, 31),
    ] {
        let mut values = json!({
            "default_bed_type": hint,
            "cool_plate_temp_initial_layer": [35],
            "cool_plate_temp": [31],
            "hot_plate_temp_initial_layer": [45],
            "hot_plate_temp": [41],
            "textured_plate_temp_initial_layer": [55],
            "textured_plate_temp": [51],
            "machine_start_gcode": "M140 S[bed_temperature_initial_layer_single]\nM190 S[bed_temperature_initial_layer_single]"
        });
        if let Some(current) = current {
            values["curr_bed_type"] = current.into();
        }
        let options: SliceOptions = serde_json::from_value(values).unwrap();
        let bytes = ares_core::slice(include_bytes!("../../../tests/parity/cube10.stl"), options)
            .await
            .unwrap();
        let gcode = std::str::from_utf8(&bytes).unwrap();
        let bed_commands: Vec<_> = gcode
            .lines()
            .filter(|line| line.starts_with("M140 ") || line.starts_with("M190 "))
            .map(|line| line.split(';').next().unwrap().trim())
            .collect();
        for command in [
            format!("M140 S{initial}"),
            format!("M190 S{initial}"),
            format!("M140 S{ordinary}"),
        ] {
            assert!(
                bed_commands.contains(&command.as_str()),
                "{current:?}, {hint:?}: {command}, {bed_commands:?}"
            );
        }
    }
}
