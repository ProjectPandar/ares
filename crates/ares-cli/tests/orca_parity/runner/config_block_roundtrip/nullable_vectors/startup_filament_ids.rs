use crate::runner::{OrcaRunner, native_api_cli::NativeCase, repo_root};

#[test]
fn startup_filament_ids_match_native_for_first_logical_filament() {
    compare("first", 0);
}

#[test]
fn startup_filament_ids_match_native_for_second_logical_filament() {
    compare("second", 1);
}

fn compare(filament: &str, expected: u32) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let path = repo_root().join(format!(
        "tests/parity/filament-group-flush-mechanisms/x2d_pla_auto_map_{filament}_filament_head1_only.3mf"
    ));
    let base = std::fs::read(path).unwrap();
    let script = "; W4_STARTUP_IDS initial={initial_filament_id} current={current_filament_id} control={initial_extruder}\n";
    let project = super::input::settings_variant(
        &base,
        &[("machine_start_gcode".to_owned(), script.into())]
            .into_iter()
            .collect(),
    );
    let case = NativeCase::slice(
        &runner,
        &project,
        &format!("startup-filament-ids-{filament}-"),
    );
    let marker =
        format!("; W4_STARTUP_IDS initial={expected} current={expected} control={expected}");
    assert_eq!(
        case.native.lines().filter(|line| *line == marker).count(),
        1
    );
    let (api, cli) = case.compare(ares_core::GenerationMetadata::deterministic(
        2026, 8, 27, 0, 0, 0,
    ));
    assert_eq!(api, cli, "raw API/CLI equality");
}
