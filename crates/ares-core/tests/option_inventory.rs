use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

const FIXED_COMMIT: &str = "8500fcdccaa10b5099ac20d252af3a7c560046f1";

#[path = "option_inventory/provenance.rs"]
mod provenance;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
struct OptionInventoryRow {
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

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum RawScope {
    Printer,
    Process,
    Filament,
    Residual,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum StaticOwner {
    MachineEnvelopeConfig,
    GCodeConfig,
    PrintConfig,
    PrintObjectConfig,
    PrintRegionConfig,
    Unowned,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
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

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum WireShape {
    ScalarString,
    Array,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum EffectiveProjection {
    Object,
    Region,
    GCode,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
struct LegacyInput {
    key: String,
    conversion: LegacyConversion,
    citation: SourceCitation,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum LegacyConversion {
    Rename,
    ValueConversion,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "kind", content = "rule")]
enum ConfigExportRule {
    Canonical,
    OmitWhenNil,
    MetadataExclusion,
    FixedTagSpecial(String),
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
struct SourceCitation {
    path: String,
    line: usize,
    symbol: String,
}

fn committed_inventory() -> Vec<OptionInventoryRow> {
    serde_json::from_str(include_str!(
        "../../../tests/ksr_fdmtest_v4/options-v242.json"
    ))
    .unwrap()
}

#[test]
fn committed_inventory_is_available_without_an_orca_checkout() {
    let rows = committed_inventory();
    assert_eq!(rows.len(), 653);
    assert_eq!(
        rows.iter()
            .map(|row| &row.key)
            .collect::<BTreeSet<_>>()
            .len(),
        653
    );
    for row in &rows {
        assert!(!row.upstream_consumers.is_empty(), "{}", row.key);
        for citation in &row.upstream_consumers {
            assert!(
                !matches!(
                    citation.path.as_str(),
                    "src/libslic3r/PrintConfig.hpp"
                        | "src/libslic3r/PrintConfig.cpp"
                        | "src/libslic3r/Preset.cpp"
                ),
                "{} has a declaration/static-list consumer",
                row.key
            );
        }
    }
}

#[test]
fn committed_inventory_keeps_qualified_enum_defaults_and_axis_declarations() {
    let rows = committed_inventory();
    let input_shaping = rows
        .iter()
        .find(|row| row.key == "input_shaping_type")
        .unwrap();
    assert_eq!(input_shaping.default_serialized, "Default");
    let nozzle_type = rows.iter().find(|row| row.key == "nozzle_type").unwrap();
    assert_eq!(nozzle_type.default_serialized, "undefine");

    let expected = [
        ("machine_max_acceleration_x", 1257),
        ("machine_max_acceleration_y", 1258),
        ("machine_max_acceleration_z", 1259),
        ("machine_max_acceleration_e", 1260),
        ("machine_max_speed_x", 1262),
        ("machine_max_speed_y", 1263),
        ("machine_max_speed_z", 1264),
        ("machine_max_speed_e", 1265),
        ("machine_max_jerk_x", 1273),
        ("machine_max_jerk_y", 1274),
        ("machine_max_jerk_z", 1275),
        ("machine_max_jerk_e", 1276),
    ];
    assert_eq!(expected.len(), 12);
    for (key, line) in expected {
        let row = rows.iter().find(|row| row.key == key).unwrap();
        assert_eq!(
            row.upstream_definition.path,
            "src/libslic3r/PrintConfig.hpp"
        );
        assert_eq!(row.upstream_definition.line, line);
        assert_eq!(row.upstream_definition.symbol, key);
    }
}

#[test]
#[ignore = "requires ORCA_SLICER_REPO fixed-commit provenance checkout"]
fn inventory_matches_fixed_orca_source_provenance() {
    let repo = std::env::var_os("ORCA_SLICER_REPO").expect("ORCA_SLICER_REPO must be set");
    let files = [
        "src/libslic3r/Config.hpp",
        "src/libslic3r/CommonDefs.hpp",
        "src/libslic3r/Config.cpp",
        "src/libslic3r/PrintConfig.hpp",
        "src/libslic3r/PrintConfig.cpp",
        "src/libslic3r/PrintConfigConstants.hpp",
        "src/libslic3r/Preset.cpp",
        "src/libslic3r/GCode.cpp",
        "src/libslic3r/Format/bbs_3mf.cpp",
    ];
    let mut source_paths = files
        .into_iter()
        .map(str::to_owned)
        .collect::<BTreeSet<_>>();
    let rows = committed_inventory();
    source_paths.extend(rows.iter().flat_map(|row| {
        std::iter::once(row.upstream_definition.path.clone())
            .chain(row.upstream_consumers.iter().map(|item| item.path.clone()))
            .chain(
                row.legacy_inputs
                    .iter()
                    .map(|item| item.citation.path.clone()),
            )
    }));
    let sources = source_paths
        .into_iter()
        .map(|path| {
            let source = provenance::git_show(&repo, &path);
            (path, source)
        })
        .collect::<BTreeMap<_, _>>();

    for row in &rows {
        provenance::verify_citation(&sources, &row.upstream_definition);
        for citation in &row.upstream_consumers {
            provenance::verify_citation(&sources, citation);
            provenance::verify_consumer_citation(
                &row.key,
                row.option_type == OrcaOptionType::Metadata,
                citation,
            );
        }
        for legacy in &row.legacy_inputs {
            provenance::verify_citation(&sources, &legacy.citation);
        }
    }
    let gcode = &sources["src/libslic3r/GCode.cpp"];
    provenance::verify_axis_defaults(&sources["src/libslic3r/PrintConfig.cpp"], &rows);
    provenance::verify_nozzle_type_default(
        &sources["src/libslic3r/PrintConfig.cpp"],
        &sources["src/libslic3r/Config.hpp"],
        &sources["src/libslic3r/CommonDefs.hpp"],
        &rows,
    );
    let derived_export_rules = provenance::derive_export_rules(gcode);
    let artifact_export_rules = rows
        .iter()
        .filter_map(|row| match &row.config_export {
            ConfigExportRule::FixedTagSpecial(rule) => Some((row.key.clone(), rule.clone())),
            _ => None,
        })
        .collect::<BTreeMap<_, _>>();
    assert_eq!(derived_export_rules, artifact_export_rules);
    provenance::verify_rust_parser_mutations(gcode);
    provenance::verify_source_mutations(&repo);
    let reconstructed = provenance::reconstruct_inventory(&repo);
    assert_eq!(reconstructed, rows);
}
