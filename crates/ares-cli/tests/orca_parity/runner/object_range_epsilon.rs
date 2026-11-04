use std::{
    fs,
    io::{Cursor, Read, Write},
};

use ares_core::{ConfigOverrides, GenerationMetadata, SlicerContext};
use sha2::{Digest, Sha256};

use super::{ExportedCase, OrcaRunner};

const CONTROL: &[u8] = include_bytes!(
    "../../../../../tests/parity/validation/object-range-epsilon-complex-control.3mf"
);

#[tokio::test]
async fn complex_region_range_within_epsilon_matches_complete_native_output() {
    let Some(runner) = native_runner() else {
        return;
    };
    compare(&runner, "complex-control", CONTROL).await;
    compare(
        &runner,
        "complex-within-above",
        include_bytes!(
            "../../../../../tests/parity/validation/object-range-epsilon-complex-within.3mf"
        ),
    )
    .await;
}

#[tokio::test]
async fn complex_range_exact_below_and_outside_epsilon_preserve_complete_native_output() {
    let Some(runner) = native_runner() else {
        return;
    };
    // Actual Native zs[10] captured from the admitted release producer, not print Z.
    let z = 2.0999999046325684_f64;
    for (label, boundary) in [
        ("complex-exact", z),
        ("complex-within-below", z - 0.00005),
        ("complex-outside-below", z - 0.0002),
        ("complex-outside-above", z + 0.0002),
    ] {
        compare(&runner, label, &with_range(CONTROL, boundary)).await;
    }
}

#[tokio::test]
async fn complex_range_inclusive_epsilon_threshold_matches_complete_native_output() {
    let Some(runner) = native_runner() else {
        return;
    };
    let input = with_range(CONTROL, 2.0999999046325684_f64 + 1e-4);
    compare(&runner, "complex-inclusive-threshold", &input).await;
}

#[tokio::test]
async fn fast_region_range_exact_and_near_boundary_preserve_complete_native_output() {
    let Some(runner) = native_runner() else {
        return;
    };
    let input =
        include_bytes!("../../../../../tests/parity/validation/object-range-wall-loops.3mf");
    for (label, boundary) in [
        ("fast-exact", 2.0999999046325684_f64),
        ("fast-within-above", 2.10005_f64),
    ] {
        compare(&runner, label, &with_range(input, boundary)).await;
    }
}

fn native_runner() -> Option<OrcaRunner> {
    crate::native_environment::binary("ARES_ORCA_BIN")?;
    let runner = OrcaRunner::from_env().unwrap();
    if runner.is_none() {
        crate::native_environment::skipped(
            "Native runner unavailable for complex epsilon comparison",
        );
    }
    runner
}

fn with_range(input: &[u8], boundary: f64) -> Vec<u8> {
    let mut archive = zip::ZipArchive::new(Cursor::new(input)).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let name = "Metadata/layer_config_ranges.xml";
    for index in 0..archive.len() {
        let mut file = archive.by_index(index).unwrap();
        if file.name() == name {
            continue;
        }
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        output
            .start_file(file.name(), zip::write::SimpleFileOptions::DEFAULT)
            .unwrap();
        output.write_all(&bytes).unwrap();
    }
    output
        .start_file(name, zip::write::SimpleFileOptions::DEFAULT)
        .unwrap();
    // Shortest round-trippable double preserves the exact inclusive threshold operand.
    write!(output, "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<objects><object id=\"1\"><range min_z=\"{boundary}\" max_z=\"6\"><option opt_key=\"layer_height\">0.2</option><option opt_key=\"wall_loops\">3</option></range></object></objects>\n").unwrap();
    output.finish().unwrap().into_inner()
}

async fn compare(runner: &OrcaRunner, label: &str, input: &[u8]) {
    let mut builder = tempfile::Builder::new();
    builder.prefix("range-epsilon-");
    let directory = if std::env::var_os("ARES_PARITY_ARTIFACT_ROOT").is_some() {
        builder.tempdir_in(crate::artifacts::root_from_env().unwrap())
    } else {
        builder.tempdir()
    }
    .unwrap()
    .keep();
    let project_path = directory.join("input.3mf");
    fs::write(&project_path, input).unwrap();
    let output_dir = directory.join("native");
    fs::create_dir(&output_dir).unwrap();
    let reference = runner
        .rerun_reference(&ExportedCase {
            label: format!("region-range-epsilon/{label}"),
            project: input.to_vec(),
            project_path,
            output_dir,
        })
        .unwrap()
        .reference;
    fs::write(directory.join("native.gcode"), &reference).unwrap();
    let actual = crate::runner::application::slice_project_plate_validated(
        input,
        GenerationMetadata::default(),
        None,
        &ConfigOverrides::default(),
        SlicerContext::new(0),
    )
    .await;
    if let Ok(bytes) = &actual {
        fs::write(directory.join("ares.gcode"), bytes).unwrap();
    }
    fs::write(directory.join("result.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "label": label,
        "input_sha256": Sha256::digest(input).iter().map(|byte| format!("{byte:02x}")).collect::<String>(),
        "native_bytes": reference.len(),
        "ares": actual.as_ref().map(|bytes| bytes.len()).map_err(ToString::to_string),
    })).unwrap()).unwrap();
    let actual = actual.unwrap_or_else(|error| panic!("{label}: {error}; artifacts {directory:?}"));
    super::first_travel_acceleration_ordering::assert_first_travel_prefix(
        &directory, label, &reference, &actual,
    );
    crate::golden::compare_ordered_bytes_generator_only(&reference, &actual)
        .unwrap_or_else(|error| panic!("{label}: {error}; artifacts {directory:?}"));
}
