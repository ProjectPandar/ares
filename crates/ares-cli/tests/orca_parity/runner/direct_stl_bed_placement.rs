use super::{OrcaRunner, profile_entry, repo_root, stages};
use std::{fs, path::Path};

#[test]
fn direct_stl_above_bed_matches_native() {
    compare(1.0);
}

#[test]
fn direct_stl_below_bed_matches_native() {
    compare(-1.0);
}

fn compare(bottom_z: f32) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let repo = repo_root();
    let profiles = crate::presets::VendorProfiles::load(
        &repo.join("OrcaSlicer/resources/profiles"),
        "Flashforge",
    )
    .unwrap();
    let [machine, process, filament] = [
        (
            "machine.json",
            profiles.machine("Flashforge Adventurer A5 0.4 nozzle"),
        ),
        (
            "process.json",
            profiles.process("0.20mm Standard @FF Adventurer A5 0.4 nozzle"),
        ),
        (
            "filament.json",
            profiles.filament("Flashforge HS PLA @FF Adventurer A5"),
        ),
    ]
    .map(|(filename, values)| {
        let path = runner.work.join(filename);
        profile_entry::write_json(&path, &serde_json::Value::Object(values.unwrap()));
        path
    });
    let mut stl = fs::read(repo.join("tests/parity/cube10.stl")).unwrap();
    // The fixture spans -5..5 in Z. Keep XY, normals and triangle order unchanged.
    for facet in stl[84..].as_chunks_mut::<50>().0 {
        for vertex in facet[12..48].as_chunks_mut::<12>().0 {
            let original = f32::from_le_bytes(vertex[8..12].try_into().unwrap());
            vertex[8..12].copy_from_slice(&(original + 5.0 + bottom_z).to_le_bytes());
        }
    }
    let input = runner.work.join("cube10.stl");
    fs::write(&input, stl).unwrap();
    let native = runner
        .slice_stl_with_settings([&machine, &process, &filament], &input)
        .unwrap();
    assert!(layers(&native) == "50", "Native must retain the full cube");
    let output = runner.work.join("cli.gcode");
    stages::run_live(
        Path::new(env!("CARGO_BIN_EXE_ares")),
        [
            "slice".into(),
            "--load-settings".into(),
            format!("{};{}", machine.display(), process.display()),
            "--load-filaments".into(),
            filament.display().to_string(),
            "-o".into(),
            output.display().to_string(),
            input.display().to_string(),
        ],
        stages::Stage::Application,
        &runner.work,
    )
    .unwrap_or_else(|error| panic!("Z={bottom_z}: {error}"));
    let cli = fs::read(output).unwrap();
    let difference = crate::golden::compare_ordered_bytes_generator_only(&native, &cli);
    assert!(
        difference.is_ok(),
        "Z={bottom_z}: native {} layers, CLI {} layers: {difference:?}; {:?}",
        layers(&native),
        layers(&cli),
        runner.work,
    );
}

fn layers(gcode: &[u8]) -> &str {
    std::str::from_utf8(gcode)
        .unwrap()
        .lines()
        .find_map(|line| line.strip_prefix("; total layer number: "))
        .unwrap()
}
