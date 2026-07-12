# M207: PrintConfig validate FFF aggregate API

## Goal
Port the source-order aggregation behavior of OrcaSlicer's FFF `validate(const FullPrintConfig&, bool)` into Ares so UI/CLI callers can run the already-ported M196-M206 validation slices through one API.

## Rewrite gate
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`. Its source boundary is `OrcaSlicer/src/libslic3r/PrintConfig.cpp:10088-10308`, covering the full FFF validation function from first error checks through final generic numeric range validation and return, plus the previously ported M196-M206 option-definition/function contexts cited by their milestone docs. It adds only `SliceOptions::validate_fff_options(under_cli)` as a source-order aggregator over existing validation slice APIs. It does not port `DynamicPrintConfig::validate` printer-technology dispatch, `FullPrintConfig::apply`, SLA behavior, preset/model loading, UI runtime, slicing, extrusion, G-code, new crate, or dependency behavior.

## Exit checklist
- `SliceOptions::validate_fff_options(under_cli: bool)` returns one `BTreeMap<String, String>` containing errors from the M196-M206 validation slices in Orca source order.
- Duplicate keys preserve the first source-order message, matching C++ `std::map::emplace` / generic range `find` behavior.
- The existing M203 spiral-vase CLI slice is called only when aggregate `under_cli` is true, matching the source `cfg.spiral_mode && under_cli` guard without changing the standalone API.
- Defaults return an empty map.
- Non-CLI spiral-vase aggregate suppresses CLI-only spiral errors while retaining all other validation errors.
- Width-limit errors suppress later generic range errors for the same key in the aggregate.
- Existing standalone validation APIs remain unchanged and usable.
- Full `DynamicPrintConfig::validate` dispatch, non-FFF/SLA validation behavior, `FullPrintConfig` materialization, and all unported generic numeric keys/types remain deferred.
- `cargo fmt --check`, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check -p ares-core --target wasm32-unknown-unknown`, and `git diff --check` pass.
