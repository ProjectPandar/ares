//! PrintConfig.cpp:1832; bbs_3mf.cpp:2632,7722; Config.hpp:1118-1158.

use super::{OptionOutcome, domains};
use crate::{CaseInputs, OrcaRunner, VendorProfiles, runner};
use serde_json::{Value, json};
use std::{
    fs,
    io::{Cursor, Read, Write},
};

pub(super) const KEY: &str = "print_compatible_printers";
const MACHINE: &str = "Creality Ender-3 0.4 nozzle";

#[test]
fn orca_parity_embedded_compatible_printer_strings_bounded_classification() {
    let Some(runner) = OrcaRunner::from_env().unwrap_or_else(|error| panic!("{error}")) else {
        return;
    };
    witnesses(&runner);
    let plans = domains::load(&runner::repo_root());
    let plan = plans.iter().find(|plan| plan.key == KEY).unwrap();
    let outcome = OptionOutcome::executed(plan, plan.cases.len(), Vec::new(), 0, None);
    assert_eq!(
        outcome.status, "PASS",
        "Native fidelity and full comparisons passed; bounded classification remains {}: {}",
        outcome.status, outcome.detail
    );
    assert_eq!(plan.cases.len(), 7);
    assert_eq!(plan.cases[0].label, "missing-default");
    assert_eq!(plan.cases[0].value, Some(json!([])));
    assert_eq!(plan.cases[1].label, "explicit-empty");
    assert_eq!(plan.cases[1].value, Some(json!([])));
    let mut unproven = plan.clone();
    unproven.provably_legal = false;
    assert_eq!(
        OptionOutcome::executed(&unproven, unproven.cases.len(), Vec::new(), 0, None).status,
        "INCOMPLETE"
    );
}

