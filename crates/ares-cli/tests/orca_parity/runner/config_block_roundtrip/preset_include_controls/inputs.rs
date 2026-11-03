use super::super::super::repo_root;
use serde_json::{Value, json};
use std::{
    fs,
    io::{Cursor, Write},
    path::Path,
};

pub(super) const PRESET: &str = "Bambu PLA Basic @BBL X2D 0.4 nozzle";
const LEAF: &str = "filament/Bambu PLA Basic @BBL X2D 0.4 nozzle.json";

pub(super) fn prepare(work: &Path, case: &str) -> Value {
    let mut catalog: Value = serde_json::from_slice(
        &fs::read(repo_root().join("tests/parity/preset-includes-e3d/catalog.json")).unwrap(),
    )
    .unwrap();
    let vendor = &mut catalog["vendors"][0];
    let variants = vendor["files"]["filament/fdm_filament_template_direct_bowden_e3d.json"]["filament_extruder_variant"].clone();
    for (name, speed) in [("include_control_a", "15"), ("include_control_b", "17")] {
        let mut template = json!({
            "type": "filament", "name": name, "instantiation": "false",
            "inherits": "fdm_filament_common", "filament_extruder_variant": variants,
            "filament_max_volumetric_speed": vec![speed; 6],
        });
        if name == "include_control_a" {
            template["filament_flow_ratio"] = json!(["1.0"]);
        }
        vendor["files"][format!("filament/{name}.json")] = template;
    }
    let rows = vendor["index"]["filament_list"].as_array_mut().unwrap();
    let position = rows
        .iter()
        .position(|entry| entry["sub_path"] == LEAF)
        .unwrap();
    for (offset, name) in ["include_control_a", "include_control_b"]
        .into_iter()
        .enumerate()
    {
        rows.insert(
            position + offset,
            json!({"name": name, "sub_path": format!("filament/{name}.json")}),
        );
    }
    vendor["files"]["filament/Bambu PLA Basic @base.json"]["filament_flow_ratio"] =
        json!(vec!["1.1"; 6]);
    let leaf = vendor["files"][LEAF].as_object_mut().unwrap();
    leaf.remove("filament_max_volumetric_speed");
    leaf.remove("filament_flow_ratio");
    leaf.insert(
        "include".into(),
        match case {
            "no-include" => json!([]),
            "reverse" => json!(["include_control_b", "include_control_a"]),
            "pre-padding-default" => json!(["include_control_a"]),
            "missing-include" => json!(["unavailable_include", "include_control_b"]),
            "cross-vendor-include" => json!(["include_control_a", "include_control_foreign"]),
            _ => json!(["include_control_a", "include_control_b"]),
        },
    );
    if case == "no-include" {
        leaf.insert("filament_extruder_variant".into(), variants);
        leaf.insert("filament_max_volumetric_speed".into(), json!(vec!["21"; 6]));
    }
    if case == "leaf-precedence" {
        leaf.insert("filament_max_volumetric_speed".into(), json!(vec!["19"; 6]));
    }
    if case == "forward-include" {
        let rows = vendor["index"]["filament_list"].as_array_mut().unwrap();
        let position = rows
            .iter()
            .position(|entry| entry["name"] == "include_control_b")
            .unwrap();
        let entry = rows.remove(position);
        rows.push(entry);
    }
    let metadata: Value = serde_json::from_slice(
        &fs::read(repo_root().join("tests/parity/preset-includes-e3d/native-vendor-metadata.json"))
            .unwrap(),
    )
    .unwrap();
    let mut index = metadata["index"].clone();
    for kind in ["machine_list", "process_list", "filament_list"] {
        index[kind] = vendor["index"][kind].clone();
    }
    let system = work.join("data/system");
    fs::create_dir_all(&system).unwrap();
    fs::write(
        system.join("BBL.json"),
        serde_json::to_vec_pretty(&index).unwrap(),
    )
    .unwrap();
    let mut documents = vendor["files"].as_object().unwrap().clone();
    documents.insert(
        metadata["model_path"].as_str().unwrap().to_owned(),
        metadata["model"].clone(),
    );
    for (path, document) in documents {
        let path = system.join("BBL").join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, serde_json::to_vec_pretty(&document).unwrap()).unwrap();
    }
    if case == "cross-vendor-include" {
        catalog["vendors"].as_array_mut().unwrap().push(json!({
            "id": "Other", "index": {"name": "Other", "filament_list": [{"name": "include_control_foreign", "sub_path": "foreign.json"}]},
            "files": {"foreign.json": {"type": "filament", "name": "include_control_foreign", "instantiation": "false", "filament_max_volumetric_speed": vec!["29"; 6]}}
        }));
    }
    fs::write(
        work.join("catalog-input.json"),
        serde_json::to_vec_pretty(&catalog).unwrap(),
    )
    .unwrap();
    let project =
        fs::read(repo_root().join("tests/parity/printer-variant-order/model-ironing-gyroid.3mf"))
            .unwrap();
    let mut source = zip::ZipArchive::new(Cursor::new(project)).unwrap();
    let mut target = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for i in 0..source.len() {
        let entry = source.by_index(i).unwrap();
        if entry.name() == "Metadata/project_settings.config" {
            let mut settings: Value = serde_json::from_reader(entry).unwrap();
            let count = settings["filament_settings_id"].as_array().unwrap().len() + 2;
            settings["different_settings_to_system"] = json!(vec![""; count]);
            target
                .start_file(
                    "Metadata/project_settings.config",
                    zip::write::SimpleFileOptions::default(),
                )
                .unwrap();
            target
                .write_all(&serde_json::to_vec(&settings).unwrap())
                .unwrap();
        } else {
            target.raw_copy_file(entry).unwrap();
        }
    }
    fs::write(
        work.join("control-input.3mf"),
        target.finish().unwrap().into_inner(),
    )
    .unwrap();
    catalog
}
