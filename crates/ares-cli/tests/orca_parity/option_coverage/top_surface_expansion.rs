//! `PrintObject::detect_surfaces_type` grows each layer region's top surfaces by
//! `top_surface_expansion` (`PrintObject.cpp:2112-2172`), on the single-nozzle base of the
//! ironing legs: its KSR model has top surfaces with raised features. The last two legs keep
//! the gate (`:2121-2122`). See `docs/architecture/top-surface-expansion-spec.md`.

use super::ironing_pattern::{assert_matches_native, prepare, single_nozzle};
use serde_json::{Map, Value};

const PREFIX: &str = "top-surface-expansion";
const KEYS: [&str; 3] = [
    "top_surface_expansion",
    "top_surface_expansion_margin",
    "top_surface_expansion_direction",
];

fn project(settings: &[(&str, &str)]) -> Vec<u8> {
    single_nozzle::base(|text| {
        let mut config: Map<String, Value> = serde_json::from_str(text).unwrap();
        assert_eq!(
            config["wall_generator"], "arachne",
            "base must use Arachne walls"
        );
        assert_eq!(
            config["only_one_wall_top"], "0",
            "base must keep every wall"
        );
        for key in KEYS {
            assert!(!config.contains_key(key), "base must not set `{key}`");
        }
        for &(key, value) in settings {
            config.insert(key.to_owned(), Value::from(value));
        }
        serde_json::to_string_pretty(&config).unwrap()
    })
}

fn run(label: &'static str, settings: &[(&str, &str)]) {
    let Some((runner, case)) = prepare((PREFIX, label, label), project(settings)) else {
        return;
    };
    let expansion = settings
        .iter()
        .find(|(key, _)| *key == "top_surface_expansion")
        .map(|(_, value)| *value)
        .unwrap();
    assert_matches_native(&runner, &case, |reference| {
        let echo = format!("; top_surface_expansion = {expansion}");
        assert!(
            String::from_utf8_lossy(reference)
                .lines()
                .any(|line| line == echo),
            "Native must echo `{echo}`"
        );
    });
}

#[test]
fn expansion_grows_top_surfaces_inward_and_outward_like_native() {
    run("inward-and-outward", &[("top_surface_expansion", "2")]);
}

#[test]
fn expansion_grows_top_surfaces_inward_like_native() {
    run(
        "inward",
        &[
            ("top_surface_expansion", "2"),
            ("top_surface_expansion_direction", "inward"),
        ],
    );
}

#[test]
fn expansion_grows_top_surfaces_outward_like_native() {
    run(
        "outward",
        &[
            ("top_surface_expansion", "2"),
            ("top_surface_expansion_direction", "outward"),
        ],
    );
}

#[test]
fn expansion_keeps_the_wall_margin_like_native() {
    run(
        "margin",
        &[
            ("top_surface_expansion", "2"),
            ("top_surface_expansion_margin", "1"),
        ],
    );
}

#[test]
fn expansion_needs_top_surface_density_like_native() {
    run(
        "no-top-density",
        &[
            ("top_surface_expansion", "2"),
            ("top_surface_density", "0%"),
        ],
    );
}

#[test]
fn expansion_needs_top_shell_layers_like_native() {
    run(
        "no-top-shells",
        &[("top_surface_expansion", "2"), ("top_shell_layers", "0")],
    );
}
