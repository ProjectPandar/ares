use std::collections::{BTreeMap, BTreeSet};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct InventoryRow {
    key: String,
    raw_scope: RawScope,
    static_owner: StaticOwner,
    option_type: OrcaOptionType,
    nullable: bool,
    default_serialized: String,
    wire_shape: WireShape,
    effective_projections: Vec<EffectiveProjection>,
    legacy_inputs: Vec<LegacyInput>,
    config_export: ConfigExportRule,
    upstream_definition: SourceCitation,
    upstream_consumers: Vec<SourceCitation>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
#[serde(rename_all = "snake_case")]
enum RawScope {
    Printer,
    Process,
    Filament,
    Residual,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
#[serde(rename_all = "snake_case")]
enum StaticOwner {
    MachineEnvelopeConfig,
    GCodeConfig,
    PrintConfig,
    PrintObjectConfig,
    PrintRegionConfig,
    Unowned,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
enum OrcaOptionType {
    #[serde(rename = "coBool")]
    Bool,
    #[serde(rename = "coBools")]
    Bools,
    #[serde(rename = "coEnum")]
    Enum,
    #[serde(rename = "coEnums")]
    Enums,
    #[serde(rename = "coFloat")]
    Float,
    #[serde(rename = "coFloatOrPercent")]
    FloatOrPercent,
    #[serde(rename = "coFloats")]
    Floats,
    #[serde(rename = "coInt")]
    Int,
    #[serde(rename = "coInts")]
    Ints,
    #[serde(rename = "coPercent")]
    Percent,
    #[serde(rename = "coPercents")]
    Percents,
    #[serde(rename = "coPoint")]
    Point,
    #[serde(rename = "coPoints")]
    Points,
    #[serde(rename = "coPointsGroups")]
    PointsGroups,
    #[serde(rename = "coString")]
    String,
    #[serde(rename = "coStrings")]
    Strings,
    Metadata,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
enum WireShape {
    ScalarString,
    Array,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
enum EffectiveProjection {
    Object,
    Region,
    GCode,
}

#[derive(Debug, Deserialize)]
struct LegacyInput {
    key: String,
    conversion: LegacyConversion,
    citation: SourceCitation,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum LegacyConversion {
    Rename,
    ValueConversion,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind", content = "rule")]
enum ConfigExportRule {
    Canonical,
    OmitWhenNil,
    MetadataExclusion,
    FixedTagSpecial(String),
}

#[derive(Debug, Deserialize)]
struct SourceCitation {
    path: String,
    line: usize,
    symbol: String,
}

#[test]
fn project_inventory_has_exact_v242_shape_and_ownership() {
    let rows: Vec<InventoryRow> = serde_json::from_str(include_str!(
        "../../../../../tests/ksr_fdmtest_v4/options-v242.json"
    ))
    .unwrap();
    assert_eq!(rows.len(), 653);
    assert!(rows.windows(2).all(|pair| pair[0].key < pair[1].key));
    assert_eq!(
        rows.iter().map(|row| row.key.as_str()).collect::<BTreeSet<_>>().len(),
        653
    );

    let scope_counts = rows.iter().fold(BTreeMap::new(), |mut counts, row| {
        *counts.entry(row.raw_scope).or_insert(0) += 1;
        counts
    });
    assert_eq!(scope_counts[&RawScope::Printer], 132);
    assert_eq!(scope_counts[&RawScope::Process], 352);
    assert_eq!(scope_counts[&RawScope::Filament], 122);
    assert_eq!(scope_counts[&RawScope::Residual], 47);
    assert_eq!(rows.iter().filter(|row| row.nullable).count(), 31);

    for (projection, expected) in [
        (EffectiveProjection::Object, 126),
        (EffectiveProjection::Region, 153),
        (EffectiveProjection::GCode, 149),
    ] {
        assert_eq!(
            rows.iter()
                .filter(|row| row.effective_projections.contains(&projection))
                .count(),
            expected
        );
    }

    let histogram = rows.iter().fold(BTreeMap::new(), |mut counts, row| {
        *counts.entry(row.option_type).or_insert(0) += 1;
        counts
    });
    for (kind, expected) in [
        (OrcaOptionType::Bool, 105),
        (OrcaOptionType::Bools, 22),
        (OrcaOptionType::Enum, 44),
        (OrcaOptionType::Enums, 9),
        (OrcaOptionType::Float, 160),
        (OrcaOptionType::FloatOrPercent, 36),
        (OrcaOptionType::Floats, 90),
        (OrcaOptionType::Int, 41),
        (OrcaOptionType::Ints, 45),
        (OrcaOptionType::Percent, 25),
        (OrcaOptionType::Percents, 5),
        (OrcaOptionType::Point, 4),
        (OrcaOptionType::Points, 6),
        (OrcaOptionType::PointsGroups, 1),
        (OrcaOptionType::String, 30),
        (OrcaOptionType::Strings, 27),
        (OrcaOptionType::Metadata, 3),
    ] {
        assert_eq!(histogram[&kind], expected, "{kind:?}");
    }

    let defaults = rows
        .iter()
        .map(|row| (row.key.as_str(), row.default_serialized.as_str()))
        .collect::<BTreeMap<_, _>>();
    assert_eq!(defaults["accel_to_decel_enable"], "1");
    assert_eq!(defaults["bridge_line_width"], "100%");
    assert_eq!(defaults["machine_max_speed_x"], "500,200");
    assert_eq!(defaults["printer_technology"], "FFF");
    assert_eq!(defaults["wrapping_exclude_area"], "");
    assert!(defaults["machine_end_gcode"].contains("G28 X0  ; home X axis\\n"));
    assert!(defaults["machine_end_gcode"].contains("M84     ; disable motors\\n"));

    let owner_counts = rows.iter().fold(BTreeMap::new(), |mut counts, row| {
        *counts.entry((row.raw_scope, row.static_owner)).or_insert(0) += 1;
        counts
    });
    for (scope, owner, expected) in [
        (RawScope::Printer, StaticOwner::MachineEnvelopeConfig, 28),
        (RawScope::Printer, StaticOwner::GCodeConfig, 62),
        (RawScope::Printer, StaticOwner::PrintConfig, 27),
        (RawScope::Printer, StaticOwner::Unowned, 15),
        (RawScope::Process, StaticOwner::PrintObjectConfig, 126),
        (RawScope::Process, StaticOwner::PrintRegionConfig, 149),
        (RawScope::Process, StaticOwner::GCodeConfig, 17),
        (RawScope::Process, StaticOwner::PrintConfig, 59),
        (RawScope::Process, StaticOwner::Unowned, 1),
        (RawScope::Filament, StaticOwner::GCodeConfig, 53),
        (RawScope::Filament, StaticOwner::PrintRegionConfig, 4),
        (RawScope::Filament, StaticOwner::PrintConfig, 48),
        (RawScope::Filament, StaticOwner::Unowned, 17),
        (RawScope::Residual, StaticOwner::GCodeConfig, 17),
        (RawScope::Residual, StaticOwner::PrintConfig, 19),
        (RawScope::Residual, StaticOwner::Unowned, 11),
    ] {
        assert_eq!(owner_counts[&(scope, owner)], expected, "{scope:?}/{owner:?}");
    }
    assert!(rows.iter().all(|row| !row.upstream_definition.path.is_empty()
        && row.upstream_definition.line > 0
        && !row.upstream_definition.symbol.is_empty()
        && !row.upstream_consumers.is_empty()));
    let mut referenced_legacy_fields = 0;
    rows.iter()
        .flat_map(|row| &row.legacy_inputs)
        .for_each(|legacy| {
            assert!(!legacy.citation.path.is_empty());
            assert!(legacy.citation.line > 0);
            assert!(!legacy.citation.symbol.is_empty());
            assert!(!legacy.key.is_empty());
            let _ = legacy.conversion;
            referenced_legacy_fields += 1;
        });
    assert!(referenced_legacy_fields > 0);
    let specials = rows
        .iter()
        .filter_map(|row| match &row.config_export {
            ConfigExportRule::FixedTagSpecial(rule) => Some((row.key.as_str(), rule.as_str())),
            _ => None,
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(
        specials,
        BTreeSet::from([
            ("extruder_colour", "filament_colour_substitution"),
            ("flush_volumes_matrix", "scaled_flush_matrix"),
            ("wipe_tower_x", "plate_coordinate_duplicate"),
            ("wipe_tower_y", "plate_coordinate_duplicate"),
        ])
    );
    assert_eq!(
        rows.iter()
            .filter(|row| matches!(row.config_export, ConfigExportRule::OmitWhenNil))
            .count(),
        31
    );
    assert_eq!(
        rows.iter()
            .filter(|row| matches!(row.config_export, ConfigExportRule::MetadataExclusion))
            .count(),
        3
    );
    let scalar_count = rows
        .iter()
        .filter(|row| row.wire_shape == WireShape::ScalarString)
        .count();
    assert_eq!(scalar_count, 448);
    assert_eq!(rows.len() - scalar_count, 205);
}

#[test]
fn project_inventory_matches_the_embedded_fixture_keys_and_shapes() {
    let fixture = super::project_fixture::project_settings_value();
    let fixture = fixture.as_object().unwrap();
    let rows: Vec<InventoryRow> = serde_json::from_str(include_str!(
        "../../../../../tests/ksr_fdmtest_v4/options-v242.json"
    ))
    .unwrap();

    let inventory_keys = rows.iter().map(|row| row.key.as_str()).collect::<BTreeSet<_>>();
    let fixture_keys = fixture.keys().map(String::as_str).collect::<BTreeSet<_>>();
    assert_eq!(inventory_keys, fixture_keys);
    assert_eq!(fixture.values().filter(|value| value.is_string()).count(), 448);
    assert_eq!(fixture.values().filter(|value| value.is_array()).count(), 205);
    assert_eq!(
        fixture
            .values()
            .filter(|value| value.as_array().is_some_and(Vec::is_empty))
            .count(),
        5
    );
    assert!(rows.iter().all(|row| match row.wire_shape {
        WireShape::ScalarString => fixture[&row.key].is_string(),
        WireShape::Array => fixture[&row.key].is_array(),
    }));
}
