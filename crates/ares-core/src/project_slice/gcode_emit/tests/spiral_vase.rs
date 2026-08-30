#[tokio::test]
async fn active_project_spiral_vase_ramps_z_across_body_extrusions() {
    const SETTINGS: &str = "Metadata/project_settings.config";
    let mut archive = crate::project_slice::tests::support::KsrArchive::new();
    let mut settings: serde_json::Value =
        serde_json::from_str(&archive.entry_text(SETTINGS)).unwrap();
    settings["spiral_mode"] = serde_json::json!("1");
    settings["bottom_shell_layers"] = serde_json::json!("1");
    settings["bottom_shell_thickness"] = serde_json::json!("0");
    settings["spiral_starting_flow_ratio"] = serde_json::json!("1");
    archive.insert_text(SETTINGS, &serde_json::to_string(&settings).unwrap());

    let output = crate::slice_project(
        archive.bytes(),
        crate::project_slice::tests::support::metadata(),
    )
    .await
    .unwrap();
    let lines = std::str::from_utf8(&output)
        .unwrap()
        .lines()
        .collect::<Vec<_>>();
    let body_start = lines
        .iter()
        .enumerate()
        .filter(|(_, line)| **line == "; CHANGE_LAYER")
        .nth(1)
        .unwrap()
        .0;
    let body_end = lines[body_start + 1..]
        .iter()
        .position(|line| *line == "; CHANGE_LAYER")
        .map_or(lines.len(), |offset| body_start + 1 + offset);
    let z_values = lines[body_start..body_end]
        .iter()
        .filter(|line| {
            line.starts_with("G1 ")
                && line.split_whitespace().any(|word| word.starts_with('X'))
                && line.split_whitespace().any(|word| word.starts_with('Y'))
                && line.split_whitespace().any(|word| word.starts_with('E'))
        })
        .filter_map(|line| {
            line.split_whitespace()
                .find_map(|word| word.strip_prefix('Z')?.parse::<f64>().ok())
        })
        .collect::<Vec<_>>();

    assert!(
        z_values.len() > 2,
        "body lines: {:?}",
        &lines[body_start..body_end]
    );
    assert!(z_values.windows(2).any(|pair| pair[0] != pair[1]));
    assert!(z_values.first() < z_values.last());
}
