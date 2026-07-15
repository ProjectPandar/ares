use std::collections::BTreeMap;

use super::super::{OptionValueKind, option_definition, option_definitions};

#[test]
fn task19b2_registry_has_fixed_inventory_and_histogram() {
    let definitions = option_definitions();
    assert_eq!(definitions.len(), 751);
    assert!(definitions.windows(2).all(|pair| pair[0].key < pair[1].key));

    let mut histogram = BTreeMap::new();
    for definition in definitions {
        *histogram
            .entry(fixed_kind_name(definition.kind))
            .or_insert(0) += 1;
    }
    assert_eq!(
        histogram,
        BTreeMap::from([
            ("coBool", 117),
            ("coBools", 22),
            ("coEnum", 49),
            ("coEnums", 9),
            ("coFloat", 210),
            ("coFloatOrPercent", 36),
            ("coFloats", 92),
            ("coInt", 47),
            ("coInts", 45),
            ("coPercent", 26),
            ("coPercents", 5),
            ("coPoint", 4),
            ("coPoints", 6),
            ("coPointsGroups", 1),
            ("coString", 48),
            ("coStrings", 34),
        ])
    );

    for (key, kind, default_value) in [
        ("bottom_surface_filament_id", OptionValueKind::Int, "0"),
        ("bridge_line_width", OptionValueKind::FloatOrPercent, "100%"),
        ("chamber_minimal_temperature", OptionValueKind::Ints, "0"),
        ("extruder", OptionValueKind::Int, "0"),
        ("flashforge_serial_number", OptionValueKind::String, ""),
        ("inner_wall_filament_id", OptionValueKind::Int, "0"),
        ("internal_solid_filament_id", OptionValueKind::Int, "0"),
        ("lightning_overhang_angle", OptionValueKind::Float, "45"),
        ("lightning_prune_angle", OptionValueKind::Float, "45"),
        (
            "lightning_straightening_angle",
            OptionValueKind::Float,
            "45",
        ),
        ("outer_wall_filament_id", OptionValueKind::Int, "0"),
        (
            "parallel_printheads_bed_exclude_areas",
            OptionValueKind::Strings,
            "",
        ),
        ("parallel_printheads_count", OptionValueKind::Int, "1"),
        ("relative_bridge_angle", OptionValueKind::Bool, "false"),
        ("sparse_infill_filament_id", OptionValueKind::Int, "0"),
        (
            "support_parallel_printheads",
            OptionValueKind::Bool,
            "false",
        ),
        ("top_surface_filament_id", OptionValueKind::Int, "0"),
        ("use_3mf", OptionValueKind::Bool, "false"),
    ] {
        let definition = option_definition(key).expect("missing fixed definition");
        assert_eq!(definition.kind, kind, "wrong kind for {key}");
        assert_eq!(
            definition.default_value, default_value,
            "wrong default for {key}"
        );
    }

    for key in [
        "solid_infill_filament",
        "sparse_infill_filament",
        "wall_filament",
    ] {
        assert!(option_definition(key).is_none(), "legacy-only row {key}");
    }
}

fn fixed_kind_name(kind: OptionValueKind) -> &'static str {
    match kind {
        OptionValueKind::Bool => "coBool",
        OptionValueKind::Bools | OptionValueKind::BoolsNullable => "coBools",
        OptionValueKind::Enum => "coEnum",
        OptionValueKind::Enums | OptionValueKind::EnumsNullable => "coEnums",
        OptionValueKind::Float => "coFloat",
        OptionValueKind::FloatOrPercent => "coFloatOrPercent",
        OptionValueKind::Floats | OptionValueKind::FloatsNullable => "coFloats",
        OptionValueKind::Int => "coInt",
        OptionValueKind::Ints | OptionValueKind::IntsNullable => "coInts",
        OptionValueKind::Percent => "coPercent",
        OptionValueKind::Percents | OptionValueKind::PercentsNullable => "coPercents",
        OptionValueKind::Point => "coPoint",
        OptionValueKind::Points => "coPoints",
        OptionValueKind::PointsGroups => "coPointsGroups",
        OptionValueKind::String => "coString",
        OptionValueKind::Strings => "coStrings",
    }
}
