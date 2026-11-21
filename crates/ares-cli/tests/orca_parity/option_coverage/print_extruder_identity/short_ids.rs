use super::{OrcaRunner, author_settings, export_cube, fs, json};
use crate::runner::{application::slice_project, native_api_cli::NativeCase};

async fn compare(label: &str, narrowed: bool, speeds: [&str; 2]) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let base = export_cube(&runner, "Bambu Lab H2D 0.4 nozzle", "short-process-ids");
    let standard = json!(["Direct Drive Standard", "Direct Drive Standard"]);
    let mut updates = vec![
        ("print_extruder_id", json!(["7"])),
        ("print_extruder_variant", standard.clone()),
        ("outer_wall_speed", json!(["500", "200"])),
        ("inner_wall_speed", json!(["300", "600"])),
    ];
    if narrowed {
        updates.push(("extruder_variant_list", standard));
    }
    let project = author_settings(&base, &updates);
    let native = NativeCase::slice(&runner, &project, label);
    let ares = slice_project(
        &project,
        ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
        ares_core::SlicerContext::new(0),
    )
    .await
    .unwrap();
    fs::write(native.root.join("ares.api.gcode"), &ares).unwrap();
    let ares = std::str::from_utf8(&ares).unwrap();
    let fields = [
        ("print_extruder_id", "7,7"),
        (
            "print_extruder_variant",
            "\"Direct Drive Standard\";\"Direct Drive Standard\"",
        ),
        ("outer_wall_speed", speeds[0]),
        ("inner_wall_speed", speeds[1]),
    ];
    for (key, expected) in fields {
        let line = format!("; {key} = ");
        let native_value = config_value(&native.native, &line);
        assert_eq!(native_value, expected, "Native {key}: {:?}", native.root);
        assert_eq!(
            config_value(ares, &line),
            native_value,
            "Ares {key}: {:?}",
            native.root,
        );
    }
}

fn config_value<'a>(gcode: &'a str, prefix: &str) -> &'a str {
    gcode
        .split_once("; CONFIG_BLOCK_START\n")
        .unwrap()
        .1
        .lines()
        .find_map(|line| line.strip_prefix(prefix))
        .unwrap()
}

#[tokio::test]
async fn narrowed_groups_preserve_two_process_columns() {
    compare("short-process-ids-narrowed-", true, ["500,200", "300,600"]).await;
}

#[tokio::test]
async fn stock_groups_keep_column_zero_for_both_heads() {
    compare("short-process-ids-stock-", false, ["500,500", "300,300"]).await;
}
