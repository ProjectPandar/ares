use crate::runner::{OrcaRunner, native_api_cli::NativeCase, repo_root};

#[test]
fn h2d_canonical_nozzle_diameters_match_native() {
    run(
        (
            "h2d-canonical-startup",
            "65787acb8b0a9459c77f8ef5009aa6a402471cd5cfbaa146d37c2318ec5b2093",
        ),
        (&["start"], "n0=0.4 n1=0.6"),
        &[
            ("nozzle_diameter", "0.39,0.61"),
            ("bridge_line_width", "0.38"),
        ],
    );
}

#[test]
fn h2c_unused_nozzle_diameters_in_all_contexts_match_native() {
    run(
        (
            "h2c-inventory-contexts",
            "753ffed53ac8bf188bb5df5d8140ac2cbc2ed3023538dd5918d6a35d7458d3d5",
        ),
        (
            &["start", "change", "end", "filament_start", "filament_end"],
            "n0=0.4 n1=0.4 n6=0.4",
        ),
        &[
            ("extruder_max_nozzle_count", "1,6"),
            ("filament_map", "1,2,2"),
            ("filament_nozzle_map", "0,2,1"),
            ("filament_volume_map", "0,0,0"),
        ],
    );
}

fn run(input: (&str, &str), marker: (&[&str], &str), fields: &[(&str, &str)]) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let (name, expected_sha) = input;
    let project =
        std::fs::read(repo_root().join(format!("tests/parity/nozzle-id-diameters/{name}.3mf")))
            .unwrap();
    assert_eq!(crate::runner::native_api_cli::sha(&project), expected_sha);
    let case = NativeCase::slice(&runner, &project, &format!("{name}-"));
    for context in marker.0 {
        assert!(case.native.contains(&format!(
            "; NOZZLE_DIAMETER_ARRAY context={context} {}\n",
            marker.1
        )));
    }
    for (field, value) in fields {
        assert!(case.native.contains(&format!("; {field} = {value}\n")));
    }
    let (api, cli) = case.compare(ares_core::GenerationMetadata::deterministic(
        2026, 8, 27, 0, 0, 0,
    ));
    assert_eq!(api, cli, "raw API/CLI equality");
}
