use ares_wasm::slice_project_plate_processed_with_catalog_bytes;

#[tokio::test]
async fn direct_bbl_project_keeps_values_when_no_catalog_can_resolve_them() {
    let input = include_bytes!("../../../tests/parity/orca_cli_ender3.3mf");
    let mut absent =
        slice_project_plate_processed_with_catalog_bytes(input.to_vec(), Some(1), None, None)
            .await
            .expect("omitting the catalog leaves all system filaments unresolved");
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
    let gcode = output.take_gcode();
    assert_eq!(
        absent.take_gcode(),
        gcode,
        "complete G-code after all lookups miss"
    );
    std::fs::write(directory.join("ares.gcode"), gcode).unwrap();
    eprintln!("artifact: {}", directory.display());
}
