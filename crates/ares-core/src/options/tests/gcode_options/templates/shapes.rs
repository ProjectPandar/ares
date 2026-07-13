use std::collections::BTreeSet;

use super::super::{gcode_rows, inventory};
use super::*;

mod filament;
mod printer;
mod process;
mod project;

macro_rules! verify_array_fields {
    (printer; $($field:ident => $key:literal = $value:expr),+ $(,)?) => {{
        let mut printer = PrinterGCodeSourceOptions::default();
        $(printer.$field = $value;)+
        let process = ProcessGCodeSourceOptions::default();
        let filament = FilamentGCodeSourceOptions::default();
        let project_source = ProjectGCodeSourceOptions::default();
        let projected = project(&printer, &process, &filament, &project_source);
        let mut verified = Vec::new();
        $(
            assert_eq!(projected.$field, printer.$field, "{} exact shape", $key);
            verified.push($key);
        )+
        verified
    }};
    (process; $($field:ident => $key:literal = $value:expr),+ $(,)?) => {{
        let printer = PrinterGCodeSourceOptions::default();
        let mut process = ProcessGCodeSourceOptions::default();
        $(process.$field = $value;)+
        let filament = FilamentGCodeSourceOptions::default();
        let project_source = ProjectGCodeSourceOptions::default();
        let projected = project(&printer, &process, &filament, &project_source);
        let mut verified = Vec::new();
        $(
            assert_eq!(projected.$field, process.$field, "{} exact shape", $key);
            verified.push($key);
        )+
        verified
    }};
    (filament; $($field:ident => $key:literal = $value:expr),+ $(,)?) => {{
        let printer = PrinterGCodeSourceOptions::default();
        let process = ProcessGCodeSourceOptions::default();
        let mut filament = FilamentGCodeSourceOptions::default();
        $(filament.$field = $value;)+
        let project_source = ProjectGCodeSourceOptions::default();
        let projected = project(&printer, &process, &filament, &project_source);
        let mut verified = Vec::new();
        $(
            assert_eq!(projected.$field, filament.$field, "{} exact shape", $key);
            verified.push($key);
        )+
        verified
    }};
    (project; $($field:ident => $key:literal = $value:expr),+ $(,)?) => {{
        let printer = PrinterGCodeSourceOptions::default();
        let process = ProcessGCodeSourceOptions::default();
        let filament = FilamentGCodeSourceOptions::default();
        let mut project_source = ProjectGCodeSourceOptions::default();
        $(project_source.$field = $value;)+
        let projected = project(&printer, &process, &filament, &project_source);
        let mut verified = Vec::new();
        $(
            assert_eq!(projected.$field, project_source.$field, "{} exact shape", $key);
            verified.push($key);
        )+
        verified
    }};
}

pub(super) use verify_array_fields;

fn bools(values: &[bool]) -> OrcaBools {
    OrcaBools(values.iter().copied().map(OrcaBool).collect())
}

fn floats(values: &[f64]) -> OrcaFloats {
    OrcaFloats(values.iter().copied().map(OrcaFloat).collect())
}

fn ints(values: &[i32]) -> OrcaInts {
    OrcaInts(values.iter().copied().map(OrcaInt).collect())
}

fn percents(values: &[f64]) -> OrcaPercents {
    OrcaPercents(values.iter().copied().map(Percent).collect())
}

fn strings(values: &[&str]) -> OrcaStrings {
    OrcaStrings(owned_strings(values))
}

fn nullable_bools(values: &[Option<bool>]) -> Vec<Nullable<OrcaBool>> {
    values
        .iter()
        .map(|value| value.map_or(Nullable::Nil, |value| Nullable::Value(OrcaBool(value))))
        .collect()
}

fn nullable_floats(values: &[Option<f64>]) -> Vec<Nullable<OrcaFloat>> {
    values
        .iter()
        .map(|value| value.map_or(Nullable::Nil, |value| Nullable::Value(OrcaFloat(value))))
        .collect()
}

fn nullable_ints(values: &[Option<i32>]) -> Vec<Nullable<OrcaInt>> {
    values
        .iter()
        .map(|value| value.map_or(Nullable::Nil, |value| Nullable::Value(OrcaInt(value))))
        .collect()
}

#[test]
fn gcode_options_shapes_preserve_all_eighty_inventory_arrays_without_selection() {
    let mut verified = printer::verify_arrays();
    verified.extend(process::verify_arrays());
    verified.extend(filament::verify_arrays());
    verified.extend(project::verify_arrays());

    let verified_set = verified.iter().copied().collect::<BTreeSet<_>>();
    assert_eq!(verified.len(), 80, "explicit field references");
    assert_eq!(verified_set.len(), 80, "duplicate explicit field references");

    let rows = inventory();
    let expected = gcode_rows(&rows)
        .into_iter()
        .filter(|row| row.wire_shape == "array")
        .map(|row| row.key.as_str())
        .collect::<BTreeSet<_>>();
    assert_eq!(expected.len(), 80, "inventory array rows");
    assert_eq!(verified_set, expected, "missing or extra array fields");
}
