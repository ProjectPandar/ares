use super::*;

#[test]
fn set_junction_deviation_formats_marlin_firmware_m205_j() {
    let mut writer = GCodeWriter::new();
    writer.set_gcode_flavor(GCodeFlavor::MarlinFirmware);

    assert_eq!(
        writer.set_junction_deviation(0.025, 0.1, false),
        "M205 J0.025\n"
    );
}

#[test]
fn set_junction_deviation_clamps_to_machine_maximum() {
    let mut writer = GCodeWriter::new();
    writer.set_gcode_flavor(GCodeFlavor::MarlinFirmware);

    assert_eq!(
        writer.set_junction_deviation(0.25, 0.1, false),
        "M205 J0.100\n"
    );
}

#[test]
fn set_junction_deviation_suppresses_unsupported_or_zero_values() {
    let mut writer = GCodeWriter::new();

    assert_eq!(writer.set_junction_deviation(0.025, 0.1, false), "");

    writer.set_gcode_flavor(GCodeFlavor::MarlinFirmware);
    assert_eq!(writer.set_junction_deviation(0.0, 0.1, false), "");
    assert_eq!(writer.set_junction_deviation(0.025, 0.0, false), "");
}

#[test]
fn set_junction_deviation_suppresses_all_non_marlin_firmware_flavors() {
    for flavor in [
        GCodeFlavor::MarlinLegacy,
        GCodeFlavor::Klipper,
        GCodeFlavor::RepRapFirmware,
        GCodeFlavor::Repetier,
        GCodeFlavor::RepRapSprinter,
        GCodeFlavor::Teacup,
        GCodeFlavor::MakerWare,
        GCodeFlavor::Sailfish,
        GCodeFlavor::Mach3,
        GCodeFlavor::Machinekit,
        GCodeFlavor::Smoothie,
        GCodeFlavor::NoExtrusion,
    ] {
        let mut writer = GCodeWriter::new();
        writer.set_gcode_flavor(flavor);

        assert_eq!(
            writer.set_junction_deviation(0.025, 0.1, false),
            "",
            "{flavor:?}"
        );
    }
}

#[test]
fn set_junction_deviation_appends_orca_comment_when_enabled() {
    let mut writer = GCodeWriter::new();
    writer.set_gcode_flavor(GCodeFlavor::MarlinFirmware);

    assert_eq!(
        writer.set_junction_deviation(0.025, 0.1, true),
        "M205 J0.025 ; Junction Deviation\n"
    );
}
