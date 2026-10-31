const INPUT: &[u8] =
    include_bytes!("../../../tests/parity/validation/layer-reset-kobra-plus-relative.3mf");
const ERROR: &str = "Relative extruder addressing requires resetting the extruder position at each layer to prevent loss of floating point accuracy. Add \"G92 E0\" to layer_gcode.";

#[tokio::test]
async fn wasm_stl_boundary_preserves_print_validation_error() {
    let input = include_bytes!("../../../tests/parity/cube10.stl").to_vec();
    let result = ares_wasm::slice_stl_bytes(
        input,
        r#"{"use_relative_e_distances":true,"gcode_flavor":"marlin"}"#,
    )
    .await;
    let root = tempfile_root("o1-wasm-stl.json", &format!("{result:?}"));
    assert_eq!(result.unwrap_err(), ERROR, "{}", root.display());
}

fn tempfile_root(name: &str, text: &str) -> std::path::PathBuf {
    let root = std::env::temp_dir().join("ares-o1-wasm-entry");
    std::fs::create_dir_all(&root).unwrap();
    let path = root.join(name);
    std::fs::write(&path, text).unwrap();
    path
}

#[test]
fn wasm_project_core_boundary_preserves_print_validation_error() {
    let result =
        ares_core::validate_project_plate(INPUT, None, &ares_core::ConfigOverrides::default());
    let artifact = tempfile_root("o1-wasm-project-core.json", &format!("{result:?}"));
    assert!(
        matches!(result, Err(ares_core::SliceError::Validate { message: ref text, .. }) if text == ERROR),
        "{}",
        artifact.display()
    );
}
