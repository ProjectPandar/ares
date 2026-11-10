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

/// The base unchanged: its only buffered delay is the first `G29`
/// (`docs/architecture/m73-block-time-precision-spec.md`).
#[test]
fn derived_base_places_progress_like_native() {
    let project = base(|text| {
        let config: serde_json::Value = serde_json::from_str(text).unwrap();
        assert_eq!(config["ironing_type"], "no ironing", "base must not iron");
        assert!(
            config.get("top_surface_expansion").is_none(),
            "base must not set `top_surface_expansion`"
        );
        text.to_owned()
    });
    let Some((runner, case)) = prepare((PREFIX, "derived-base", "no ironing"), project) else {
        return;
    };
    assert_matches_native(&runner, &case, |reference| {
        let gcode = String::from_utf8_lossy(reference);
        let commands = gcode
            .lines()
            .map(|line| line.split(';').next().unwrap_or_default())
            .collect::<Vec<_>>();
        assert!(
            commands.iter().any(|code| code.starts_with("G29 ")),
            "Native must run `G29`"
        );
        let delay = commands.iter().find(|code| {
            let mut words = code.split_whitespace();
            match words.next() {
                Some("G4" | "M400") => words.any(|word| word.starts_with(['S', 'P'])),
                Some("SYNC") => true,
                _ => false,
            }
        });
        assert!(
            delay.is_none(),
            "Native must not wait outside `G29`: {delay:?}"
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
