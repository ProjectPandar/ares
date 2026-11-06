use std::{fs, path::Path, process::Output};

use serde_json::json;

pub(super) fn verify(expected: &str, native: &[u8], api: &[u8], cli: &Output, directory: &Path) {
    let cli_bytes = fs::read(directory.join("cli/output.gcode")).unwrap();
    let native_lines = lines(native);
    let api_lines = lines(api);
    let cli_lines = lines(&cli_bytes);
    let api_matches = api_lines == native_lines;
    let cli_matches = cli_lines == native_lines;
    fs::write(
        directory.join("dirty-column-comparison.json"),
        serde_json::to_vec_pretty(&json!({
            "upstream":"345efec64c7b95a05ce5cf96df0dde66a158835c",
            "expected":expected,
            "native_lines":native_lines,"api_lines":api_lines,"cli_lines":cli_lines,
            "native_matches_literal":native_lines == [expected],
            "api_matches_native":api_matches,"cli_matches_native":cli_matches,
            "cli_exit":cli.status.code(),
            "cli_stderr":String::from_utf8_lossy(&cli.stderr),
            "api_cli_raw_equal":api == cli_bytes,
            "api_sha256":super::super::super::sha256(api),
            "cli_sha256":super::super::super::sha256(&cli_bytes),
            "whole_output_parity_claimed":false,
        }))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        native_lines,
        [expected],
        "Native dirty-column premise; {directory:?}"
    );
    assert!(cli.status.success(), "CLI generation; {directory:?}");
    assert_eq!(api, cli_bytes, "raw API/CLI output; {directory:?}");
    assert!(
        api_matches && cli_matches,
        "CLI override dirty-column mismatch: Native={native_lines:?}; API={api_lines:?}; CLI={cli_lines:?}; {directory:?}"
    );
}

fn lines(bytes: &[u8]) -> Vec<&str> {
    std::str::from_utf8(bytes)
        .unwrap()
        .lines()
        .filter(|line| line.starts_with("; different_settings_to_system = "))
        .collect()
}
