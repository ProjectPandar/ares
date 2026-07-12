# M200 Spec: PrintConfig validate skirt height and bridge flow ratios

## Goal
Port OrcaSlicer's `skirt_height` and bridge-flow validation block from `Slic3r::validate(const FullPrintConfig&, bool)` into Ares as `SliceOptions::validate_skirt_and_bridge_flow_options()`, returning validation messages for this source slice without adding full validation dispatch or later checks.

## Upstream rewrite boundary
This milestone is governed by `docs/architecture/ard-0020-future-milestone-rewrite-gate.md`.

Exact upstream source boundary:

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:10172-10185`: `skirt_height`, `bridge_flow`, and `internal_bridge_flow` validation/error insertion.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:1266-1284` and `PrintConfig.hpp:1083-1084`: `bridge_flow` and `internal_bridge_flow` option-definition/default context.
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:5559-5565` and `PrintConfig.hpp:1553`: `skirt_height` option-definition/default context.

Important upstream behavior to preserve:

- The `internal_bridge_flow` error insertion in `PrintConfig.cpp:10183-10185` is guarded by `if (cfg.bridge_flow <= 0)`, not by `cfg.internal_bridge_flow <= 0`. M200 must preserve that source behavior instead of silently correcting it.

Related upstream behavior explicitly deferred:

- `PrintConfig.cpp:10187+` extruder-clearance, filament-flow, spiral-vase, and later validation checks.
- `PrintConfig.cpp:8629-8647` full `DynamicPrintConfig::validate` dispatch and `FullPrintConfig` materialization.
- Preset/model loading machinery, UI runtime behavior, slicing, geometry, extrusion planning, G-code writer behavior, filesystem behavior, network behavior, and any Ares-owned pipeline changes.

Rust destination boundary:

- `crates/ares-core/src/options/validation.rs`: add `SliceOptions::validate_skirt_and_bridge_flow_options(&self) -> Result<BTreeMap<String, String>, SliceError>` plus private helper reuse if needed.
- `crates/ares-core/src/options/tests/validation/`: add source-behavior tests in a focused module.
- `docs/roadmap.md` and `docs/milestones/m200-print-config-validate-skirt-and-bridge-flow.md`: milestone sequencing docs.

## Functional requirements

1. Add public read-only API `SliceOptions::validate_skirt_and_bridge_flow_options()` returning `Result<BTreeMap<String, String>, SliceError>`.
2. Missing `skirt_height`, `bridge_flow`, and `internal_bridge_flow` use source-cited registry defaults and return no errors.
3. If `skirt_height < 0`, report key `skirt_height` with message `invalid value {skirt_height}`.
4. If `bridge_flow <= 0`, report key `bridge_flow` with message `invalid value {bridge_flow:.6}`, matching C++ `std::to_string(double)` formatting for this source slice.
5. If `bridge_flow <= 0`, also report key `internal_bridge_flow` with message `invalid value {internal_bridge_flow:.6}`, matching the upstream guard in `PrintConfig.cpp:10183-10185` and C++ `std::to_string(double)` formatting.
6. If `internal_bridge_flow <= 0` while `bridge_flow > 0`, return no `internal_bridge_flow` error from this M200 API, matching the upstream source guard.
7. JSON non-integer `skirt_height` values and non-number/non-numeric-string `bridge_flow` / `internal_bridge_flow` values return `SliceError::InvalidInput`; numeric strings remain accepted to match existing Ares numeric option boundary behavior.
8. Preserve existing M196-M199 validation APIs, count APIs, registry APIs, legacy normalization, and FDM normalization behavior.
9. Do not add full validation dispatch, extruder-clearance checks, filament-flow checks, spiral-vase checks, later validation checks, slicing, extrusion, G-code behavior, new crates, or dependencies.
10. Keep modified Rust files under 400 LOC.

## Acceptance checks

- Tests prove default/absent values return an empty validation map.
- Tests prove negative `skirt_height` reports exact message `invalid value -1` under `skirt_height`.
- Tests prove `bridge_flow = 0` reports exact message `invalid value 0.000000` under `bridge_flow` and reports exact message `invalid value 1.000000` under `internal_bridge_flow` when `internal_bridge_flow` is defaulted.
- Tests prove `internal_bridge_flow = 0` alone is not reported when `bridge_flow > 0`.
- Tests prove malformed boundary values return `SliceError::InvalidInput`, while numeric string bridge-flow values are accepted.
- Tests prove existing M196/M197/M198/M199 validation APIs remain intact.
- Plan/spec explicitly account for deferred `PrintConfig.cpp:10187+` validation behavior and deferred `DynamicPrintConfig::validate` dispatch.
- `cargo fmt --check`
- `cargo test`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo check -p ares-core --target wasm32-unknown-unknown`
- `git diff --check`
- LOC check for modified Rust files
