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
    let inventory: Value = serde_json::from_str(
        &std::fs::read_to_string(repo.join("tests/ksr_fdmtest_v4/options-v242.json")).unwrap(),
    )
    .unwrap();
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
    let mut provably_legal = false;
    let (values, mut omission) = if let Some(authored) = authored_probes(&key) {
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
        provably_legal = unbounded_schema;
        (
            authored
                .iter()
                .map(|&(label, value)| case(label, value, entry))
                .collect(),
            (!unbounded_schema).then_some(
                if key == "enable_long_retraction_when_cut" {
                    "named modes covered; unbounded coInt endpoints remain unverified"
                } else {
                    "authored probes sample the legal interior; schema bounds rejected by upstream validation"
                },
            ),
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
        provably_legal = !locked;
        (
            values,
            locked.then_some(
                "single-nozzle context covered; global range endpoints remain unverified",
            ),
        )
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
    // Config.hpp:2475-2479 supplies FLT_MAX when resolution omits its max.
    if key == "resolution" {
        provably_legal = false;
        omission = Some("implicit FLT_MAX ceiling and full-range interior remain unverified");
    }
    Some(OptionPlan {
        key,
        option_type,
        raw_scope,
        source,
        cases: values,
        omission,
        width_domain,
        provably_legal,
    })
}

mod numeric_domains;

#[path = "bounds.rs"]
mod bounds;
use bounds::*;

#[path = "domains/probes.rs"]
mod probes;
use probes::authored_probes;
