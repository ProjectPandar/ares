#![cfg(target_os = "linux")]

use std::fs;

use ares_core::{PresetCatalogHandle, SliceError};
use serde_json::{Value, json};

use super::{OrcaRunner, cache_identity, profile_entry, repo_root, stages, write_preset};

#[derive(Clone, Copy, Debug, PartialEq)]
enum Conclusion {
    Compatible,
    NotCompatible,
    Missing,
}

fn compare(vendor: &str, printer: &str, expected: Conclusion) {
    let Some(_) = crate::native_environment::binary("ARES_ORCA_BIN") else {
        return;
    };
    let runner = OrcaRunner::from_env().unwrap().unwrap();
    let producer = cache_identity::identify(&runner.bin).unwrap();
    let root = repo_root().join("OrcaSlicer/resources/profiles");
    let profiles = crate::presets::VendorProfiles::load(&root, vendor).unwrap();
    let machine = profiles.machine(printer).unwrap();
    let process = machine["default_print_profile"].as_str().unwrap();
    let filament = machine["default_filament_profile"][0].as_str().unwrap();
    let missing = !profiles.process_exists(process);
    let work = tempfile::Builder::new()
        .prefix("vendor-profile-compatibility-")
        .tempdir_in(&runner.work)
        .unwrap()
        .keep();
    let machine_file = work.join("machine.json");
    let process_file = work.join(format!("{process}.json"));
    let filament_file = work.join("filament.json");
    write_preset(&machine_file, "machine", &machine).unwrap();
    write_preset(
        &filament_file,
        "filament",
        &profiles.filament(filament).unwrap(),
    )
    .unwrap();
    if !missing {
        write_preset(
            &process_file,
            "process",
            &profiles.process(process).unwrap(),
        )
        .unwrap();
    }
    let native_work = work.join("native");
    fs::create_dir(&native_work).unwrap();
    let admission = stages::run(
        &runner.bin,
        [
            "--load-settings".to_owned(),
            format!("{};{}", machine_file.display(), process_file.display()),
            "--load-filaments".to_owned(),
            filament_file.to_string_lossy().into_owned(),
            "--arrange".to_owned(),
            "1".to_owned(),
            "--export-3mf".to_owned(),
            work.join("input.3mf").to_string_lossy().into_owned(),
            repo_root()
                .join("tests/parity/cube10.stl")
                .to_string_lossy()
                .into_owned(),
        ],
        stages::Stage::Export,
        &native_work,
    );
    let native_result: Value =
        serde_json::from_slice(&fs::read(native_work.join("result.json")).unwrap()).unwrap();
    let native_code = native_result["return_code"].as_i64().unwrap();
    let native_evidence = admission
        .as_ref()
        .err()
        .and_then(|failure| failure.evidence.clone());
    let failure = admission.as_ref().err();
    let (conclusion, captured) = match failure {
        None => (Conclusion::Compatible, None),
        Some(failure) => {
            assert_eq!(
                failure.kind,
                stages::FailureKind::Process,
                "Native admission: {failure}"
            );
            let captured = failure.captured.as_ref().unwrap();
            let conclusion = match native_code {
                -17 => {
                    let native_log = fs::read_to_string(native_work.join("00000.log")).unwrap();
                    assert_eq!(captured.code, Some(239), "Native admission");
                    assert!(
                        native_log.contains("2652: process not compatible with printer."),
                        "Native admission: {native_log}"
                    );
                    assert!(
                        !missing,
                        "Native admission: expected an existing process identity"
                    );
                    Conclusion::NotCompatible
                }
                -3 => {
                    assert_eq!(captured.code, Some(253), "Native admission");
                    assert!(missing, "Native admission: target unexpectedly exists");
                    assert!(
                        String::from_utf8_lossy(&captured.stderr).contains(&format!(
                            "can not find setting file: {}",
                            process_file.display()
                        )),
                        "Native admission"
                    );
                    Conclusion::Missing
                }
                other => {
                    panic!("Native admission: different conclusion {other}, artifacts {work:?}")
                }
            };
            (conclusion, Some(captured))
        }
    };
    assert_eq!(conclusion, expected, "Native admission; artifacts {work:?}");
    assert_eq!(
        work.join("input.3mf").exists(),
        expected == Conclusion::Compatible,
        "Native admission: exported project {work:?}"
    );
    let native_compatible = match conclusion {
        Conclusion::Compatible => Some(true),
        Conclusion::NotCompatible => Some(false),
        Conclusion::Missing => None,
    };

    let vendors = [
        profile_entry::vendor(&root, "OrcaFilamentLibrary"),
        profile_entry::vendor(&root, vendor),
    ];
    profile_entry::write_json(&work.join("catalog.json"), &json!({"vendors": vendors}));
    let mut catalog = PresetCatalogHandle::new();
    for envelope in &vendors {
        catalog
            .add_vendor(&serde_json::to_vec(envelope).unwrap())
            .unwrap();
    }
    let actual = catalog.filter_compatible(printer, Some(process));
    let (ares_compatible, ares_error, expected_error) = match actual {
        Ok(filtered) => {
            let filtered = serde_json::to_value(filtered).unwrap();
            profile_entry::write_json(&work.join("ares-filtered.json"), &filtered);
            let compatible = filtered["processes"]
                .as_array()
                .unwrap()
                .iter()
                .any(|name| name.as_str() == Some(process));
            (Some(compatible), None, None)
        }
        Err(error) => {
            let expected = native_compatible
                .is_none()
                .then(|| format!("process preset '{process}' was not found"));
            let message = match error {
                SliceError::InvalidInput(message) => message,
                other => panic!("Core error type differs: {other:?}; artifacts {work:?}"),
            };
            (None, Some(message), expected)
        }
    };
    profile_entry::write_json(
        &work.join("comparison.json"),
        &json!({
            "vendor": vendor, "printer": printer, "default_process": process, "default_filament": filament,
            "native_code": native_code, "native_exit": captured.and_then(|captured| captured.code), "native_compatible": native_compatible,
            "ares_compatible": ares_compatible, "ares_missing_error": ares_error,
            "native_evidence": native_evidence, "producer": producer.receipt,
            "machine_sha256": cache_identity::raw_digest(&fs::read(&machine_file).unwrap()),
            "process_sha256": if missing { None } else { Some(cache_identity::raw_digest(&fs::read(&process_file).unwrap())) },
            "filament_sha256": cache_identity::raw_digest(&fs::read(&filament_file).unwrap()),
            "catalog_sha256": cache_identity::raw_digest(&fs::read(work.join("catalog.json")).unwrap()),
            "comparison_boundary": "public profiles catalog/compatibility, not CLI channels or slicing",
        }),
    );
    assert_eq!(
        ares_compatible, native_compatible,
        "Core compatibility differs; artifacts {work:?}"
    );
    if native_compatible.is_none() {
        assert!(
            ares_error.is_some(),
            "Core missing-name error absent; artifacts {work:?}"
        );
        assert_eq!(
            ares_error, expected_error,
            "Core missing-name error differs; artifacts {work:?}"
        );
    } else {
        assert!(
            ares_error.is_none(),
            "Core error differs; artifacts {work:?}"
        );
    }
}

