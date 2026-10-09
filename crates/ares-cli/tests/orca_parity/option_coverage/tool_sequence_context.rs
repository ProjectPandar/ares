//! ParameterUtils.cpp:6-29 and bbs_3mf.cpp:4428-4435 admission fixture.
use super::*;
use crate::runner::{CaseInputs, ExportedCase};
use serde_json::json;
use std::io::{Cursor, Read, Write};

fn bind_plate(body: &str, fields: &[(&str, String)]) -> String {
    let mut body = body.to_owned();
    for (key, value) in fields {
        let start = format!("<metadata key=\"{key}\" value=\"");
        let metadata = format!("{start}{value}\"/>");
        if let Some((_, tail)) = body.split_once(&start) {
            let old = tail.split_once('"').unwrap().0;
            body = body.replacen(&format!("{start}{old}\"/>"), &metadata, 1);
        } else {
            body.push_str(&format!("\n    {metadata}"));
        }
    }
    body
}

// Failure inventory: export may shrink the palette or replace plate sequences;
// first-layer IDs are zero-based, range IDs one-based; comments alone do not
// prove usage. Never classify export/application/observer errors as production RED.
pub(super) fn prepare(
    runner: &OrcaRunner,
    label: &str,
    count: usize,
    sequence: &[usize],
) -> ExportedCase {
    let profiles = VendorProfiles::load(&profiles_root(), "Creality").unwrap();
    let selection =
        parity::select_printer(&profiles, "Creality", "Creality Ender-3 0.4 nozzle").unwrap();
    let mut machine = profiles.machine(&selection.printer).unwrap();
    machine.insert("single_extruder_multi_material".into(), json!("1"));
    machine.insert("use_relative_e_distances".into(), json!("1"));
    let start = format!(
        "{}\n; G4_SEQUENCE F{{initial_extruder}}\n",
        machine["machine_start_gcode"].as_str().unwrap()
    );
    let change = format!(
        "{}\n; G4_SEQUENCE F{{next_extruder}}\n",
        machine["change_filament_gcode"].as_str().unwrap()
    );
    let mut process = profiles.process(&selection.process).unwrap();
    parity::normalize_process_defaults(&machine, &mut process);
    let mut filaments = vec![profiles.filament(&selection.filaments[0]).unwrap(); 2];
    parity::normalize_filament_defaults(&mut filaments);
    for (filament, (colour, temperature)) in filaments
        .iter_mut()
        .zip([("#F2754E", "211"), ("#0000FF", "233")])
    {
        filament.insert("filament_colour".into(), json!([colour]));
        filament.insert("nozzle_temperature".into(), json!([temperature]));
        filament.insert(
            "nozzle_temperature_initial_layer".into(),
            json!([temperature]),
        );
    }
    let vector = sequence.iter().map(usize::to_string).collect::<Vec<_>>();
    let dependencies = json!({
        "machine_start_gcode":start, "change_filament_gcode":change,
        "other_layers_print_sequence_nums":count.to_string(),
        "other_layers_print_sequence":vector,
        "first_layer_print_sequence":["0","1"],
        "first_layer_sequence_choice":"Customize","other_layers_sequence_choice":"Customize",
        "print_sequence":"by layer", "filament_map_mode":"Manual", "filament_map":["1","1"],
        "single_extruder_multi_material":"1", "use_relative_e_distances":"1",
        "filament_is_support":["0","0"],
        "nozzle_diameter":["0.4"], "nozzle_volume_type":["Standard"],
        "outer_wall_filament_id":"1", "inner_wall_filament_id":"1",
        "sparse_infill_filament_id":"2", "internal_solid_filament_id":"2",
        "top_surface_filament_id":"2", "bottom_surface_filament_id":"2",
        "sparse_infill_density":"40%", "layer_height":"0.2", "initial_layer_print_height":"0.2",
        "enable_support":"0", "raft_layers":"0", "skirt_loops":"0", "brim_type":"no_brim",
        "enable_prime_tower":"0", "disable_m73":"1"
    });
    let mut overrides = parity::smoke_overrides();
    for key in [
        "wall_filament",
        "sparse_infill_filament",
        "solid_infill_filament",
    ] {
        overrides.remove(key);
    }
    overrides.extend(dependencies.as_object().unwrap().clone());
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
    let root = parity::artifacts::root_from_env().unwrap();
    let stem = label.replace('/', "-");
    parity::artifacts::overwrite(
        &root.join(format!("{stem}-exported.3mf")),
        &exported.project,
    )
    .unwrap();
    let before = runner::application::settings(&exported.project).unwrap();
    let mut fields = dependencies.as_object().unwrap().clone();
    fields.insert("filament_colour".into(), json!(["#F2754E", "#0000FF"]));
    fields.insert("filament_self_index".into(), json!(["1", "2"]));
    fields.insert("nozzle_temperature".into(), json!(["211", "233"]));
    fields.insert(
        "nozzle_temperature_initial_layer".into(),
        json!(["211", "233"]),
    );
    fields.insert(
        "flush_volumes_matrix".into(),
        json!(["0", "140", "140", "0"]),
    );
    fields.insert("flush_multiplier".into(), json!(["1"]));
    let plate = [
        ("first_layer_print_sequence", "0 1".to_owned()),
        (
            "other_layers_print_sequence",
            sequence
                .iter()
                .map(usize::to_string)
                .collect::<Vec<_>>()
                .join(" "),
        ),
        ("other_layers_print_sequence_nums", count.to_string()),
        ("filament_maps", "1 1".to_owned()),
        ("filament_map_mode", "Manual".to_owned()),
        ("print_sequence", "by layer".to_owned()),
    ];
    let mut input = zip::ZipArchive::new(Cursor::new(&exported.project)).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let mut unchanged = Vec::new();
    let mut plate_before = String::new();
    let mut plate_after = String::new();
    for index in 0..input.len() {
        let mut file = input.by_index(index).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        match file.name() {
            "Metadata/project_settings.config" => {
                let mut settings: Value = serde_json::from_slice(&bytes).unwrap();
                for key in [
                    "wall_filament",
                    "sparse_infill_filament",
                    "solid_infill_filament",
                ] {
                    settings.as_object_mut().unwrap().remove(key);
                }
                settings.as_object_mut().unwrap().extend(fields.clone());
                bytes = serde_json::to_vec_pretty(&settings).unwrap();
            }
            "Metadata/model_settings.config" => {
                let xml = String::from_utf8(bytes).unwrap();
                let (prefix, tail) = xml.split_once("<plate>").unwrap();
                let (body, suffix) = tail.split_once("</plate>").unwrap();
                plate_before = body.to_owned();
                let body = bind_plate(body, &plate);
                plate_after = body.clone();
                bytes = format!("{prefix}<plate>{body}</plate>{suffix}").into_bytes();
            }
            _ => {
                unchanged.push(file.name().to_owned());
            }
        }
        output
            .start_file(
                file.name(),
                zip::write::SimpleFileOptions::DEFAULT.compression_method(file.compression()),
            )
            .unwrap();
        output.write_all(&bytes).unwrap();
    }
    drop(input);
    let project = output.finish().unwrap().into_inner();
    let mut original = zip::ZipArchive::new(Cursor::new(&exported.project)).unwrap();
    let mut actual = zip::ZipArchive::new(Cursor::new(&project)).unwrap();
    for name in &unchanged {
        let mut before = Vec::new();
        let mut after = Vec::new();
        original
            .by_name(name)
            .unwrap()
            .read_to_end(&mut before)
            .unwrap();
        actual
            .by_name(name)
            .unwrap()
            .read_to_end(&mut after)
            .unwrap();
        assert_eq!(before, after, "nonowned ZIP member {name}");
    }
    drop(original);
    drop(actual);
    exported.project = project;
    exported = runner
        .bind_filament_map_mode(exported, &json!("Manual"))
        .unwrap();
    parity::artifacts::overwrite(&root.join(format!("{stem}-bound.3mf")), &exported.project)
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
    for (key, value) in &plate {
        assert!(plate_after.contains(&format!("<metadata key=\"{key}\" value=\"{value}\"/>")));
    }
    let loaded = ares_core::load_project(&exported.project).unwrap();
    let typed = &loaded.settings().project.print;
    assert_eq!(typed.other_layers_print_sequence_nums.0, count as i32);
    assert_eq!(
        typed
            .other_layers_print_sequence
            .0
            .iter()
            .map(|id| id.0)
            .collect::<Vec<_>>(),
        sequence.iter().map(|&id| id as i32).collect::<Vec<_>>()
    );
    parity::artifacts::overwrite(&root.join(format!("{stem}-application.json")), &serde_json::to_vec_pretty(&json!({
        "source":"ParameterUtils.cpp:6-29; ToolOrdering.cpp:1320-1362; bbs_3mf.cpp:4428-4435; OrcaSlicer.cpp:5979",
        "requested":fields,"exported":before,"application":proofs,
        "plate_before":plate_before,"plate_after":plate_after,"unchanged_members":unchanged,
        "ares_typed_count":typed.other_layers_print_sequence_nums.0,
        "ares_typed_vector":typed.other_layers_print_sequence.0.iter().map(|id| id.0).collect::<Vec<_>>(),
        "deferred":["by-object","count>=3","malformed vectors","other G4 consumers"]
    })).unwrap()).unwrap();
    exported
}
