use super::*;
use crate::runner::CaseInputs;
use std::io::{Cursor, Read};

#[test]
fn single_volume_offset_matches_complete_prism_output() {
    if !crate::runner::reference_enabled("ARES_PARITY_VOLUME_OFFSETS_REFERENCE") {
        return;
    }
    if std::env::var("ARES_PARITY_SEEDED_REFERENCE").as_deref() != Ok("1") {
        crate::native_environment::skipped("seeded-only reference");
        return;
    }
    let runner = OrcaRunner::from_env().unwrap().unwrap();
    assert_eq!(std::env::var("ORCA_REPLAY_SEED").unwrap(), "0");
    let profiles = VendorProfiles::load(&profiles_root(), "Creality").unwrap();
    let selection =
        parity::select_printer(&profiles, "Creality", "Creality Ender-3 0.4 nozzle").unwrap();
    let machine = profiles.machine(&selection.printer).unwrap();
    let mut process = profiles.process(&selection.process).unwrap();
    parity::normalize_process_defaults(&machine, &mut process);
    for (key, value) in [
        ("wall_loops", "0"),
        ("detect_narrow_internal_solid_infill", "1"),
        ("fuzzy_skin", "none"),
        ("top_shell_layers", "7"),
        ("bottom_shell_layers", "5"),
        ("sparse_infill_density", "15%"),
    ] {
        process.insert(key.into(), Value::String(value.into()));
    }
    let mut filaments = selection
        .filaments
        .iter()
        .map(|name| profiles.filament(name).unwrap())
        .collect::<Vec<_>>();
    parity::normalize_filament_defaults(&mut filaments);
    let label = "single-volume-offset/rotated-short-branch";
    let exported = runner
        .export_case(
            &CaseInputs {
                label,
                machine: &machine,
                process: &process,
                filaments: &filaments,
            },
            &parity::smoke_overrides(),
            &runner::repo_root().join("tests/parity/arachne/short-branch.stl"),
        )
        .unwrap();
    let settings = runner::application::settings(&exported.project).unwrap();
    assert_eq!(settings["wall_loops"], "0");
    assert_eq!(settings["detect_narrow_internal_solid_infill"], "1");
    let mut archive = zip::ZipArchive::new(Cursor::new(&exported.project)).unwrap();
    let mut model = String::new();
    archive
        .by_name("3D/3dmodel.model")
        .unwrap()
        .read_to_string(&mut model)
        .unwrap();
    let matrix = |element: &str| {
        model
            .split(element)
            .nth(1)
            .unwrap()
            .split("transform=\"")
            .nth(1)
            .unwrap()
            .split('"')
            .next()
            .unwrap()
            .split_whitespace()
            .map(|value| value.parse::<f64>().unwrap())
            .collect::<Vec<_>>()
    };
    let component = matrix("<component ");
    let instance = matrix("<item ");
    assert_eq!(component.len(), 12);
    assert_eq!(instance.len(), 12);
    assert!(component[9..].iter().any(|offset| *offset != 0.0));
    let project = ares_core::load_project(&exported.project).unwrap();
    let object = &project.objects()[0];
    // bbs_3mf.cpp:7844–7852 writes three rows per column; 2337–2358
    // changes only instance offsets, so all linear coefficients must survive.
    let transform = object.instances()[0].transform().to_column_major();
    for (column, expected) in instance[..9].as_chunks::<3>().0.iter().enumerate() {
        assert_eq!(&transform[column * 4..column * 4 + 3], expected);
    }
    assert_eq!(object.volumes().len(), 1);
    let reference = runner.slice_case(&exported).unwrap();
    let outcome = crate::artifacts::compare(
        &crate::artifacts::root_from_env().unwrap(),
        label,
        Ok(&exported.project),
        Ok(&reference.reference),
        serde_json::json!({
            "kind": "single_volume_offset_normalization",
            "binary": std::env::var("ORCA_REPLAY_BIN").unwrap(),
            "seed": 0,
            "cpu": std::env::var("ORCA_REPLAY_CPU").unwrap(),
        }),
    )
    .unwrap();
    assert_eq!(outcome.status, "PASS", "{}", outcome.detail);
    assert_eq!(
        object.volumes()[0]
            .transform()
            .transform_point(ares_core::Point3d::new(0.0, 0.0, 0.0)),
        ares_core::Point3d::new(0.0, 0.0, 0.0)
    );
}
