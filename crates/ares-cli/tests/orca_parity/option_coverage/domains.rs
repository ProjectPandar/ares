use std::path::Path;

use serde_json::Value;

#[path = "widths.rs"]
pub(super) mod widths;

#[derive(Clone, Debug)]
pub(super) struct OptionPlan {
    pub(super) key: String,
    pub(super) option_type: String,
    pub(super) raw_scope: String,
    pub(super) source: String,
    pub(super) cases: Vec<OptionCase>,
    pub(super) omission: Option<&'static str>,
    pub(super) width_domain: Option<widths::WidthDomain>,
    pub(super) tool_metadata: Option<Value>,
    /// Whether the generated cases provably cover the option's legal domain:
    /// bool/enum exhaust their declared value set, schema-bounded numerics
    /// cover `min`/`max`/interior, and width options carry a context-bound
    /// `width_domain`. Unbounded/baseline-only domains are not provable.
    pub(super) provably_legal: bool,
}

#[derive(Clone, Debug)]
pub(super) struct OptionCase {
    pub(super) label: String,
    pub(super) value: Option<Value>,
}

/// Offline callers return `None` (and skip) when the vendored upstream
/// source tree is absent — CI jobs do not checkout `OrcaSlicer/`.
pub(super) fn try_load(repo: &Path) -> Option<Vec<OptionPlan>> {
    if !repo
        .join("OrcaSlicer/src/libslic3r/PrintConfig.cpp")
        .exists()
    {
        eprintln!("skipping: OrcaSlicer source tree not vendored at {repo:?}/OrcaSlicer");
        return None;
    }
    Some(load(repo))
}

pub(super) fn load(repo: &Path) -> Vec<OptionPlan> {
    let mut inventory: Value = serde_json::from_str(
        &std::fs::read_to_string(repo.join("tests/ksr_fdmtest_v4/options-v242.json")).unwrap(),
    )
    .unwrap();
    // The historical inventory omits this live standalone bool (not the enum token).
    inventory.as_array_mut().unwrap().push(serde_json::json!({
        "key":"brim_ears","option_type":"coBool","raw_scope":"process",
        "upstream_definition":{"path":"src/libslic3r/PrintConfig.cpp","line":1751}
    }));
    let source =
        std::fs::read_to_string(repo.join("OrcaSlicer/src/libslic3r/PrintConfig.cpp")).unwrap();
    let lines = source.lines().collect::<Vec<_>>();
    inventory
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|entry| plan(entry, &lines, &source))
        .collect()
}

