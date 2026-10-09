//! G4 admission input: Manual maps belong to the project and plate owners.
use super::*;
use crate::runner::{CaseInputs, ExportedCase};
use serde_json::json;
use std::io::{Cursor, Read, Write};

// Failure inventory: export can shrink the palette/map or overwrite plate mode;
// two configured filaments need real wall/infill usage, not config-only proof.
// Keep the stock tool-change commands: their mapped retraction is the witness.
pub(super) fn mapping(runner: &OrcaRunner, label: &str, requested: &Value) -> ExportedCase {
    let profiles = VendorProfiles::load(&profiles_root(), "BBL").unwrap();
    let selection = parity::select_printer(&profiles, "BBL", "Bambu Lab H2D 0.4 nozzle").unwrap();
    let mut machine = profiles.machine(&selection.printer).unwrap();
    machine.insert("filament_map_mode".into(), json!("Manual"));
    machine.insert("filament_map".into(), requested.clone());
    machine.insert("use_relative_e_distances".into(), json!("1"));
    machine.insert(
        "retract_length_toolchange".into(),
        json!(["1", "1", "2", "2"]),
    );
    let start = format!(
        "{}\n; G4_ACTIVE F{{initial_extruder}} N{{filament_map[initial_extruder]}}\n",
        machine["machine_start_gcode"].as_str().unwrap()
    );
    machine.insert("machine_start_gcode".into(), json!(start));
    let change = format!(
        "{}\n; G4_ACTIVE F{{next_extruder}} N{{filament_map[next_extruder]}}\n",
        machine["change_filament_gcode"].as_str().unwrap()
    );
    machine.insert("change_filament_gcode".into(), json!(change));
    let mut process = profiles.process(&selection.process).unwrap();
    parity::normalize_process_defaults(&machine, &mut process);
    let mut filaments = vec![profiles.filament(&selection.filaments[0]).unwrap(); 2];
    parity::normalize_filament_defaults(&mut filaments);
    for (filament, (colour, temperature)) in filaments
        .iter_mut()
        .zip([("#F2754E", "211"), ("#0000FF", "233")])
    {
        filament.insert(
            "filament_extruder_variant".into(),
            json!(["Direct Drive Standard"]),
        );
        filament.insert("filament_colour".into(), json!([colour]));
        filament.insert("nozzle_temperature".into(), json!([temperature]));
        filament.insert(
            "nozzle_temperature_initial_layer".into(),
            json!([temperature]),
        );
    }
    let mut overrides = parity::smoke_overrides();
    let dependencies = json!({
        "filament_map":requested, "filament_map_mode":"Manual",
        "nozzle_diameter":["0.4","0.4"], "use_relative_e_distances":"1",
        "retract_length_toolchange":["1","1","2","2"],
        "wall_filament":"1", "sparse_infill_filament":"2", "solid_infill_filament":"2",
        "sparse_infill_density":"40%", "layer_height":"0.2", "initial_layer_print_height":"0.2",
        "enable_support":"0", "raft_layers":"0", "enable_prime_tower":"0",
        "disable_m73":"1"
    });
    for (key, value) in dependencies.as_object().unwrap() {
        overrides.insert(key.clone(), value.clone());
    }
    overrides.insert(
        "machine_start_gcode".into(),
        machine["machine_start_gcode"].clone(),
    );
    overrides.insert(
        "change_filament_gcode".into(),
        machine["change_filament_gcode"].clone(),
    );
    let mut exported = runner
        .export_case(
            &CaseInputs {
                label,
                machine: &machine,
                process: &process,
                filaments: &filaments,
            },
            &overrides,
            &runner::repo_root().join("tests/parity/cube10.stl"),
        )
        .unwrap();
    let before = runner::application::settings(&exported.project).unwrap();
    // Native export can emit only one map/colour despite two filament payloads.
    // Bind coherent logical identity before either slicer consumes this project.
    let mut fields = dependencies.as_object().unwrap().clone();
    fields.insert("filament_colour".into(), json!(["#F2754E", "#0000FF"]));
    fields.insert("filament_self_index".into(), json!(["1", "2"]));
    fields.insert(
        "flush_volumes_matrix".into(),
        json!(["0", "140", "140", "0", "0", "140", "140", "0"]),
    );
    fields.insert("flush_multiplier".into(), json!(["1", "1"]));
    bind_settings(&mut exported, &fields);
    exported = runner
        .bind_filament_map_mode(exported, &json!("Manual"))
        .unwrap();
    let proofs = fields
        .iter()
        .map(|(key, value)| {
            (
                key.clone(),
                runner::application::verify_value(&exported.project, key, value).unwrap(),
            )
        })
        .collect::<Map<_, _>>();
    runner::filament_map::verify(&exported.project, &json!("Manual")).unwrap();
    let artifact_root = parity::artifacts::root_from_env().unwrap();
    parity::artifacts::overwrite(
        &artifact_root.join(format!("{}-application.json", label.replace('/', "-"))),
        &serde_json::to_vec_pretty(&json!({
            "source":"PrintConfig.cpp:2489; ToolOrdering.cpp:1275-1312; GCode.hpp:103-107; bbs_3mf.cpp:4453-4460,7970-7981",
            "exported_before_binding":before, "requested":fields, "application":proofs,
            "scope":"two 0.4mm nozzles, two PLA filaments, Manual mode, wall1/infill2",
            "deferred":["automatic grouping","dynamic mapping","arbitrary vector shapes"]
        }))
        .unwrap(),
    )
    .unwrap();
    exported
}

fn bind_settings(exported: &mut ExportedCase, fields: &Map<String, Value>) {
    let mut input = zip::ZipArchive::new(Cursor::new(&exported.project)).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..input.len() {
        let mut file = input.by_index(index).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        if file.name() == "Metadata/project_settings.config" {
            let mut settings: Value = serde_json::from_slice(&bytes).unwrap();
            settings.as_object_mut().unwrap().extend(fields.clone());
            bytes = serde_json::to_vec_pretty(&settings).unwrap();
        } else if file.name() == "Metadata/model_settings.config" {
            // bbs_3mf.cpp:4453-4460,7970-7981: plate maps override project maps.
            let xml = String::from_utf8(bytes).unwrap();
            let plate = xml
                .split_once("<plate>")
                .unwrap()
                .1
                .split_once("</plate>")
                .unwrap()
                .0;
            let old = plate
                .split_once("<metadata key=\"filament_maps\" value=\"")
                .map(|(_, tail)| tail.split_once('"').unwrap().0);
            let requested = fields["filament_map"]
                .as_array()
                .unwrap()
                .iter()
                .map(|value| value.as_str().unwrap())
                .collect::<Vec<_>>()
                .join(" ");
            let after = format!("<metadata key=\"filament_maps\" value=\"{requested}\"/>");
            bytes = match old {
                Some(old) => xml.replacen(
                    &format!("<metadata key=\"filament_maps\" value=\"{old}\"/>"),
                    &after,
                    1,
                ),
                None => xml.replacen("<plate>", &format!("<plate>\n    {after}"), 1),
            }
            .into_bytes();
        }
        output
            .start_file(
                file.name(),
                zip::write::SimpleFileOptions::DEFAULT.compression_method(file.compression()),
            )
            .unwrap();
        output.write_all(&bytes).unwrap();
    }
    exported.project = output.finish().unwrap().into_inner();
}
