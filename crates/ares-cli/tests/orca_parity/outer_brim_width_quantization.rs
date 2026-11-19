//! Whole-file parity of the explicit outer brim at adjacent first-layer widths.

use std::{fs, path::Path};

use ares_core::{ConfigOverrides, LoadSettingsFile, LoadSettingsSliceRequest, Placement};
use serde_json::{Map, Value, json};

use crate::{golden, runner::OrcaRunner};

const X1C: [&str; 4] = [
    "BBL",
    "Bambu Lab X1 Carbon 0.4 nozzle",
    "0.20mm Standard @BBL X1C",
    "Bambu PLA Basic @BBL X1C",
];
const K1: [&str; 4] = [
    "Creality",
    "Creality K1 (0.4 nozzle)",
    "0.20mm Standard @Creality K1 (0.4 nozzle)",
    "Generic PLA HF @Creality",
];
const MK4: [&str; 4] = [
    "Prusa",
    "Prusa MK4 0.4 nozzle",
    "0.20mm Standard @MK4",
    "Generic PLA @Prusa MK4",
];

#[tokio::test]
async fn x1c_outer_brim_preserves_six_spacings_at_half_millimeter() {
    check(X1C, "0.50").await;
}

#[tokio::test]
async fn k1_outer_brim_preserves_six_spacings_at_half_millimeter() {
    check(K1, "0.50").await;
}

#[tokio::test]
async fn mk4_outer_brim_preserves_six_spacings_at_half_millimeter() {
    check(MK4, "0.50").await;
}

#[tokio::test]
async fn x1c_outer_brim_matches_at_fifty_one_hundredths() {
    check(X1C, "0.51").await;
}

#[tokio::test]
async fn k1_outer_brim_matches_at_fifty_one_hundredths() {
    check(K1, "0.51").await;
}

#[tokio::test]
async fn mk4_outer_brim_matches_at_fifty_one_hundredths() {
    check(MK4, "0.51").await;
}

async fn check([vendor, machine, process, filament]: [&str; 4], line_width: &str) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let profiles_root = crate::runner::repo_root().join("OrcaSlicer/resources/profiles");
    let mut process = stock_profile(&profiles_root, vendor, "process", process);
    for (key, value) in [
        ("initial_layer_line_width", line_width),
        ("brim_type", "outer_only"),
        ("brim_width", "3.6"),
        ("enable_support", "0"),
        ("raft_layers", "0"),
    ] {
        process.insert(key.to_owned(), json!(value));
    }
    let labels = ["machine.json", "process.json", "filament.json"];
    let settings = [
        stock_profile(&profiles_root, vendor, "machine", machine),
        process,
        stock_profile(&profiles_root, vendor, "filament", filament),
    ]
    .map(|fields| serde_json::to_string(&fields).unwrap());
    let work = tempfile::Builder::new()
        .prefix("outer-brim-width-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let paths = labels.map(|label| work.join(label));
    for (path, fields) in paths.iter().zip(&settings) {
        fs::write(path, fields).unwrap();
    }
    let plate = flat_plate();
    let filename = "flat.stl";
    let model = work.join(filename);
    fs::write(&model, &plate).unwrap();
    let native = runner
        .slice_stl_with_settings(paths.each_ref().map(|path| path.as_path()), &model)
        .unwrap();
    fs::write(work.join("native.gcode"), &native).unwrap();
    let files = std::array::from_fn::<_, 3, _>(|i| LoadSettingsFile {
        label: labels[i],
        json: &settings[i],
        source_identity: None,
    });
    let ares = ares_core::slice_with_load_settings(
        &plate,
        LoadSettingsSliceRequest {
            orient: false,
            filename,
            catalog_json: None,
            loaded_sources: None,
            uptodate: false,
            settings: &files[..2],
            filaments: &files[2..],
            plate: None,
            overrides: &ConfigOverrides::default(),
            placement: Placement::None,
            sliced_plate: &mut ares_core::SlicedPlate::default(),
        },
        ares_core::GenerationMetadata::deterministic(1980, 1, 1, 0, 0, 0),
        ares_core::SlicerContext::new(0),
    )
    .await
    .unwrap();
    fs::write(work.join("ares.gcode"), &ares).unwrap();
    let native_text = std::str::from_utf8(&native).unwrap();
    assert!(
        native_text
            .lines()
            .any(|line| matches!(line, ";TYPE:Brim" | "; FEATURE: Brim"))
    );
    assert!(native_text.contains("; brim_width = 3.6\n"));
    assert!(native_text.contains(&format!(
        "; initial_layer_line_width = {}\n",
        line_width.trim_end_matches('0')
    )));
    if let Err(detail) = golden::compare_ordered_bytes_generator_only(&native, &ares) {
        panic!("{machine}, {line_width} mm: {detail}; {work:?}");
    }
}

fn stock_profile(root: &Path, vendor: &str, kind: &str, name: &str) -> Map<String, Value> {
    let filename = format!("{name}.json");
    let path = root.join(vendor).join(kind).join(&filename);
    let path = if kind == "filament" && !path.is_file() {
        root.join("OrcaFilamentLibrary").join(kind).join(filename)
    } else {
        path
    };
    let mut fields: Map<String, Value> = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
    let mut resolved = Map::new();
    if let Some(parent) = fields.remove("inherits") {
        resolved.extend(stock_profile(root, vendor, kind, parent.as_str().unwrap()));
    }
    // Stock includes supply printer and filament templates, including start G-code.
    if let Some(includes) = fields.remove("include") {
        for include in includes.as_array().unwrap() {
            resolved.extend(stock_profile(root, vendor, kind, include.as_str().unwrap()));
        }
    }
    resolved.extend(fields);
    resolved
}

fn flat_plate() -> Vec<u8> {
    let corners: [[f32; 3]; 8] = [
        [100., 100., 0.],
        [114., 100., 0.],
        [114., 112., 0.],
        [100., 112., 0.],
        [100., 100., 3.2],
        [114., 100., 3.2],
        [114., 112., 3.2],
        [100., 112., 3.2],
    ];
    let triangles = [
        [0, 2, 1],
        [0, 3, 2],
        [4, 5, 6],
        [4, 6, 7],
        [0, 1, 5],
        [0, 5, 4],
        [1, 2, 6],
        [1, 6, 5],
        [2, 3, 7],
        [2, 7, 6],
        [3, 0, 4],
        [3, 4, 7],
    ];
    let mut stl = vec![0; 80];
    stl.extend_from_slice(&12_u32.to_le_bytes());
    for triangle in triangles {
        stl.extend_from_slice(&[0; 12]);
        for point in triangle {
            for axis in corners[point] {
                stl.extend_from_slice(&axis.to_le_bytes());
            }
        }
        stl.extend_from_slice(&[0; 2]);
    }
    stl
}
