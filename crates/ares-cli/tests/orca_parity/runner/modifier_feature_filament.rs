//! Public target-equality REDs for PrintObject's modifier feature filament IDs.
use super::{OrcaRunner, cache_identity, profile_entry, stages};
use serde_json::{Value, json};
use std::{fs, path::Path};

#[test]
fn modifier_feature_filament_sparse_infill_matches_native() {
    run("sparse_infill");
}
#[test]
fn modifier_feature_filament_internal_solid_matches_native() {
    run("internal_solid");
}
#[test]
fn modifier_feature_filament_top_surface_matches_native() {
    run("top_surface");
}
#[test]
fn modifier_feature_filament_bottom_surface_matches_native() {
    run("bottom_surface");
}
#[test]
fn modifier_feature_filament_outer_wall_matches_native() {
    run("outer_wall");
}
#[test]
fn modifier_feature_filament_inner_wall_matches_native() {
    run("inner_wall");
}

fn run(role: &str) {
    if !super::reference_enabled("ARES_PARITY_PROFILE_ENTRY_REFERENCE") {
        return;
    }
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let fixtures = super::repo_root().join("tests/parity/modifier-feature-filament");
    let manifest: Value =
        serde_json::from_slice(&fs::read(fixtures.join("manifest.json")).unwrap()).unwrap();
    let feature = manifest["roles"][role]["feature"].as_str().unwrap();
    let control = fs::read(fixtures.join("control.3mf")).unwrap();
    assert_eq!(
        cache_identity::raw_digest(&control),
        manifest["control_sha256"]
    );
    let native_control = native_case(&runner, &control, "control");
    assert_eq!(local_role_t1(&native_control, feature), 0);
    public_case(&runner.work, role, "control", &control, &native_control);

    let positive = fs::read(fixtures.join(format!("{role}.3mf"))).unwrap();
    assert_eq!(
        cache_identity::raw_digest(&positive),
        manifest["roles"][role]["sha256"]
    );
    let native_positive = native_case(&runner, &positive, "positive");
    let local_segments = local_role_t1(&native_positive, feature);
    profile_entry::write_json(
        &runner.work.join("native-role-qualification.json"),
        &json!({"role":role,"feature":feature,"local_T1_segments":local_segments,
            "control_native_sha256":cache_identity::raw_digest(&native_control),
            "positive_native_sha256":cache_identity::raw_digest(&native_positive)}),
    );
    assert!(
        local_segments > 0,
        "Native local T1 role qualification: {role}"
    );
    public_case(&runner.work, role, "positive", &positive, &native_positive);
}

fn native_case(runner: &OrcaRunner, input: &[u8], label: &str) -> Vec<u8> {
    let directory = runner.work.join(label);
    fs::create_dir(&directory).unwrap();
    let input_path = directory.join("input.3mf");
    fs::write(&input_path, input).unwrap();
    stages::run(
        runner.bin(),
        [
            "--slice".to_owned(),
            "0".to_owned(),
            "--outputdir".to_owned(),
            directory.to_str().unwrap().to_owned(),
            input_path.to_str().unwrap().to_owned(),
        ],
        stages::Stage::Slice,
        &directory,
    )
    .unwrap();
    let output = fs::read(directory.join("plate_1.gcode")).unwrap();
    assert!(
        std::str::from_utf8(&output)
            .unwrap()
            .lines()
            .any(|line| line == "; CONFIG_BLOCK_END")
    );
    output
}

fn public_case(work: &Path, role: &str, label: &str, input: &[u8], native: &[u8]) {
    let directory = work.join(format!("public-{label}"));
    fs::create_dir(&directory).unwrap();
    let input_path = directory.join("input.3mf");
    fs::write(&input_path, input).unwrap();
    let cli = crate::runner::application::cli_command()
        .args(["slice", "-o"])
        .arg(directory.join("cli.gcode"))
        .arg(&input_path)
        .output()
        .unwrap();
    fs::write(directory.join("cli.stdout"), &cli.stdout).unwrap();
    fs::write(directory.join("cli.stderr"), &cli.stderr).unwrap();
    let api = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(crate::runner::application::slice_project_plate_validated(
            input,
            ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            None,
            &ares_core::ConfigOverrides::default(),
            ares_core::SlicerContext::new(0),
        ));
    profile_entry::write_json(
        &directory.join("public-attempt.json"),
        &json!({"role":role,"label":label,"api_error":api.as_ref().err().map(|error|format!("{error:?}")),
            "api_ok":api.is_ok(),"cli_exit":cli.status.code(),
            "comparison":"NOT EVALUATED before target admission assertions"}),
    );
    assert!(
        cli.status.success(),
        "target CLI slicing failed for {role}/{label}: {}",
        String::from_utf8_lossy(&cli.stderr)
    );
    let api =
        api.unwrap_or_else(|error| panic!("target API slicing failed for {role}/{label}: {error}"));
    fs::write(directory.join("api.gcode"), &api).unwrap();
    let cli_bytes = fs::read(directory.join("cli.gcode")).unwrap();
    assert_eq!(api, cli_bytes, "raw API/CLI bytes for {role}/{label}");
    crate::golden::compare_ordered_bytes_generator_only(native, &api)
        .unwrap_or_else(|difference| panic!("strict Native/API for {role}/{label}: {difference}"));
    crate::golden::compare_ordered_bytes_generator_only(native, &cli_bytes)
        .unwrap_or_else(|difference| panic!("strict Native/CLI for {role}/{label}: {difference}"));
    profile_entry::write_json(
        &directory.join("public-result.json"),
        &json!({"role":role,"label":label,"status":"TARGET_EQUAL"}),
    );
}

fn local_role_t1(gcode: &[u8], feature: &str) -> usize {
    let mut tool = "";
    let mut current_feature = "";
    let mut relative_e = false;
    let mut xy = [None, None];
    let mut count = 0;
    for line in std::str::from_utf8(gcode).unwrap().lines() {
        if let Some(value) = line.strip_prefix("; FEATURE: ") {
            current_feature = value;
        } else if let Some(value) = line.strip_prefix(";TYPE:") {
            current_feature = value;
        }
        let instruction = line.split(';').next().unwrap();
        let mut words = instruction.split_ascii_whitespace();
        let Some(command) = words.next() else {
            continue;
        };
        match command {
            "M83" => relative_e = true,
            "M82" => relative_e = false,
            _ if command.starts_with('T') => tool = command,
            _ => {}
        }
        if !matches!(command, "G0" | "G1" | "G2" | "G3") {
            continue;
        }
        let mut next = xy;
        let mut e = 0.0;
        let mut moved_xy = false;
        for word in words {
            if let Some(value) = word.strip_prefix('X') {
                next[0] = Some(value.parse::<f64>().unwrap());
                moved_xy = true;
            } else if let Some(value) = word.strip_prefix('Y') {
                next[1] = Some(value.parse::<f64>().unwrap());
                moved_xy = true;
            } else if let Some(value) = word.strip_prefix('E') {
                e = value.parse::<f64>().unwrap();
            }
        }
        // Frozen modifier world bbox after component and root item transforms.
        let inside = [xy, next].into_iter().all(|[x, y]| {
            x.is_some_and(|x| (132.999..=138.001).contains(&x))
                && y.is_some_and(|y| (117.999..=138.001).contains(&y))
        });
        if tool == "T1" && current_feature == feature && relative_e && e > 0.0 && moved_xy && inside
        {
            count += 1;
        }
        xy = next;
    }
    count
}
