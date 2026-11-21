use super::{OrcaRunner, VendorProfiles};
use crate::runner::{CaseInputs, native_api_cli::NativeCase, repo_root};
use serde_json::{Map, json};

const CC: &str = "Elegoo Centauri Carbon 0.4 nozzle";
const CC2: &str = "Elegoo Centauri Carbon 2 0.4 nozzle";
const START: &str = "G90\nM83\n";
const PURGE: &str = "M6211 A1 L200 T0 Q210 R210 S210";

fn check(label: &str, printer: &str, start: Option<&str>, model: Option<&str>) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let repo = repo_root();
    let profiles =
        VendorProfiles::load(&repo.join("OrcaSlicer/resources/profiles"), "Elegoo").unwrap();
    let selection = crate::select_printer(&profiles, "Elegoo", printer).unwrap();
    let mut machine = profiles.machine(printer).unwrap();
    let process = profiles
        .process(if printer == CC {
            "0.28mm Extra Draft @Elegoo CC 0.4 nozzle"
        } else {
            &selection.process
        })
        .unwrap();
    if let Some(start) = start {
        machine.insert("machine_start_gcode".into(), json!(start));
    }
    if let Some(model) = model {
        machine.insert("printer_model".into(), json!(model));
    }
    let filament = profiles.filament(&selection.filaments[0]).unwrap();
    let exported = runner
        .export_case(
            &CaseInputs {
                label,
                machine: &machine,
                process: &process,
                filaments: &[filament],
            },
            &Map::new(),
            &repo.join("tests/parity/cube10.stl"),
        )
        .unwrap();
    let case = NativeCase::slice(&runner, &exported.project, &format!("{label}-"));
    let effective_model = machine["printer_model"].as_str().unwrap();
    assert!(
        case.native
            .lines()
            .any(|line| line.strip_prefix("; printer_model = ") == Some(effective_model)),
        "{:?}",
        case.root
    );
    if start.is_none_or(|start| start.contains("M6211")) {
        assert!(
            case.native.lines().any(|line| line.starts_with("M6211 ")),
            "{:?}",
            case.root
        );
    }
    case.compare(ares_core::GenerationMetadata::deterministic(
        2026, 11, 18, 0, 0, 0,
    ));
}

#[test]
fn centauri_carbon_start_purge() {
    check("m6211-cc", CC, Some(&format!("{START}{PURGE}\n")), None);
}

#[test]
fn centauri_carbon_without_command() {
    check("m6211-cc-control", CC, Some(START), None);
}

#[test]
fn centauri_carbon_2_stock_start_purge() {
    check("m6211-cc2-stock", CC2, None, None);
}

#[test]
fn centauri_2_explicit_feeds_cooldown_and_alias() {
    check(
        "m6211-cc2-explicit",
        CC2,
        Some("G90\nM83\nM6211 L180 T0 K50 M240 N450 P800 R270 S210\n"),
        Some("eLeGoO Centauri 2"),
    );
}

#[test]
fn other_elegoo_has_zero_delay_but_accounts_for_purge() {
    check(
        "m6211-other-elegoo",
        CC,
        Some(&format!("{START}{PURGE}\nm6211 L20 T0.4 S215\n")),
        Some("eLeGoO Processor Witness"),
    );
}

#[test]
fn non_elegoo_ignores_command() {
    check(
        "m6211-other-printer",
        CC,
        Some(&format!("{START}{PURGE}\n")),
        Some("Witness Elegoo Centauri Carbon"),
    );
}

#[test]
fn invalid_arguments_are_ignored() {
    check(
        "m6211-invalid",
        CC,
        Some("G90\nM83\nM6211 L0 T0\nM6211 L200 T-1\nM6211 L200 T0.5\nM6211 T0\nM6211 L200\n"),
        None,
    );
}
