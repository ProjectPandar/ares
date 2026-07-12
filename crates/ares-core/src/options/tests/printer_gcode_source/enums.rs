use serde::{Serialize, de::DeserializeOwned};

use super::super::super::{
    BedTemperatureFormula, ExtruderType, GCodeFlavor, NozzleType, PowerLossRecoveryMode,
    PrinterStructure, RetractLiftEnforce, WipeTowerType, ZHopType,
};

#[test]
fn printer_gcode_source_enums_have_exact_fixed_tag_domains() {
    assert_domain(&[
        ("by_first_filament", BedTemperatureFormula::FirstFilament),
        ("by_highest_temp", BedTemperatureFormula::HighestTemp),
    ]);
    assert_domain(&[
        ("printer_configuration", PowerLossRecoveryMode::PrinterConfiguration),
        ("enable", PowerLossRecoveryMode::Enable),
        ("disable", PowerLossRecoveryMode::Disable),
    ]);
    assert_domain(&[
        ("marlin", GCodeFlavor::MarlinLegacy),
        ("klipper", GCodeFlavor::Klipper),
        ("reprapfirmware", GCodeFlavor::RepRapFirmware),
        ("repetier", GCodeFlavor::Repetier),
        ("marlin2", GCodeFlavor::MarlinFirmware),
        ("reprap", GCodeFlavor::RepRapSprinter),
        ("teacup", GCodeFlavor::Teacup),
        ("makerware", GCodeFlavor::MakerWare),
        ("sailfish", GCodeFlavor::Sailfish),
        ("smoothie", GCodeFlavor::Smoothie),
        ("mach3", GCodeFlavor::Mach3),
        ("machinekit", GCodeFlavor::Machinekit),
        ("no-extrusion", GCodeFlavor::NoExtrusion),
    ]);
    assert_domain(&[
        ("undefine", NozzleType::Undefine),
        ("hardened_steel", NozzleType::HardenedSteel),
        ("stainless_steel", NozzleType::StainlessSteel),
        ("tungsten_carbide", NozzleType::TungstenCarbide),
        ("brass", NozzleType::Brass),
    ]);
    assert_domain(&[
        ("undefine", PrinterStructure::Undefine),
        ("corexy", PrinterStructure::CoreXy),
        ("i3", PrinterStructure::I3),
        ("hbot", PrinterStructure::Hbot),
        ("delta", PrinterStructure::Delta),
    ]);
    assert_domain(&[
        ("Auto Lift", ZHopType::Auto),
        ("Normal Lift", ZHopType::Normal),
        ("Slope Lift", ZHopType::Slope),
        ("Spiral Lift", ZHopType::Spiral),
    ]);
    assert_domain(&[
        ("All Surfaces", RetractLiftEnforce::AllSurfaces),
        ("Top Only", RetractLiftEnforce::TopOnly),
        ("Bottom Only", RetractLiftEnforce::BottomOnly),
        ("Top and Bottom", RetractLiftEnforce::TopAndBottom),
    ]);
    assert_domain(&[
        ("Direct Drive", ExtruderType::DirectDrive),
        ("Bowden", ExtruderType::Bowden),
    ]);
    assert_domain(&[("type1", WipeTowerType::Type1), ("type2", WipeTowerType::Type2)]);

    assert!(serde_json::from_str::<NozzleType>(r#""E3D""#).is_err());
}

fn assert_domain<T>(domain: &[(&str, T)])
where
    T: Copy + std::fmt::Debug + PartialEq + Serialize + DeserializeOwned,
{
    for (wire, expected) in domain {
        let json = format!(r#""{wire}""#);
        assert_eq!(serde_json::from_str::<T>(&json).unwrap(), *expected);
        assert_eq!(serde_json::to_string(expected).unwrap(), json);
    }
}
