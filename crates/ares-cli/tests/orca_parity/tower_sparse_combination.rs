use crate::{
    OrcaRunner,
    runner::{application, native_api_cli::NativeCase},
};
use ares_core::{ConfigOverrides, GenerationMetadata, SlicerContext};
use serde_json::{Value, json};
use std::{
    fs,
    io::{Cursor, Read, Write},
};

const TYPE2: &[u8] =
    include_bytes!("../../../../tests/parity/power-loss-recovery/empty-klipper.3mf");
const TYPE1: &[u8] = include_bytes!(
    "../../../../tests/parity/filament-group-flush-restriction-pairs/x2d_pla_both_heads_reach_models.3mf"
);

fn project(type1: bool, combine: bool) -> Vec<u8> {
    let mut source = zip::ZipArchive::new(Cursor::new(if type1 { TYPE1 } else { TYPE2 })).unwrap();
    let mut dest = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..source.len() {
        let mut file = source.by_index(index).unwrap();
        let bytes = match file.name() {
            "Metadata/project_settings.config" => {
                let mut config: Value = serde_json::from_reader(&mut file).unwrap();
                config["top_surface_filament_id"] = json!("2");
                config["layer_height"] = json!(if type1 { "0.12" } else { "0.16" });
                config["enable_prime_tower"] = json!("1");
                config["wipe_tower_sparse_layers_combination"] =
                    json!(if combine { "1" } else { "0" });
                serde_json::to_vec(&config).unwrap()
            }
            "Metadata/model_settings.config" => {
                let mut xml = String::new();
                file.read_to_string(&mut xml).unwrap();
                assert!(xml.contains("key=\"extruder\" value=\"2\""));
                xml.replace(
                    "key=\"extruder\" value=\"2\"",
                    "key=\"extruder\" value=\"1\"",
                )
                .replace(
                    "key=\"extruder\" value=\"3\"",
                    "key=\"extruder\" value=\"1\"",
                )
                .into_bytes()
            }
            _ => {
                dest.raw_copy_file(file).unwrap();
                continue;
            }
        };
        dest.start_file(file.name(), zip::write::SimpleFileOptions::default())
            .unwrap();
        dest.write_all(&bytes).unwrap();
    }
    dest.finish().unwrap().into_inner()
}

fn case(type1: bool, combine: bool) -> Option<NativeCase> {
    let runner = OrcaRunner::from_env().unwrap()?;
    let kind = if type1 { "type1" } else { "type2" };
    let mode = if combine { "on" } else { "off" };
    Some(NativeCase::slice(
        &runner,
        &project(type1, combine),
        &format!("{kind}-tower-combination-{mode}-"),
    ))
}

fn type2(combine: bool) {
    let Some(case) = case(false, combine) else {
        return;
    };
    let second_layer = case
        .native
        .split_once("\n;Z:0.36\n")
        .unwrap()
        .1
        .split_once("\n;LAYER_CHANGE\n")
        .unwrap()
        .0;
    assert_eq!(second_layer.contains(";TYPE:Prime tower"), !combine);
    case.compare(GenerationMetadata::default());
}

fn tower_layers(gcode: &str) -> Vec<(&str, Vec<&str>)> {
    gcode
        .split("\n; CHANGE_LAYER\n")
        .filter_map(|layer| {
            let z = layer
                .lines()
                .find_map(|line| line.strip_prefix("; Z_HEIGHT: "))?;
            let commands = layer
                .split("\n; WIPE_TOWER_START\n")
                .skip(1)
                .flat_map(|tower| {
                    tower
                        .split_once("\n; WIPE_TOWER_END")
                        .expect("tower end marker")
                        .0
                        .lines()
                })
                .collect::<Vec<_>>();
            (!commands.is_empty()).then_some((z, commands))
        })
        .collect()
}

fn type1(combine: bool) {
    let Some(case) = case(true, combine) else {
        return;
    };
    let input = case.root.join("input.3mf");
    let api = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(application::slice_project_plate_validated(
            &fs::read(&input).unwrap(),
            GenerationMetadata::default(),
            None,
            &ConfigOverrides::default(),
            SlicerContext::new(0),
        ))
        .unwrap();
    fs::write(case.root.join("ares.api.gcode"), &api).unwrap();
    let cli_path = case.root.join("ares.cli.gcode");
    let output = application::cli_command()
        .args(["slice", "-o"])
        .arg(&cli_path)
        .arg(input)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "CLI at {:?}: {:?}",
        case.root,
        output.status
    );
    let cli = fs::read(cli_path).unwrap();
    let expected = tower_layers(&case.native);
    let heights = |layers: &[(&str, Vec<&str>)]| {
        layers
            .iter()
            .map(|(z, _)| (*z).to_owned())
            .collect::<Vec<_>>()
    };
    assert_eq!(expected.iter().any(|(z, _)| *z == "0.32"), !combine);
    let receiving = |layers: Vec<(&str, Vec<&str>)>| {
        layers
            .into_iter()
            .find(|(z, _)| *z == "0.44")
            .expect("tower receiving layer")
            .1
            .iter()
            .map(|s| (*s).to_owned())
            .collect::<Vec<_>>()
    };
    for (producer, gcode) in [("API", api), ("CLI", cli)] {
        let gcode = std::str::from_utf8(&gcode).unwrap();
        let actual = tower_layers(gcode);
        assert_eq!(
            heights(&actual),
            heights(&expected),
            "{producer} tower-bearing model layers at {:?}",
            case.root
        );
        assert_eq!(
            receiving(actual),
            receiving(expected.clone()),
            "{producer} tower receiving layer at {:?}",
            case.root
        );
    }
}

#[test]
fn type2_tower_combines_sparse_layers() {
    type2(true);
}

#[test]
fn type2_tower_preserves_sparse_layers_when_combination_is_off() {
    type2(false);
}

#[test]
fn type1_tower_combines_sparse_layers() {
    type1(true);
}

#[test]
fn type1_tower_preserves_sparse_layers_when_combination_is_off() {
    type1(false);
}