fn plan(entry: &Value, lines: &[&str], source_text: &str) -> Option<OptionPlan> {
    let key = entry.get("key")?.as_str()?.to_owned();
    let option_type = entry.get("option_type")?.as_str()?.to_owned();
    let raw_scope = entry.get("raw_scope")?.as_str()?.to_owned();
    if option_type == "Metadata" {
        return None;
    }
    let definition = entry.get("upstream_definition")?;
    let line = definition.get("line")?.as_u64()? as usize;
    let source = format!("{}:{}", definition.get("path")?.as_str()?, line);
    let block = definition_block(lines, &key, line);
    let mut width_domain = None;
    let mut tool_metadata = None;
    let mut provably_legal = false;
    let (values, omission) = if key == super::adaptive_pa_bridges::KEY {
        provably_legal = true;
        (super::adaptive_pa_bridges::cases(), None)
    } else if key == super::print_compatible_printers::KEY {
        provably_legal = true;
        (super::print_compatible_printers::cases(), None)
    } else if key == "post_process" {
        #[cfg(target_os = "linux")]
        {
            let (values, metadata) = super::post_process_matrix::generate();
            tool_metadata = Some(metadata);
            provably_legal = true;
            (values, None)
        }
        #[cfg(not(target_os = "linux"))]
        {
            (
                vec![baseline_case()],
                Some(
                    "bounded CLI admission witnesses require Linux; external execution is uncovered",
                ),
            )
        }
    } else if let Some((values, metadata)) = super::print_extruder_identity::generate(&key) {
        tool_metadata = Some(metadata);
        provably_legal = true;
        (values, None)
    } else if let Some((values, metadata)) = super::legacy_brim_ears::generate(&key) {
        tool_metadata = Some(metadata);
        provably_legal = true;
        (values, None)
    } else if let Some((values, metadata)) = super::nozzle_dataset::generate(&key) {
        tool_metadata = Some(metadata);
        provably_legal = true;
        (values, None)
    } else if let Some((values, metadata)) = super::semm_ramming::generate(&key) {
        tool_metadata = Some(metadata);
        provably_legal = true;
        (values, None)
    } else if let Some((values, metadata)) = super::filament_self_index::generate(&key) {
        tool_metadata = Some(metadata);
        provably_legal = true;
        (values, None)
    } else if let Some((values, metadata)) = super::master_extruder_id::generate(&key) {
        tool_metadata = Some(metadata);
        provably_legal = true;
        (values, None)
    } else if let Some((values, metadata)) = super::printer_physical_identity::generate(&key) {
        tool_metadata = Some(metadata);
        provably_legal = true;
        (values, None)
    } else if let Some((values, metadata)) = tool_domains::generate(&key) {
        tool_metadata = Some(metadata);
        // Native-witnessed H2D, two complete selectors, Manual mode, wall1/infill2.
        provably_legal = true;
        (values, None)
    } else if let Some((values, metadata)) = tool_sequence_domains::generate(&key) {
        tool_metadata = Some(metadata);
        provably_legal = true;
        (values, None)
    } else if let Some(authored) = authored_probes(&key) {
        // Per-key legal domains: free-form `coString`/`coPoint(s)`
        // probes cover an unbounded schema domain completely; NUMERIC
        // authored probes only sample the legal interior (the schema
        // min/max that upstream slice-time validation clamps are
        // intentionally not probed), so they stay INCOMPLETE — the
        // domain is exercised but the boundary claim isn't proven.
        let unbounded_schema = matches!(
            option_type.as_str(),
            "coString" | "coStrings" | "coPoint" | "coPoints"
        );
        // PrintConfig.cpp:692-696 / Config.hpp:1605-1717: the admitted
        // grouped polygons cover this bounded authored scope, not numeric endpoints.
        let grouped_printable_area =
            option_type == "coPointsGroups" && key == "extruder_printable_area";
        let bounded_height = matches!(key.as_str(), "layer_height" | "initial_layer_print_height");
        // Runtime-witnessed bridge roles, single nozzle/filament finite scope.
        let bounded_bridge = matches!(key.as_str(), "bridge_flow" | "internal_bridge_flow");
        // GCode.cpp:6467-6513 / Extruder.hpp:61: finite single-filament witnesses.
        let bounded_filament = matches!(key.as_str(), "filament_flow_ratio" | "filament_diameter");
        let mut values = authored
            .iter()
            .map(|&(label, value)| case(label, value, entry))
            .collect();
        let bounded_clearance = super::tests::clearance::admit(&key, &mut values);
        let bounded_printable_height =
            super::tests::printable_height::admission::admit(&key, &mut values);
        provably_legal = unbounded_schema
            || grouped_printable_area
            || bounded_height
            || bounded_bridge
            || bounded_filament
            || bounded_clearance
            || bounded_printable_height
            || key == "enable_long_retraction_when_cut";
        (
            values,
            (!provably_legal).then_some("authored probes sample the legal interior; schema bounds rejected by upstream validation"),
        )
    } else if is_width_option(&key) {
        let locked = matches!(
            key.as_str(),
            "skin_infill_line_width" | "skeleton_infill_line_width"
        );
        let raw = if locked {
            // These definitions inherit ConfigOptionDef's default FLT_MAX ceiling.
            (0.0, f64::from(f32::MAX))
        } else {
            raw_numeric_bounds(&block).expect("width schema must have explicit raw bounds")
        };
        let (domain, values) = widths::generate(&key, entry, raw, false, true);
        width_domain = Some(domain);
        // Print.cpp:1543-1556 / Fill.cpp:990-1000: locked-width E2Es
        // close the declared single-nozzle context, not a global width range.
        provably_legal = true;
        (values, None)
    } else if matches!(option_type.as_str(), "coBool" | "coBools") {
        // A boolean domain is exactly {0,1}; both cases are legal by
        // construction, so full comparison is the legality proof.
        // Nullable booleans additionally admit `nil` (the option is
        // then omitted from the exported config) — probe it too.
        provably_legal = true;
        let mut values = vec![case("false", "0", entry), case("true", "1", entry)];
        if entry.get("nullable").and_then(Value::as_bool) == Some(true) {
            values.push(case("nil", "nil", entry));
        }
        (values, None)
    } else if matches!(option_type.as_str(), "coEnum" | "coEnums") {
        let values = enum_values(&key, &block, source_text)
            .into_iter()
            .map(|value| case(&value, &value, entry))
            .collect::<Vec<_>>();
        // An enum's declared value set is its whole legal domain.
        provably_legal = !values.is_empty();
        let omission = values
            .is_empty()
            .then_some("enum domain not explicit in definition block");
        (values, omission)
    } else if range_type(&option_type) {
        let (cases, omission, legal) = numeric_domains::generate(entry, &key, &option_type, &block);
        provably_legal = legal;
        (cases, omission)
    } else {
        // `coPoint(s)` unbounded options (bed polygons, exclude areas)
        // reject/crash on generic probes (`printable_area` degenerate
        // polygon); baseline default remains until per-key legal domains
        // are authored. The omission reason feeds the sweep gate.
        (
            vec![baseline_case()],
            Some("unbounded domain (string/point/blob) — baseline only"),
        )
    };
    Some(OptionPlan {
        key,
        option_type,
        raw_scope,
        source,
        cases: values,
        omission,
        width_domain,
        tool_metadata,
        provably_legal,
    })
}

#[path = "numeric_boundaries.rs"]
pub(super) mod numeric_boundaries;
mod numeric_domains;
#[path = "tool_domains.rs"]
mod tool_domains;
#[path = "tool_sequence_domains.rs"]
mod tool_sequence_domains;

#[cfg(test)]
#[path = "numeric_metadata_tests.rs"]
mod numeric_metadata_tests;

#[cfg(test)]
#[path = "numeric_boundary_report_tests.rs"]
mod numeric_boundary_report_tests;

#[path = "bounds.rs"]
mod bounds;
use bounds::*;

#[path = "domains/probes.rs"]
mod probes;
use probes::authored_probes;