macro_rules! case {
    ($test:ident, $vendor:literal, $printer:literal, $expected:expr) => {
        #[test]
        fn $test() {
            compare($vendor, $printer, $expected);
        }
    };
}

case!(
    k1_se_08_default_compatibility_matches_runtime_native,
    "Creality",
    "Creality K1 SE 0.8 nozzle",
    Conclusion::Compatible
);
case!(
    ratrig_300_08_default_compatibility_matches_runtime_native,
    "Ratrig",
    "RatRig V-Core 4 300 0.8 nozzle",
    Conclusion::Compatible
);
case!(
    ratrig_400_08_default_compatibility_matches_runtime_native,
    "Ratrig",
    "RatRig V-Core 4 400 0.8 nozzle",
    Conclusion::Compatible
);
case!(
    ratrig_500_08_default_compatibility_matches_runtime_native,
    "Ratrig",
    "RatRig V-Core 4 500 0.8 nozzle",
    Conclusion::Compatible
);
case!(
    tiq2_025_default_compatibility_matches_runtime_native,
    "iQ",
    "iQ TiQ2 0.25 Nozzle",
    Conclusion::Compatible
);
case!(
    tiq2_06_default_compatibility_matches_runtime_native,
    "iQ",
    "iQ TiQ2 0.6 Nozzle",
    Conclusion::Compatible
);
case!(
    tiq2_08_default_compatibility_matches_runtime_native,
    "iQ",
    "iQ TiQ2 0.8 Nozzle",
    Conclusion::Compatible
);
case!(
    tiq8_025_default_compatibility_matches_runtime_native,
    "iQ",
    "iQ TiQ8 0.25 Nozzle",
    Conclusion::Compatible
);
case!(
    tiq8_06_default_compatibility_matches_runtime_native,
    "iQ",
    "iQ TiQ8 0.6 Nozzle",
    Conclusion::Compatible
);
case!(
    tiq8_08_default_compatibility_matches_runtime_native,
    "iQ",
    "iQ TiQ8 0.8 Nozzle",
    Conclusion::Compatible
);