pub(super) fn witnesses(runner: &OrcaRunner) {
    let root = tempfile::Builder::new()
        .prefix("embedded-pcp-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let original = original(runner);
    let settings = runner::application::settings(&original).unwrap();
    assert_eq!(settings["printer_settings_id"], json!(MACHINE));
    assert_eq!(settings["enable_prime_tower"], json!("0"));
    let mut compared = 0;
    for (label, requested) in [
        ("retained-baseline", Some(settings[KEY].clone())),
        ("missing-default", None),
        ("explicit-empty", Some(json!([]))),
        ("empty-element", Some(json!([MACHINE, ""]))),
        ("one-label", Some(json!([MACHINE, "Embedded Printer A"]))),
        (
            "ordered-duplicates",
            Some(json!([
                MACHINE,
                "Embedded Printer B",
                "Embedded Printer A",
                "Embedded Printer B"
            ])),
        ),
        (
            "escaped-elements",
            Some(json!([
                MACHINE,
                "semi;colon",
                "quote\"slash\\",
                "line\nnext",
                "tab\tvalue"
            ])),
        ),
        ("unicode-label", Some(json!([MACHINE, "打印机 Ω"]))),
    ] {
        let directory = root.join(label);
        fs::create_dir(&directory).unwrap();
        let project = rewrite(&original, requested.as_ref());
        let expected = requested.clone().unwrap_or_else(|| json!([]));
        let input = directory.join("input.3mf");
        fs::write(&input, &project).unwrap();
        let read = ares_core::load_project(&project).unwrap();
        assert_eq!(
            json!(read.settings().project.preset.print_compatible_printers.0),
            expected
        );
        let serialized = serde_json::to_value(&read.settings().project.preset).unwrap();
        assert_eq!(serialized[KEY], expected);
        let native_dir = directory.join("native");
        fs::create_dir(&native_dir).unwrap();
        runner::stages::run(
            runner.bin(),
            [
                "--slice".to_owned(),
                "0".to_owned(),
                "--outputdir".to_owned(),
                native_dir.to_str().unwrap().to_owned(),
                input.to_str().unwrap().to_owned(),
            ],
            runner::stages::Stage::Slice,
            &directory,
        )
        .unwrap();
        let native = fs::read(native_dir.join("plate_1.gcode")).unwrap();
        let text = std::str::from_utf8(&native).unwrap();
        let footer = text
            .lines()
            .filter_map(|line| {
                line.strip_prefix("; print_compatible_printers = ")
                    .map(str::to_owned)
            })
            .collect::<Vec<_>>();
        assert_eq!(footer.len(), 1, "keyed Native footer missing: {label}");
        // The footer's ConfigOptionStrings::serialize token is compared in the
        // complete stream below. Config.cpp:72-120 does not quote semicolons in
        // otherwise-simple elements, so it is not a reversible vector witness.
        // Native save_to_json/vserialize is the independent typed-vector proof.
        let roundtrip = directory.join("roundtrip.3mf");
        runner::stages::run(
            runner.bin(),
            [
                "--export-3mf".to_owned(),
                roundtrip.to_str().unwrap().to_owned(),
                input.to_str().unwrap().to_owned(),
            ],
            runner::stages::Stage::Export,
            &directory,
        )
        .unwrap();
        let native_json = runner::application::settings(&fs::read(&roundtrip).unwrap()).unwrap();
        assert_eq!(
            native_json[KEY], expected,
            "Native writer fidelity: {label}"
        );
        let outcome = crate::compare_case(&runner::ParityCase {
            label: format!("option/embedded-pcp/{label}"),
            project,
            reference: native,
        });
        fs::write(directory.join("result.json"), serde_json::to_vec_pretty(&json!({
            "requested":requested,"expected":expected,"native_footer":footer,
            "native_roundtrip":native_json[KEY],"ares_typed":serialized[KEY],
            "status":outcome.status,"detail":outcome.detail,
            "scope":"embedded seven-case ordered coStrings fidelity; not compatibility decisions",
        })).unwrap()).unwrap();
        assert_eq!(outcome.status, "PASS", "{label}: {}", outcome.detail);
        compared += 1;
    }
    eprintln!("Embedded compatible printer artifacts: {}", root.display());
    assert_eq!(
        compared, 8,
        "retained baseline plus all seven revised witnesses"
    );
}

pub(super) fn cases() -> Vec<domains::OptionCase> {
    [
        ("missing-default", json!([])),
        ("explicit-empty", json!([])),
        ("empty-element", json!([MACHINE, ""])),
        ("one-label", json!([MACHINE, "Embedded Printer A"])),
        (
            "ordered-duplicates",
            json!([
                MACHINE,
                "Embedded Printer B",
                "Embedded Printer A",
                "Embedded Printer B"
            ]),
        ),
        (
            "escaped-elements",
            json!([
                MACHINE,
                "semi;colon",
                "quote\"slash\\",
                "line\nnext",
                "tab\tvalue"
            ]),
        ),
        ("unicode-label", json!([MACHINE, "打印机 Ω"])),
    ]
    .into_iter()
    .map(|(label, value)| domains::OptionCase {
        label: label.into(),
        value: Some(value),
    })
    .collect()
}

pub(super) fn metadata() -> Value {
    json!({
        "scope":"seven finite embedded typed coStrings fidelity witnesses; not global string/vector domain or compatibility decisions",
        "source":"PrintConfig.cpp:1832-1834; bbs_3mf.cpp:2632-2653,7722-7727; Config.hpp:1118-1158; Config.cpp:1464-1504",
        "printer_settings_id":MACHINE,
        "case_value_meaning":"expected typed vector; missing-default means remove raw key, explicit-empty means set raw key to []",
        "raw_input_actions":{"missing-default":"remove_key", "explicit-empty":"set_empty_array", "other_five":"set_exact_array"},
        "additional_control":"retained original baseline, executed separately; not used as a noninjected complete plan",
        "footer":"actual serialized token compared in complete output; not assumed reversible for semicolons",
        "native_vector_authority":"actual --export-3mf Metadata/project_settings.config via ConfigOptionStrings::vserialize",
        "deferred":["worker5 nonmatching-machine -17 rejection/compatibility filtering/load-settings", "lone empty singleton without machine ID", "arbitrary string/array domain", "scalar/nested/malformed JSON", "other coStrings keys"],
    })
}

fn original(runner: &OrcaRunner) -> Vec<u8> {
    // Reproduce the already admitted adaptive-pa/no-cooling source project,
    // then change only the embedded key; no profile overlays during replay.
    let profiles = VendorProfiles::load(
        &runner::repo_root().join("OrcaSlicer/resources/profiles"),
        "Creality",
    )
    .unwrap();
    let selection = crate::select_printer(&profiles, "Creality", MACHINE).unwrap();
    let machine = profiles.machine(&selection.printer).unwrap();
    let mut process = profiles.process(&selection.process).unwrap();
    crate::normalize_process_defaults(&machine, &mut process);
    for (key, value) in [
        ("enable_prime_tower", "0"),
        ("gcode_comments", "0"),
        ("enable_overhang_speed", "1"),
    ] {
        process.insert(key.into(), json!(value));
    }
    let mut filaments = selection
        .filaments
        .iter()
        .map(|name| profiles.filament(name).unwrap())
        .collect::<Vec<_>>();
    crate::normalize_filament_defaults(&mut filaments);
    for filament in &mut filaments {
        for (key, value) in [
            ("enable_pressure_advance", "1"),
            ("pressure_advance", "0.035"),
            ("adaptive_pressure_advance", "1"),
            (
                "adaptive_pressure_advance_model",
                "0.015,1,100\n0.045,5,100\n0.09,20,100\n0.02,1,1000\n0.08,5,1000\n0.12,20,1000\n0.035,1,5000\n0.1,5,5000\n0.2,20,5000",
            ),
            ("adaptive_pressure_advance_overhangs", "0"),
            ("adaptive_pressure_advance_bridges", "0.06"),
            ("slow_down_for_layer_cooling", "0"),
        ] {
            filament.insert(key.into(), json!([value]));
        }
    }
    runner
        .export_case(
            &CaseInputs {
                label: "option/embedded-pcp/source",
                machine: &machine,
                process: &process,
                filaments: &filaments,
            },
            &crate::smoke_overrides(),
            &runner::repo_root().join("tests/parity/cube10.stl"),
        )
        .unwrap()
        .project
}

fn rewrite(input: &[u8], requested: Option<&Value>) -> Vec<u8> {
    let mut reader = zip::ZipArchive::new(Cursor::new(input)).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..reader.len() {
        let mut member = reader.by_index(index).unwrap();
        let name = member.name().to_owned();
        let mut bytes = Vec::new();
        member.read_to_end(&mut bytes).unwrap();
        if name == "Metadata/project_settings.config" {
            let mut settings: serde_json::Map<String, Value> =
                serde_json::from_slice(&bytes).unwrap();
            if let Some(value) = requested {
                settings.insert(KEY.into(), value.clone());
            } else {
                settings.remove(KEY);
            }
            bytes = serde_json::to_vec(&settings).unwrap();
        }
        output
            .start_file(name, zip::write::SimpleFileOptions::default())
            .unwrap();
        output.write_all(&bytes).unwrap();
    }
    let rewritten = output.finish().unwrap().into_inner();
    // All other member contents remain exactly the original, not just selected metadata.
    let mut check = zip::ZipArchive::new(Cursor::new(&rewritten)).unwrap();
    assert_eq!(reader.len(), check.len());
    for index in 0..reader.len() {
        let mut old = reader.by_index(index).unwrap();
        if old.name() == "Metadata/project_settings.config" {
            continue;
        }
        let mut new = check.by_name(old.name()).unwrap();
        let (mut a, mut b) = (Vec::new(), Vec::new());
        old.read_to_end(&mut a).unwrap();
        new.read_to_end(&mut b).unwrap();
        assert_eq!(a, b);
    }
    rewritten
}
