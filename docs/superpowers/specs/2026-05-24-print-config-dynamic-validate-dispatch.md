# M208 Spec: DynamicPrintConfig validate printer-technology dispatch

## Goal
Port OrcaSlicer's `DynamicPrintConfig::validate(bool under_cli)` printer-technology dispatch shell into Ares as a public read-only `SliceOptions` API that routes FFF configs to the already source-cited M207 aggregate validator and returns an empty map for SLA/non-FFF configs.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:8629-8647`: `DynamicPrintConfig::validate(bool under_cli)`, including absent `printer_technology` default to `ptFFF`, `ptFFF` dispatch to `Slic3r::validate(fpc, under_cli)`, and default non-FFF empty map return.
- `OrcaSlicer/src/libslic3r/PrintConfig.hpp:641`: `DynamicPrintConfig::validate(bool under_cli = false)` declaration.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:131-135`: `PrinterTechnology` enum string mapping for `FFF` / `SLA`.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:676-682`: `printer_technology` option definition, enum values, and default `ptFFF`.
- M207 milestone docs: `SliceOptions::validate_fff_options(under_cli)` is the already-ported FFF validation destination.

Related upstream behavior explicitly deferred:

- `FullPrintConfig fpc; fpc.apply(*this, true)` materialization and all `FullPrintConfig` typed field semantics.
- The internal C++ `ConfigOptionEnumGeneric` storage model and full enum deserialization machinery.
- Future SLA validation if Orca adds it; this milestone preserves current source behavior where non-FFF returns an empty map.
- Preset/model loading machinery, UI runtime behavior, slicing, geometry, extrusion planning, G-code writer behavior, filesystem behavior, network behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- Create `crates/ares-core/src/options/validation/dispatch.rs` with `SliceOptions::validate_print_config(&self, under_cli: bool) -> Result<BTreeMap<String, String>, SliceError>`.
- Modify `crates/ares-core/src/options/validation.rs` to register `mod dispatch;`.
- Add `crates/ares-core/src/options/tests/validation/dispatch.rs` and register it from `crates/ares-core/src/options/tests/validation/mod.rs`.
- `docs/roadmap.md` and `docs/milestones/m208-print-config-dynamic-validate-dispatch.md`: milestone sequencing docs.

## Functional requirements

1. Add public read-only API `SliceOptions::validate_print_config(under_cli: bool) -> Result<BTreeMap<String, String>, SliceError>`.
2. If `printer_technology` is absent, treat it as source default `FFF` and return `self.validate_fff_options(under_cli)`.
3. If `printer_technology` is string `"FFF"`, return `self.validate_fff_options(under_cli)`.
4. If `printer_technology` is string `"SLA"`, return an empty `BTreeMap`.
5. If `printer_technology` is any other string, return `SliceError::InvalidInput` at the Ares JSON boundary because this milestone does not implement C++ typed enum deserialization; this is a boundary validation equivalent to rejecting unsupported enum strings before dispatch.
6. If `printer_technology` exists but is not a string, return `SliceError::InvalidInput`.
7. Forward `under_cli` unchanged to `validate_fff_options` for absent/FFF configs.
8. Preserve existing `validate_fff_options` and all standalone M196-M206 validation APIs unchanged.
9. Do not add `FullPrintConfig` materialization, typed enum storage, SLA validation, generic validation for unported numeric keys/types, slicing, extrusion, G-code behavior, new crates, or dependencies.
10. Keep modified Rust files under 400 LOC.

## Acceptance checks

- Tests prove absent `printer_technology` dispatches to FFF validation by reporting an invalid FFF option.
- Tests prove explicit `"FFF"` dispatches to FFF validation by reporting an invalid FFF option.
- Tests prove explicit `"SLA"` returns an empty map even when FFF-only invalid options are present.
- Tests prove `under_cli` is forwarded to FFF validation: the same spiral-vase CLI-invalid config reports `wall_loops` when `under_cli = true` and suppresses it when `under_cli = false`.
- Tests prove unknown string and non-string `printer_technology` return `SliceError::InvalidInput`.
- Tests prove existing `validate_fff_options` remains callable and unchanged.
- Plan/spec explicitly account for deferred `FullPrintConfig` materialization, typed enum machinery, SLA validation, and unported generic numeric validation.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
