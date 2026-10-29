const PROJECT_SETTINGS: &str = "Metadata/project_settings.config";
const FILAMENT_START_BLOCK: &str = concat!(
    "\t\"filament_start_gcode\": [\r\n",
    "\t\t\"; filament start gcode\\n\",\r\n",
    "\t\t\"; filament start gcode\\n\"\r\n",
    "\t]",
);

#[tokio::test]
async fn filament_start_stays_outside_layer_cooling_rewrite() {
    let preamble = print_preamble("G1 X1 F9000\\nG1 X2 F9000").await;

    assert!(preamble.contains("G1 X1 F9000\nG1 X2 F9000\n"));
}

#[tokio::test]
async fn whitespace_only_filament_start_emits_no_print_start_line() {
    let preamble = print_preamble(" ").await;

    assert!(!preamble.lines().any(|line| line == " "));
}

async fn print_preamble(value: &str) -> String {
    let mut archive = crate::project_slice::tests::support::KsrArchive::new();
    let replacement =
        format!("\t\"filament_start_gcode\": [\r\n\t\t\"{value}\",\r\n\t\t\"{value}\"\r\n\t]");
    archive.replace_unique(PROJECT_SETTINGS, FILAMENT_START_BLOCK, &replacement);
    archive.replace_unique(
        PROJECT_SETTINGS,
        "\"printer_model\": \"Bambu Lab X2D\"",
        "\"printer_model\": \"SeeMeCNC Artemis\"",
    );
    // The KSR stock `time_lapse_gcode` belongs to the X2D insert-lambda
    // overlay; on this non-BBL routing it renders through the layer-start
    // overlay (`GCode.cpp:4667-4674`), which does not provide
    // `{has_timelapse_safe_pos}` — Native 2.4.2 rejects the stock template
    // here (`timelapse_gcode Parsing error ... Not a variable name`), so
    // the template is cleared to keep this test on an accepted input.
    let mut settings: serde_json::Value =
        serde_json::from_str(&archive.entry_text(PROJECT_SETTINGS)).unwrap();
    settings["time_lapse_gcode"] = serde_json::Value::String(String::new());
    archive.insert_text(PROJECT_SETTINGS, &serde_json::to_string(&settings).unwrap());

    let output = crate::slice_project(
        &archive.bytes(),
        crate::project_slice::tests::support::metadata(),
        crate::SlicerContext::new(0),
    )
    .await
    .unwrap();
    let output = std::str::from_utf8(&output).unwrap();
    let first_layer = output
        .find("; CHANGE_LAYER")
        .or_else(|| output.find(";LAYER_CHANGE"))
        .unwrap();
    output[..first_layer].to_owned()
}
