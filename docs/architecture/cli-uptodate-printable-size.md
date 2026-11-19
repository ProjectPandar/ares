# Native CLI up-to-date printable-size admission

## Source boundary

This slice ports the retained-machine admission in the pinned OrcaSlicer v2.5.0 source, `OrcaSlicer.cpp:1899-1909` and `OrcaSlicer.cpp:2563-2600`. The 3MF loader records the embedded `printable_area` bounding-box width and depth and `printable_height`, truncating each value to `int`. After a retained machine file is loaded, Native computes the same three values from that file and rejects the refresh when any embedded dimension is larger than the loaded machine dimension. The diagnostic is `Printer Settings: the printable size {w, d, h} exceeds the default size.`, the CLI result code is `CLI_MODIFIED_PARAMS_TO_PRINTER` (`-23`), and the result report is written before exit. Native also reads `model_id` from the external printer-model record, but Ares has no equivalent plate-data output or judgment seam; this slice therefore adds no new I/O for that metadata.

## Ares destination

The I/O-free retained-refresh loader in `crates/ares-core/src/profiles/project_refresh` owns the comparison between the raw project settings and the loaded machine preset. The CLI failure mapper owns the `CLI_MODIFIED_PARAMS_TO_PRINTER` (`-23`) family result and report serialization. The public CLI parity module is registered from `crates/ares-cli/tests/orca_parity/case_support.rs`; the root `orca_parity.rs` remains unchanged.

## Included

- Capture the embedded dimensions using the upstream bbox and truncation semantics.
- Compare them with the flattened retained machine file after that file has been loaded, including explicit files and resource fallback.
- Reject only when all loaded dimensions are positive and one embedded dimension exceeds its corresponding loaded dimension, matching the cited Native branch.
- Preserve the exact diagnostic, exit code, and `result.json` error string.
- Preserve the existing boundary for Native's external `printer_model_id` metadata read without adding new Ares I/O, because Ares has no equivalent plate-data output or judgment seam.
- Add a synthetic 3MF witness with an oversized retained bed and an exactly equal boundary witness; compare Native and Ares whole-file output where slicing succeeds and compare CLI status/result reports for rejection.

## Deferred

This slice does not port Native's unrelated machine envelope checks, printer selection, GUI behavior, resource packaging, or post-refresh bed translation. It does not modify the accepted retained-preset argv routing or the root parity registry.

## Acceptance

The synthetic oversized witness rejects in both Native and Ares with no G-code, matching CLI result code and error string. The exact-equality witness does not reject and its complete G-code matches between Native and Ares after the suite's existing generator-banner normalization. The affected cases pass in both official and seeded Native modes, the refresh paired check has no prior-pass regressions, and formatting, all-target Clippy, and WASM checks pass.
