use crate::runner::{OrcaRunner, native_api_cli::NativeCase, repo_root};

#[test]
fn h2d_initial_nozzle_id_matches_native() {
    run(
        (
            "h2d-canonical-startup",
            "f6a40929f428cedd3a407007af94ccf4154b054c54c031e194cef160067c5343",
        ),
        &[
            "; NOZZLE_IDS context=start initial=1 current=1\n".to_owned(),
            "; NOZZLE_DIAMETER_ARRAY context=start n0=0.4 n1=0.6\n".to_owned(),
        ],
        &[
            ("filament_map", "2"),
            ("filament_nozzle_map", "1"),
            ("nozzle_diameter", "0.39,0.61"),
        ],
    );
}

#[test]
fn h2c_nozzle_ids_match_native_in_all_contexts() {
    let mut markers = [
        ("start", 0),
        ("filament_start", 0),
        ("filament_start", 2),
        ("filament_start", 1),
        ("filament_end", 0),
        ("filament_end", 2),
        ("filament_end", 1),
        ("end", 2),
    ]
    .map(|(context, current)| {
        format!("; NOZZLE_IDS context={context} initial=0 current={current}\n")
    })
    .to_vec();
    let nozzles = [0, 2, 1];
    for (old, next) in [(0, 1), (1, 2), (2, 0), (0, 2), (2, 1), (1, 0)] {
        markers.push(format!(
            "; NOZZLE_IDS context=change initial=0 current={} next={} previous={old} next_filament={next}\n",
            nozzles[old], nozzles[next],
        ));
    }
    run(
        (
            "h2c-inventory-contexts",
            "b1e2af10f2f878ec3f80b5f0af30f44df2e4723e20d490736538003941785e8e",
        ),
        &markers,
        &[
            ("extruder_max_nozzle_count", "1,6"),
            ("filament_map", "1,2,2"),
            ("filament_nozzle_map", "0,2,1"),
        ],
    );
}

fn run(input: (&str, &str), markers: &[String], fields: &[(&str, &str)]) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let (name, expected_sha) = input;
    let project = std::fs::read(repo_root().join(format!(
        "tests/parity/nozzle-id-diameters/scalars/{name}.3mf"
    )))
    .unwrap();
    assert_eq!(crate::runner::native_api_cli::sha(&project), expected_sha);
    let case = NativeCase::slice(&runner, &project, &format!("nozzle-ids-{name}-"));
    for marker in markers {
        assert!(case.native.contains(marker));
    }
    for (field, value) in fields {
        assert!(case.native.contains(&format!("; {field} = {value}\n")));
    }
    let (api, cli) = case.compare(ares_core::GenerationMetadata::deterministic(
        2026, 8, 27, 0, 0, 0,
    ));
    assert_eq!(api, cli, "raw API/CLI equality");
}
