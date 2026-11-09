//! Object ironing patterns on a single-nozzle base built at runtime from two committed
//! fixtures: the KSR model, model settings and plate of the X1C Arachne fixture, with the
//! project configuration of the MK4 rear-seam control
//! (`docs/architecture/ironing-single-nozzle-legs-spec.md`).

use super::{PREFIX, SETTINGS, assert_matches_native, iron_top, prepare, rewrite_settings};
use std::io::{Cursor, Read};

const MESH: &[u8] = include_bytes!(
    "../../../../../../tests/parity/wall-generator-arachne/x1c-wall-generator-arachne.3mf"
);
const MACHINE: &[u8] =
    include_bytes!("../../../../../../tests/parity/rear-seam-visibility/aligned-control.3mf");

fn machine_settings() -> String {
    let mut archive = zip::ZipArchive::new(Cursor::new(MACHINE)).unwrap();
    let mut text = String::new();
    archive
        .by_name(SETTINGS)
        .unwrap()
        .read_to_string(&mut text)
        .unwrap();
    text
}

/// The derived base with its MK4 configuration passed through `edit`.
pub(in super::super) fn base(edit: impl FnOnce(&str) -> String) -> Vec<u8> {
    let settings = edit(&machine_settings());
    let config: serde_json::Value = serde_json::from_str(&settings).unwrap();
    assert_eq!(
        config["nozzle_diameter"].as_array().map(Vec::len),
        Some(1),
        "derived base must have a single nozzle"
    );
    rewrite_settings(MESH, |_| settings)
}

fn project(pattern: &str) -> Vec<u8> {
    base(|text| iron_top(text, pattern))
}

/// Layers (`;LAYER_CHANGE`) whose body holds a `;TYPE:Ironing` block.
fn ironed_layers(gcode: &[u8]) -> usize {
    String::from_utf8_lossy(gcode)
        .split(";LAYER_CHANGE\n")
        .skip(1)
        .filter(|layer| layer.lines().any(|line| line == ";TYPE:Ironing"))
        .count()
}

fn run(label: &'static str) {
    let Some((runner, case)) = prepare((PREFIX, label, label), project(label)) else {
        return;
    };
    assert_matches_native(&runner, &case, |reference| {
        let layers = ironed_layers(reference);
        assert!(
            layers > 1,
            "Native must iron more than one layer with `{label}`, got {layers}"
        );
    });
}

macro_rules! legs {
    ($($name:ident => $pattern:literal,)*) => {$(#[test] fn $name() { run($pattern); })*};
}

legs! {
    monotonic_ironing_matches_native => "monotonic",
    monotonicline_ironing_matches_native => "monotonicline",
    alignedrectilinear_ironing_matches_native => "alignedrectilinear",
    line_ironing_matches_native => "line",
    grid_ironing_matches_native => "grid",
    triangles_ironing_matches_native => "triangles",
    tri_hexagon_ironing_matches_native => "tri-hexagon",
    cubic_ironing_matches_native => "cubic",
    quartercubic_ironing_matches_native => "quartercubic",
    honeycomb_ironing_matches_native => "honeycomb",
    lateral_honeycomb_ironing_matches_native => "lateral-honeycomb",
    crosshatch_ironing_matches_native => "crosshatch",
    tpmsd_ironing_matches_native => "tpmsd",
    tpmsfk_ironing_matches_native => "tpmsfk",
    gyroid_ironing_matches_native => "gyroid",
}
