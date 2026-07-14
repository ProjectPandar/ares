use std::collections::{BTreeMap, BTreeSet};

use serde_json::{Map, Value};

use super::super::{gcode_rows, inventory};
use super::super::super::super::{
    FilamentGCodeSourceOptions, GCodeOptions, PrinterGCodeSourceOptions,
    ProcessGCodeSourceOptions, ProjectGCodeSourceOptions,
};

pub(super) struct Fixture {
    pub(super) raw: Map<String, Value>,
    pub(super) source_keys: BTreeMap<&'static str, BTreeSet<String>>,
    pub(super) printer: PrinterGCodeSourceOptions,
    pub(super) process: ProcessGCodeSourceOptions,
    pub(super) filament: FilamentGCodeSourceOptions,
    pub(super) project: ProjectGCodeSourceOptions,
    pub(super) projected: GCodeOptions,
}

pub(super) fn load_fixture() -> Fixture {
    let raw = super::super::super::project_fixture::project_settings_value()
        .as_object()
        .unwrap()
        .clone();
    let inventory = inventory();
    let rows = gcode_rows(&inventory);
    let mut printer_map = Map::new();
    let mut process_map = Map::new();
    let mut filament_map = Map::new();
    let mut project_map = Map::new();

    for row in rows {
        let destination = match row.raw_scope.as_str() {
            "printer" => &mut printer_map,
            "process" => &mut process_map,
            "filament" => &mut filament_map,
            "residual" => &mut project_map,
            scope => panic!("unexpected G-code raw scope {scope}"),
        };
        assert!(
            destination
                .insert(row.key.clone(), raw[&row.key].clone())
                .is_none(),
            "duplicate fixture key {}",
            row.key
        );
    }

    let source_keys = BTreeMap::from([
        ("printer", printer_map.keys().cloned().collect()),
        ("process", process_map.keys().cloned().collect()),
        ("filament", filament_map.keys().cloned().collect()),
        ("residual", project_map.keys().cloned().collect()),
    ]);
    let printer = serde_json::from_value(Value::Object(printer_map)).unwrap();
    let process = serde_json::from_value(Value::Object(process_map)).unwrap();
    let filament = serde_json::from_value(Value::Object(filament_map)).unwrap();
    let project = serde_json::from_value(Value::Object(project_map)).unwrap();
    let projected = GCodeOptions::from_sources(&printer, &process, &filament, &project);

    Fixture {
        raw,
        source_keys,
        printer,
        process,
        filament,
        project,
        projected,
    }
}
