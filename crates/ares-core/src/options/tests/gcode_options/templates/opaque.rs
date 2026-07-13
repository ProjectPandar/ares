use super::*;

fn assert_payload_bytes(field: &str, actual: &[String], expected: &[&str]) {
    assert_eq!(actual.len(), expected.len(), "{field} element count");
    for (index, (actual, expected)) in actual.iter().zip(expected).enumerate() {
        assert_eq!(
            actual.as_bytes(),
            expected.as_bytes(),
            "{field}[{index}] bytes"
        );
    }
}

#[test]
fn gcode_options_templates_preserve_four_opaque_wrapper_payloads() {
    let csv = ["0,0,{k|v};a\\b\n1,2,路径", "tail,with,[nested;tokens]"];
    let tuple = ["1 2 {a,b}|c;d\\e\r\n元组", ""];
    let ramming = ["120 100|0.05 6.6;{ram\\path}\n冲压", "x|y,z;w"];
    let compensation = ["0,0|{flow;value}\\curve\n补偿", "\r\n1.5,0.8571"];

    let filament = FilamentGCodeSourceOptions {
        adaptive_pressure_advance_model: CsvTable(owned_strings(&csv)),
        volumetric_speed_coefficients: SpaceTuple(owned_strings(&tuple)),
        filament_ramming_parameters: RammingParameters(owned_strings(&ramming)),
        ..FilamentGCodeSourceOptions::default()
    };

    let process = ProcessGCodeSourceOptions {
        small_area_infill_flow_compensation_model: OrcaStrings(owned_strings(&compensation)),
        ..ProcessGCodeSourceOptions::default()
    };

    let printer = PrinterGCodeSourceOptions::default();
    let project_source = ProjectGCodeSourceOptions::default();
    let projected = project(&printer, &process, &filament, &project_source);

    assert_eq!(
        projected.adaptive_pressure_advance_model,
        filament.adaptive_pressure_advance_model
    );
    assert_payload_bytes(
        "adaptive_pressure_advance_model",
        &projected.adaptive_pressure_advance_model.0,
        &csv,
    );
    assert_eq!(
        projected.volumetric_speed_coefficients,
        filament.volumetric_speed_coefficients
    );
    assert_payload_bytes(
        "volumetric_speed_coefficients",
        &projected.volumetric_speed_coefficients.0,
        &tuple,
    );
    assert_eq!(
        projected.filament_ramming_parameters,
        filament.filament_ramming_parameters
    );
    assert_payload_bytes(
        "filament_ramming_parameters",
        &projected.filament_ramming_parameters.0,
        &ramming,
    );
    assert_eq!(
        projected.small_area_infill_flow_compensation_model,
        process.small_area_infill_flow_compensation_model
    );
    assert_payload_bytes(
        "small_area_infill_flow_compensation_model",
        &projected.small_area_infill_flow_compensation_model.0,
        &compensation,
    );
}
