use crate::{
    FloatOrPercent, Project,
    project::effective_config::resolve_bounded_project_config,
    project_slice::{task22n_browser_input_oracle, task22n_browser_oracle},
};

use super::archive::{ArchiveBuilder, assert_single_entry_replacement, semantic_identity};
use crate::project_slice::tests::perimeters::oracle::{NRecord, WireFlow, parse_n};

mod bridges;
mod oracle;
mod selectors;
mod widths;

use oracle::*;

const PROCESS: &str = "Metadata/project_settings.config";
const MODEL: &str = "Metadata/model_settings.config";
const FIRST_LAYER: u8 = 1;
const BOTH_LAYERS: u8 = 3;
const INTERNAL_PERIMETER_ROLE: u8 = 1;
const EXTERNAL_PERIMETER_ROLE: u8 = 2;
const OVERHANG_ROLE: u8 = 4;
const SOLID_INFILL_ROLE: u8 = 8;

#[derive(Clone, Copy)]
struct Edit {
    path: &'static str,
    from: &'static str,
    to: &'static str,
}

const fn process(from: &'static str, to: &'static str) -> Edit {
    Edit {
        path: PROCESS,
        from,
        to,
    }
}

const INITIAL_ZERO: Edit = process(
    r#""initial_layer_line_width": "0.5""#,
    r#""initial_layer_line_width": "0""#,
);
const NOZZLES_46: Edit = process(
    "\"nozzle_diameter\": [\r\n\t\t\"0.4\",\r\n\t\t\"0.4\"\r\n\t]",
    "\"nozzle_diameter\": [\r\n\t\t\"0.4\",\r\n\t\t\"0.6\"\r\n\t]",
);
const OUTER_TWO: Edit = process(
    r#""outer_wall_filament_id": "0""#,
    r#""outer_wall_filament_id": "2""#,
);
const INNER_TWO: Edit = process(
    r#""inner_wall_filament_id": "0""#,
    r#""inner_wall_filament_id": "2""#,
);
const SOLID_TWO: Edit = process(
    r#""internal_solid_filament_id": "0""#,
    r#""internal_solid_filament_id": "2""#,
);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Key {
    Initial,
    OuterWidth,
    InnerWidth,
    SolidWidth,
    ObjectWidth,
    OuterSelector,
    InnerSelector,
    SolidSelector,
    ScopedOuter,
    Nozzles,
    BridgeWidth,
    BridgeFlow,
    Thick,
    FilamentMap,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Value {
    Width(bool, u64),
    Int(i32),
    Float(u64),
    Bool(bool),
    Pair([u64; 2]),
    IntPair([i32; 2]),
}

const fn absolute(value: f64) -> Value {
    Value::Width(false, value.to_bits())
}

const fn percent(value: f64) -> Value {
    Value::Width(true, value.to_bits())
}

const fn float(value: f64) -> Value {
    Value::Float(value.to_bits())
}

const fn pair(a: f64, b: f64) -> Value {
    Value::Pair([a.to_bits(), b.to_bits()])
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Bits([u32; 4], bool, u64);

const fn bits(fields: [u32; 4], bridge: bool, volume: u64) -> Bits {
    Bits(fields, bridge, volume)
}

#[derive(Clone, Copy)]
struct Change {
    layers: u8,
    roles: u8,
    before: Bits,
    after: Bits,
}

const fn change(layers: u8, roles: u8, before: Bits, after: Bits) -> Change {
    Change {
        layers,
        roles,
        before,
        after,
    }
}

struct OptionPair<'a> {
    name: &'a str,
    setup: &'a [Edit],
    delta: Edit,
    key: Key,
    raw: [Value; 2],
    effective: [Value; 2],
    changes: &'a [Change],
}

fn run(pair: OptionPair<'_>) {
    let OptionPair {
        name,
        setup,
        delta,
        key,
        raw,
        effective,
        changes,
    } = pair;
    let before = archive(setup, None);
    let after = archive(setup, Some(delta));
    assert_single_entry_replacement(&before, &after, delta.path, delta.from, delta.to);
    assert_ne!(
        semantic_identity(&before),
        semantic_identity(&after),
        "{name}"
    );
    assert_eq!(
        [loaded(&before, key), loaded(&after, key)],
        [(raw[0], effective[0]), (raw[1], effective[1])],
        "{name}"
    );
    let before_m = task22n_browser_input_oracle(&before).unwrap();
    let after_m = task22n_browser_input_oracle(&after).unwrap();
    assert_eq!(before_m, after_m, "{name}: Task 22M");
    let before_n = task22n_browser_oracle(&before).unwrap();
    let after_n = task22n_browser_oracle(&after).unwrap();
    if changes.is_empty() {
        assert_eq!(before_n, after_n, "{name}: invariant N");
    }
    let before = parse_n(&before_n).unwrap();
    let after = parse_n(&after_n).unwrap();
    let ([before], [after]) = (before.objects.as_slice(), after.objects.as_slice()) else {
        panic!("{name}: one object")
    };
    assert_eq!(
        (before.source, before.transform, before.planned),
        (after.source, after.transform, after.planned)
    );
    assert_eq!(before.slots.len(), 2, "{name}: two archive layers");
    for (layer, (before, after)) in before.slots.iter().zip(&after.slots).enumerate() {
        let (Some(before), Some(after)) = (before, after) else {
            panic!("{name}: populated slot")
        };
        assert_context(before, after, name);
        for role in 0..4 {
            let selected = changes
                .iter()
                .filter(|change| {
                    change.layers & (1 << layer) != 0 && change.roles & (1 << role) != 0
                })
                .collect::<Vec<_>>();
            assert!(selected.len() <= 1, "{name}: overlapping oracle changes");
            let actual = (flow(before.flows[role]), flow(after.flows[role]));
            assert_eq!(
                actual,
                selected
                    .first()
                    .map_or((actual.0, actual.0), |change| (change.before, change.after)),
                "{name}: layer {layer} role {role}"
            );
        }
    }
}

fn archive(setup: &[Edit], delta: Option<Edit>) -> Vec<u8> {
    let mut archive = ArchiveBuilder::new();
    for edit in setup.iter().copied().chain(delta) {
        archive.replace_unique(edit.path, edit.from, edit.to);
    }
    archive.bytes()
}

fn loaded(bytes: &[u8], key: Key) -> (Value, Value) {
    let project = crate::load_project(bytes).unwrap();
    let raw = raw_value(&project, key);
    let resolved = resolve_bounded_project_config(&project).unwrap();
    let object = &resolved.objects[0];
    let region = &object.layer_candidates[0].model_parts[0].region;
    let full = &resolved.views.full;
    let effective = match key {
        Key::Initial => width(full.process.print.initial_layer_line_width),
        Key::OuterWidth => width(region.outer_wall_line_width),
        Key::InnerWidth => width(region.inner_wall_line_width),
        Key::SolidWidth => width(region.internal_solid_infill_line_width),
        Key::ObjectWidth => width(object.object.line_width),
        Key::OuterSelector | Key::ScopedOuter => Value::Int(region.outer_wall_filament_id.0),
        Key::InnerSelector => Value::Int(region.inner_wall_filament_id.0),
        Key::SolidSelector => Value::Int(region.internal_solid_filament_id.0),
        Key::Nozzles => float_pair(&full.project.print.nozzle_diameter.0),
        Key::BridgeWidth => width(region.bridge_line_width),
        Key::BridgeFlow => Value::Float(region.bridge_flow.0.to_bits()),
        Key::Thick => Value::Bool(object.object.thick_bridges.0),
        Key::FilamentMap => int_pair(&full.project.gcode.filament_map.0),
    };
    (raw, effective)
}

fn raw_value(project: &Project, key: Key) -> Value {
    let settings = project.settings();
    match key {
        Key::Initial => width(settings.process.print.initial_layer_line_width),
        Key::OuterWidth => width(settings.process.region.outer_wall_line_width),
        Key::InnerWidth => width(settings.process.region.inner_wall_line_width),
        Key::SolidWidth => width(settings.process.region.internal_solid_infill_line_width),
        Key::ObjectWidth => width(settings.process.object.line_width),
        Key::OuterSelector => Value::Int(settings.process.region.outer_wall_filament_id.0),
        Key::InnerSelector => Value::Int(settings.process.region.inner_wall_filament_id.0),
        Key::SolidSelector => Value::Int(settings.process.region.internal_solid_filament_id.0),
        Key::ScopedOuter => Value::Int(
            project.objects()[0].volumes()[0]
                .region_overrides()
                .outer_wall_filament_id
                .unwrap()
                .0,
        ),
        Key::Nozzles => float_pair(&settings.project.print.nozzle_diameter.0),
        Key::BridgeWidth => width(settings.process.region.bridge_line_width),
        Key::BridgeFlow => Value::Float(settings.process.region.bridge_flow.0.to_bits()),
        Key::Thick => Value::Bool(settings.process.object.thick_bridges.0),
        Key::FilamentMap => int_pair(&settings.project.gcode.filament_map.0),
    }
}

fn width(value: FloatOrPercent) -> Value {
    match value {
        FloatOrPercent::Float(value) => absolute(value),
        FloatOrPercent::Percent(value) => percent(value.0),
    }
}

fn float_pair(values: &[crate::OrcaFloat]) -> Value {
    let [a, b] = values else {
        panic!("two nozzles")
    };
    pair(a.0, b.0)
}

fn int_pair(values: &[crate::OrcaInt]) -> Value {
    let [a, b] = values else {
        panic!("two filament mappings")
    };
    Value::IntPair([a.0, b.0])
}

fn flow(value: WireFlow) -> Bits {
    Bits(value.fields, value.bridge, value.mm3_per_mm)
}

fn assert_context(left: &NRecord, right: &NRecord, name: &str) {
    assert_eq!(
        (
            left.source,
            left.transform,
            left.planned,
            left.layer,
            left.region
        ),
        (
            right.source,
            right.transform,
            right.planned,
            right.layer,
            right.region
        ),
        "{name}"
    );
    assert_eq!(
        (
            &left.compatible,
            left.current,
            left.lower,
            left.upper,
            left.upper_same
        ),
        (
            &right.compatible,
            right.current,
            right.lower,
            right.upper,
            right.upper_same
        ),
        "{name}"
    );
    assert_eq!(
        (
            &left.current_surfaces,
            &left.lower_slices,
            &left.upper_slices,
            &left.upper_same_surfaces
        ),
        (
            &right.current_surfaces,
            &right.lower_slices,
            &right.upper_slices,
            &right.upper_same_surfaces
        ),
        "{name}"
    );
    assert_eq!(
        (
            left.height,
            left.slice_z,
            left.spiral,
            left.rotation,
            left.dispatch
        ),
        (
            right.height,
            right.slice_z,
            right.spiral,
            right.rotation,
            right.dispatch
        ),
        "{name}"
    );
}
