use ares_core::{DefaultBedType, PresetCatalogHandle, ProfileKind, ProjectBedType};

// Preset.cpp:1140–1175 and PrintConfig.hpp's BedType enum at 40ecc66f7.
#[test]
fn default_bed_hint_matches_enum_names_before_legacy_integer_prefixes() {
    use ProjectBedType::{
        CoolPlate, EngineeringPlate, HighTempPlate, SupertackPlate, TexturedCoolPlate,
        TexturedPeiPlate,
    };
    for (hint, expected) in [
        ("Cool Plate", CoolPlate),
        ("Engineering Plate", EngineeringPlate),
        ("High Temp Plate", HighTempPlate),
        ("Textured PEI Plate", TexturedPeiPlate),
        ("Textured Cool Plate", TexturedCoolPlate),
        ("Supertack Plate", SupertackPlate),
        ("1", CoolPlate),
        ("2", EngineeringPlate),
        ("3", HighTempPlate),
        ("4", TexturedPeiPlate),
        ("5", TexturedCoolPlate),
        ("6", SupertackPlate),
        (" +04suffix", TexturedPeiPlate),
        ("\u{b}2.9", EngineeringPlate),
        ("Default Plate", HighTempPlate),
        ("SuperTack Plate", HighTempPlate),
        ("textured pei plate", HighTempPlate),
        (" Textured PEI Plate", HighTempPlate),
        ("invalid", HighTempPlate),
        (" ", HighTempPlate),
        ("0", HighTempPlate),
        ("-1", HighTempPlate),
        ("7", HighTempPlate),
    ] {
        for model in ["BL-P001", "C13", "C11", "unknown"] {
            assert_eq!(
                DefaultBedType(hint.into()).resolve(model),
                expected,
                "{hint:?}, {model}"
            );
        }
    }
}

#[test]
fn only_an_empty_default_bed_hint_uses_the_model_identifier() {
    for (model, expected) in [
        ("BL-P001", ProjectBedType::CoolPlate),
        ("BL-P002", ProjectBedType::CoolPlate),
        ("C13", ProjectBedType::CoolPlate),
        ("C11", ProjectBedType::HighTempPlate),
        ("unknown", ProjectBedType::HighTempPlate),
        ("", ProjectBedType::HighTempPlate),
    ] {
        assert_eq!(
            DefaultBedType::default().resolve(model),
            expected,
            "{model}"
        );
    }
}

#[test]
fn unchanged_u1_resource_bytes_resolve_the_inherited_textured_pei_hint() {
    let mut catalog = PresetCatalogHandle::new();
    for (filename, bytes) in [
        (
            "fdm_klipper.json",
            include_bytes!("../../../tests/profiles/default-bed-type/fdm_klipper.json").as_slice(),
        ),
        (
            "fdm_toolchanger.json",
            include_bytes!("../../../tests/profiles/default-bed-type/fdm_toolchanger.json"),
        ),
        (
            "fdm_U1.json",
            include_bytes!("../../../tests/profiles/default-bed-type/fdm_U1.json"),
        ),
        (
            "Snapmaker U1 (0.4 nozzle).json",
            include_bytes!(
                "../../../tests/profiles/default-bed-type/Snapmaker U1 (0.4 nozzle).json"
            ),
        ),
    ] {
        catalog.add_import(bytes, filename).unwrap();
    }
    let mut values = serde_json::to_value(
        catalog
            .preset_values(
                ProfileKind::Machine,
                &["Snapmaker U1 (0.4 nozzle)".to_owned()],
                &["default_bed_type"],
            )
            .unwrap(),
    )
    .unwrap();
    let hint: DefaultBedType =
        serde_json::from_value(values[0]["default_bed_type"].take()).unwrap();
    assert_eq!(hint.0, "Textured PEI Plate");
    assert_eq!(hint.resolve(""), ProjectBedType::TexturedPeiPlate);
}

// PartPlate.cpp:267–314: Default is an in-memory plate inheritance marker.
#[test]
fn plate_bed_selection_inherits_the_project_but_preserves_concrete_overrides() {
    use ProjectBedType::{DefaultPlate, HighTempPlate, TexturedPeiPlate};
    assert_eq!(DefaultPlate.resolve(HighTempPlate), HighTempPlate);
    assert_eq!(DefaultPlate.resolve(TexturedPeiPlate), TexturedPeiPlate);
    assert_eq!(TexturedPeiPlate.resolve(HighTempPlate), TexturedPeiPlate);
    assert_eq!(HighTempPlate.resolve(TexturedPeiPlate), HighTempPlate);
}
