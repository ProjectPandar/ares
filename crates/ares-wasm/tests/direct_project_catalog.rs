use ares_wasm::slice_project_plate_processed_with_catalog_bytes;

#[tokio::test]
async fn direct_bbl_project_requires_explicit_catalog_and_slices_with_supplied_bundle() {
    let input = include_bytes!("../../../tests/parity/orca_cli_ender3.3mf");
    let absent =
        slice_project_plate_processed_with_catalog_bytes(input.to_vec(), Some(1), None, None).await;
    assert!(absent.is_err());
    assert!(absent.err().unwrap().contains("system preset bundle"));
    let output = slice_project_plate_processed_with_catalog_bytes(
        input.to_vec(),
        Some(1),
        None,
        Some(r#"{"vendors":[],"imports":[]}"#),
    )
    .await
    .expect("unresolved system preset retains the embedded slot");
    let directory =
        std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("direct-project-catalog");
    std::fs::create_dir_all(&directory).unwrap();
    let mut output = output;
    std::fs::write(directory.join("ares.gcode"), output.take_gcode()).unwrap();
    eprintln!("artifact: {}", directory.display());
}
