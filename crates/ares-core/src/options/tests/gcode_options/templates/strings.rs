use super::*;

macro_rules! assert_scalar_template {
    ($projected:ident, $source:ident, $field:ident, $expected:ident) => {
        assert_eq!(
            $projected.$field, $source.$field,
            "{} typed projection",
            stringify!($field)
        );
        assert_eq!(
            $projected.$field.0.as_bytes(),
            $expected.as_bytes(),
            "{} bytes",
            stringify!($field)
        );
    };
}

fn assert_vector_template(field: &str, actual: &OrcaStrings, expected: &[&str]) {
    assert_eq!(actual.0.len(), expected.len(), "{field} element count");
    for (index, (actual, expected)) in actual.0.iter().zip(expected).enumerate() {
        assert_eq!(
            actual.as_bytes(),
            expected.as_bytes(),
            "{field}[{index}] bytes"
        );
    }
}

#[test]
fn gcode_options_templates_preserve_all_sixteen_fields_byte_for_byte() {
    let before_layer = "G1 X1\nG1 Y2\n";
    let by_object = "M117 UTF-8: 零件";
    let machine_end = "M104 S0\r\nG28 X\r\n";
    let layer_change = "G1 X{layer_num}\n; literal \\n stays\n";
    let time_lapse = "";
    let wrapping = "M118 C:\\prints\\{input_filename_base}\n";
    let file_start = "{if layer_num > 0}\n;开始\n{endif}\n";
    let machine_start = "G28\r\nG1 Z{first_layer_height}\r\n";
    let change_filament = "M600 ; {filament_type[initial_tool]}\n";
    let change_role = "M117 role={extrusion_role}\n";
    let machine_pause = "M0\r\n";
    let custom = "\\\\server\\share\\file.gcode\n";
    let process_role = "{if extrusion_role==\"桥接\"}\r\nM117 bridge\r\n{endif}\r\n";
    let filament_end = ["", "M702 ;结束\n", "C:\\filament\\{filament_type}\r\n"];
    let filament_start = ["M701\n", "温度={nozzle_temperature[0]}\n"];
    let filament_role = ["\\role\\{extrusion_role}", "", "M118 尾行\r\n"];

    let printer = PrinterGCodeSourceOptions {
        before_layer_change_gcode: OrcaString(before_layer.to_owned()),
        printing_by_object_gcode: OrcaString(by_object.to_owned()),
        machine_end_gcode: OrcaString(machine_end.to_owned()),
        layer_change_gcode: OrcaString(layer_change.to_owned()),
        time_lapse_gcode: OrcaString(time_lapse.to_owned()),
        wrapping_detection_gcode: OrcaString(wrapping.to_owned()),
        file_start_gcode: OrcaString(file_start.to_owned()),
        machine_start_gcode: OrcaString(machine_start.to_owned()),
        change_filament_gcode: OrcaString(change_filament.to_owned()),
        change_extrusion_role_gcode: OrcaString(change_role.to_owned()),
        machine_pause_gcode: OrcaString(machine_pause.to_owned()),
        template_custom_gcode: OrcaString(custom.to_owned()),
        ..PrinterGCodeSourceOptions::default()
    };

    let process = ProcessGCodeSourceOptions {
        process_change_extrusion_role_gcode: OrcaString(process_role.to_owned()),
        ..ProcessGCodeSourceOptions::default()
    };

    let filament = FilamentGCodeSourceOptions {
        filament_end_gcode: OrcaStrings(owned_strings(&filament_end)),
        filament_start_gcode: OrcaStrings(owned_strings(&filament_start)),
        filament_change_extrusion_role_gcode: OrcaStrings(owned_strings(&filament_role)),
        ..FilamentGCodeSourceOptions::default()
    };

    let project_source = ProjectGCodeSourceOptions::default();
    let projected = project(&printer, &process, &filament, &project_source);

    assert_scalar_template!(projected, printer, before_layer_change_gcode, before_layer);
    assert_scalar_template!(projected, printer, printing_by_object_gcode, by_object);
    assert_scalar_template!(projected, printer, machine_end_gcode, machine_end);
    assert_scalar_template!(projected, printer, layer_change_gcode, layer_change);
    assert_scalar_template!(projected, printer, time_lapse_gcode, time_lapse);
    assert_scalar_template!(projected, printer, wrapping_detection_gcode, wrapping);
    assert_scalar_template!(projected, printer, file_start_gcode, file_start);
    assert_scalar_template!(projected, printer, machine_start_gcode, machine_start);
    assert_scalar_template!(projected, printer, change_filament_gcode, change_filament);
    assert_scalar_template!(projected, printer, change_extrusion_role_gcode, change_role);
    assert_scalar_template!(projected, printer, machine_pause_gcode, machine_pause);
    assert_scalar_template!(projected, printer, template_custom_gcode, custom);
    assert_scalar_template!(
        projected,
        process,
        process_change_extrusion_role_gcode,
        process_role
    );

    assert_eq!(projected.filament_end_gcode, filament.filament_end_gcode);
    assert_vector_template("filament_end_gcode", &projected.filament_end_gcode, &filament_end);
    assert_eq!(projected.filament_start_gcode, filament.filament_start_gcode);
    assert_vector_template(
        "filament_start_gcode",
        &projected.filament_start_gcode,
        &filament_start,
    );
    assert_eq!(
        projected.filament_change_extrusion_role_gcode,
        filament.filament_change_extrusion_role_gcode
    );
    assert_vector_template(
        "filament_change_extrusion_role_gcode",
        &projected.filament_change_extrusion_role_gcode,
        &filament_role,
    );
}
