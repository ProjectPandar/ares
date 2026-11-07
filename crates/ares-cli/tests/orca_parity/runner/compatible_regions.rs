use super::{CaseInputs, ExportedCase, OrcaRunner, application, repo_root, volume_fixture};
use crate::{self as parity, presets::VendorProfiles};
use serde_json::json;
use std::io::{Cursor, Read};

#[path = "compatible_regions/fixture.rs"]
mod fixture;

#[path = "compatible_regions/inherited_filament.rs"]
mod inherited_filament;

#[path = "compatible_regions/top_pattern.rs"]
mod top_pattern;

#[path = "compatible_regions/solid_pattern.rs"]
mod solid_pattern;

#[path = "compatible_regions/modifier_travel.rs"]
mod modifier_travel;

#[path = "compatible_regions/same_xy.rs"]
mod same_xy;

#[path = "compatible_regions/end_layer.rs"]
mod end_layer;

#[path = "compatible_regions/infill_first.rs"]
mod infill_first;

#[path = "compatible_regions/tool_templates.rs"]
mod tool_templates;

#[test]
fn compatible_regions_with_later_higher_density_preserve_complete_output() {
    if !crate::runner::reference_enabled("ARES_PARITY_REGION_PERIMETERS_REFERENCE") {
        return;
    }
    let runner = OrcaRunner::from_env().unwrap().unwrap();
    let profiles = VendorProfiles::load(
        &repo_root().join("OrcaSlicer/resources/profiles"),
        "Creality",
    )
    .unwrap();
    let selection =
        parity::select_printer(&profiles, "Creality", "Creality Ender-3 0.4 nozzle").unwrap();
    let machine = profiles.machine(&selection.printer).unwrap();
    let mut process = profiles.process(&selection.process).unwrap();
    parity::normalize_process_defaults(&machine, &mut process);
    let settings = [
        ("enable_support", "0"),
        ("sparse_infill_density", "0%"),
        ("sparse_infill_pattern", "rectilinear"),
    ];
    for (key, value) in settings {
        process.insert(key.into(), json!(value));
    }
    let mut filaments = selection
        .filaments
        .iter()
        .map(|name| profiles.filament(name).unwrap())
        .collect::<Vec<_>>();
    parity::normalize_filament_defaults(&mut filaments);
    let exported = runner
        .export_case(
            &CaseInputs {
                label: "compatible-regions/later-higher-density",
                machine: &machine,
                process: &process,
                filaments: &filaments,
            },
            &parity::smoke_overrides(),
            &repo_root().join("tests/parity/overhang.stl"),
        )
        .unwrap();
    let mut baseline = runner.slice_case(&exported).unwrap();
    baseline.label.push_str("/without-modifier");
    let outcome = parity::compare_case(&baseline);
    assert_eq!(outcome.status, "PASS", "{}", outcome.detail);
    assert!(!has_sparse_extrusion(
        std::str::from_utf8(&baseline.reference).unwrap()
    ));

    let project = volume_fixture::add_volumes(
        &exported.project,
        "modifier_part",
        &[(0.6, -6.0)],
        &[("sparse_infill_density", "50%")],
    );
    for (key, value) in settings {
        application::verify_value(&project, key, &json!(value)).unwrap();
    }
    let loaded = ares_core::load_project(&project).unwrap();
    assert_eq!(loaded.objects().len(), 1);
    assert_eq!(loaded.objects()[0].volumes().len(), 2);
    assert_eq!(
        loaded.objects()[0]
            .volumes()
            .iter()
            .map(|volume| (volume.id(), volume.volume_type()))
            .collect::<Vec<_>>(),
        [
            (1, ares_core::ProjectVolumeType::ModelPart),
            (3, ares_core::ProjectVolumeType::ParameterModifier),
        ]
    );
    let modifier = &loaded.objects()[0].volumes()[1];
    assert_eq!(
        modifier
            .transform()
            .transform_point(ares_core::Point3d::new(1.0, 0.0, 0.0)),
        ares_core::Point3d::new(-5.4, 0.0, 7.5)
    );
    let mut archive = zip::ZipArchive::new(Cursor::new(&project)).unwrap();
    let mut xml = String::new();
    archive
        .by_name("Metadata/model_settings.config")
        .unwrap()
        .read_to_string(&mut xml)
        .unwrap();
    let (_, modifier_xml) = xml
        .split_once("<part id=\"3\" subtype=\"modifier_part\">")
        .unwrap();
    let (modifier_xml, _) = modifier_xml.split_once("</part>").unwrap();
    let metadata = regex::Regex::new(r#"<metadata key="([^"]+)" value="([^"]*)"/>"#).unwrap();
    let mut values = metadata
        .captures_iter(modifier_xml)
        .map(|capture| (capture[1].to_owned(), capture[2].to_owned()))
        .collect::<Vec<_>>();
    values.sort();
    assert_eq!(
        values,
        [
            (
                "matrix".into(),
                "0.6 0 0 -6 0 1 0 0 0 0 1 7.5 0 0 0 1".into()
            ),
            ("name".into(), "overhang-annotation-3".into()),
            ("sparse_infill_density".into(), "50%".into()),
        ]
    );
    let project_path = runner.work.join("compatible-regions.3mf");
    let output_dir = runner.work.join("compatible-regions-native");
    std::fs::write(&project_path, &project).unwrap();
    std::fs::create_dir_all(&output_dir).unwrap();
    let sliced = runner
        .slice_case(&ExportedCase {
            label: exported.label,
            project,
            project_path,
            output_dir,
        })
        .unwrap();
    let outcome = parity::compare_case(&sliced);
    assert!(has_sparse_extrusion(
        std::str::from_utf8(&sliced.reference).unwrap()
    ));
    assert_eq!(outcome.status, "PASS", "{}", outcome.detail);
}

fn has_sparse_extrusion(reference: &str) -> bool {
    let mut sparse = false;
    let mut relative_e = false;
    reference.lines().any(|line| {
        let instruction = line.split(';').next().unwrap();
        let words = instruction.split_ascii_whitespace();
        match words.clone().next() {
            Some("M83") => relative_e = true,
            Some("M82") => relative_e = false,
            _ => {}
        }
        if let Some(role) = line.strip_prefix(";TYPE:") {
            sparse = role == "Sparse infill";
        }
        relative_e
            && sparse
            && instruction.starts_with("G1 ")
            && words.clone().any(|word| word.starts_with(['X', 'Y']))
            && words.clone().any(|word| {
                word.strip_prefix('E')
                    .is_some_and(|value| value.parse::<f64>().unwrap() > 0.0)
            })
    })
}
