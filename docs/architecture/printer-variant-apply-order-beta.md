# Printer variant apply order: beta CLI rewrite slice

## Status and owning source

ARD and public RED only; submit their review record before a production port.
Base: `5910df8eb0c2a9cab5958a1dcce7fdb34d075ecb`.

Orca beta source: `345efec64c7b95a05ce5cf96df0dde66a158835c`:

- `src/libslic3r/PrintApply.cpp:1422–1430`, `Print::apply`: call
  `update_values_to_printer_extruders` for `printer_options_with_variant_2`
  with stride 2 **before** `printer_options_with_variant_1` with stride 1.
  Its comment explains that variant 1 reduces `printer_extruder_id` and
  `printer_extruder_variant`, making subsequent lookup against those reduced
  columns incorrect.
- `src/libslic3r/PrintConfig.cpp:11661–11814`,
  `DynamicPrintConfig::update_values_to_printer_extruders`: resolve the
  extruder/variant columns and select `variant_index * stride + i`.
- `src/libslic3r/PrintConfig.cpp:10859+`,
  `DynamicPrintConfig::get_index_for_extruder`: the shared lookup owner.

`PresetBundle.cpp:194+`, `PresetBundle::construct_full_config`, has the
opposite call order at 269–270. That construction path is not the observed
CLI apply owner: the runtime stack is `CLI::run` →
`Print::apply(extruder_applied=false)`. This slice follows `Print::apply`;
it neither changes nor judges the separate PresetBundle path.

## Rust destination and decision

Destination: `crates/ares-core/src/options/project_variants.rs`,
`materialize_project_variants`, and the corresponding lookup commentary in
`options/project_variants/index.rs`,
`resolve_printer_indices_for_variant_two`.

The eventual port must resolve and materialize variant 2 before resolving
and materializing variant 1. Both lookups must see the original printer
id/variant columns. Preserve the existing selector arithmetic, stride,
scalar types, miss policy, process selection, checkpoint, and subsequent
filament materialization. Do not precompute replacement output values or
introduce a compatibility branch.

The typed materialization is a temporary compatibility shell around the
cited `DynamicPrintConfig` operation. Correct its ordering to the upstream
CLI owner; do not add an Ares-owned configuration pipeline or new API.
This remains platform-neutral core logic for WASM, Windows, macOS, and Linux.

## Measured discriminating behavior

The frozen model-ironing project supplies eight `machine_max_speed_e`
values: `30,30,30,30,120,120,120,120`. At the observed materialization entries,
Native and Rust retain this ordered vector and the same printer table:

- ids: `1,1,2,2`;
- variants: Direct Drive Standard, Direct Drive High Flow, Bowden Standard,
  Bowden High Flow.

Native runs stride 2 first and produces `30,30,120,120`, then reduces the
printer metadata. Rust instead runs variant 1 with observed indices `0,2`,
reducing the ids to `1,2`; its subsequent variant-2 lookup returns `0,1`
and selects four `30`s from the still-eight-entry machine vector.
The numeric values were not lost by 3MF parsing.

## Frozen public RED

Fixture: `tests/parity/printer-variant-order/model-ironing-gyroid.3mf`.
SHA-256: `a17ab11edca5ef6ec93aa202d87f75e1898526a604c954dd166b939cc94b0553`.
It freezes the previously qualified model-ironing input without changing
its project settings, mesh, feeds, metadata, or progress commands.

Tests in `crates/ares-cli/tests/orca_parity/runner/printer_variant_order.rs`:

1. Independently run Native twice on the frozen bytes, verify the Native
   vector `30,30,120,120`, and require the public validated API and CLI to
   export the same `machine_max_speed_e` vector. Retain their complete bytes,
   raw API/CLI equality, and unchanged complete-comparator results before
   asserting the focused vector.
2. Independently double-replay Native and run the existing complete ordered
   Native/API/CLI comparator with only its generator-comment exception.

The public configured vector is the discriminating seam. Do not substitute
startup `M203 E120`: that command also depends on the separately deferred
filament map and need not become exact under this ordering-only port.
The focused check is not a full-output parity exception.

## Included and deferred behavior

Include only variant-2-before-variant-1 scheduling and the directly affected
lookup commentary. Leave the key sets, selectors, raw payloads, process and
filament phases, and existing error policies unchanged.

Explicitly defer:

- General filament grouping and map production in
  `GCode/ToolOrdering.cpp:1752+` and the map write-back caller at 3079.
- `Print.cpp:4656+`, `Print::update_filament_maps_to_config`, and its
  multiple-filament rematerialization at 4726. No forced `2,1` map,
  nozzle-map change, or retraction repair belongs here.
- The observed pre-apply E3D High Flow expansion: Native has twelve
  filament-variant values while Rust retains the archive's eight. Its
  earlier preset-refresh producer is a separate slice. Full loader or
  grouping-input equality is not claimed.
- Preset/default refresh, FilamentGroup optimization, writer changes,
  arc fitting, ironing geometry, timing/M73, and footer compatibility work.

## Review and verification gates

Keep ARD, public RED, and any later production port in separate commits.
Deliver the ARD/RED review record before GREEN; do not use a temporary
production edit to demonstrate sensitivity in this phase.

A later approved ordering port must make the focused API/CLI vector match
Native, retain the same frozen test inventory and complete comparator, and
report every full-case result and observable change. The model-ironing
case may remain FAIL because deferred configuration differences persist;
a focused PASS is not complete-output GREEN or proof of a sole timing cause.
Require formatting and strict all-target Clippy without suppressions, and
preserve the deferred source boundaries unchanged.
