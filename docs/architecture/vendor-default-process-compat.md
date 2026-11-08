# Vendor default-process compatibility admission

Test-fixture slice beside the `OrcaSlicer.cpp` CLI admission for OrcaSlicer v2.5.0-beta `345efec64`. `runner::vendor_profile_compatibility` reads each stock machine's `default_print_profile`, exports a project with it, and compares the Native and Ares conclusions. At main `2c20f7759` all 10 cases end in the fixture's `expect_err`: the fixture still assumes the pre-2.5 rejection for these printers.

## Problem
The 2.5 resources list every one of the 10 printers in the `compatible_printers` of its own machine's default process, and each of those processes now exists:

| Vendor | Printer | Default process (`compatible_printers` contains the printer) |
|---|---|---|
| Creality | Creality K1 SE 0.8 nozzle | `0.40mm Standard @Creality K1 SE 0.8 nozzle` |
| Ratrig | RatRig V-Core 4 300, 400, 500 0.8 nozzle | `0.30mm Big @RatRig V-Core 4 0.8` |
| iQ | iQ TiQ2 0.25, 0.6, 0.8 Nozzle | `0.15mm`, `0.30mm`, `0.40mm Standard @iQ TiQ2 (…)` |
| iQ | iQ TiQ8 0.25, 0.6, 0.8 Nozzle | `0.15mm`, `0.30mm`, `0.40mm Standard @iQ TiQ8 (…)` |

Files under `resources/profiles/<Vendor>/process/`.

Native therefore admits every case: `check_compat` (`OrcaSlicer.cpp:2890`) sees the printer in the list, the log prints `index 0, new print compatible printer …` (`:2877`) and `… ,compatible 1` (`:2926`), and the export writes the project with exit code 0. It no longer reaches `process not compatible with printer.` (`:3000-3003`) or `can not find setting file` (`:2208`).

Ares already agrees. `PresetCatalogHandle::filter_compatible` (the port of `Preset::is_compatible_with_printer`, `Preset.cpp:909`) returns a filtered catalog containing the process for all 10. A test-only change is enough.

## Upstream boundary
- **Admission:** `OrcaSlicer.cpp:2877-2930` logs the compatible-printer list and the `compatible <bool>` conclusion. `check_compat` (`:2890`) consults the process configuration and the printer's compatible list; `process_compatible` starts from the list membership (`:2963`).
- **Rejections the fixture still names:**
  - `CLI_PROCESS_NOT_COMPATIBLE` (`:3000-3003`), log line `2652: process not compatible with printer.`;
  - missing setting file (`:2208`), log line `can not find setting file: <path>`.
- **Ares:** `PresetCatalogHandle::filter_compatible` → `Preset.cpp:909` `is_compatible_with_printer` (printer list membership, then `compatible_printers_condition`).

## Destination
`crates/ares-cli/tests/orca_parity/runner/vendor_profile_compatibility.rs` only. No production change.

## Change
- **Each case pins its 2.5 conclusion.** The `case!` macro takes an expected `Conclusion` (`Compatible`, `NotCompatible`, `Missing`) beside the vendor and printer. All 10 pass `Compatible`, the conclusion the 2.5 resources determine above.
- **Native admission is observed, not assumed.**
  - A successful `stages::run` is `Compatible`, and it additionally has to leave the exported project (`input.3mf`).
  - An error reads `return_code` from `result.json`: `-17` is `NotCompatible` (exit 239, `2652:` log line), `-3` is `Missing` (exit 253, `can not find setting file` for the target path); any other code panics.
  - Any outcome without an exported project is `NotCompatible`/`Missing` and must leave no `input.3mf`.
  - `assert_eq!(conclusion, expected)` makes a case fail when Native diverges from the 2.5 resources, so the fixture is not "either conclusion passes".
- **The downstream comparison is unchanged.** `native_compatible` derives from the observed conclusion (`Some(true)`, `Some(false)`, `None`), so the exact `ares_compatible == native_compatible` assertion and the missing-name assertion still run for every case.
- **One rename.** `k1_se_08_missing_default_matches_runtime_native` becomes `k1_se_08_default_compatibility_matches_runtime_native`: the 2.5 resources ship that default process, so it is no longer a missing-name case. The other 9 names already describe compatibility.

## Included and deferred
- **Included:**
  - the success admission for the 10 listed printers;
  - the retention of the `-17` and `-3` observations, still asserted for their own cases.
- **Deferred:** `compatible_printers_condition` evaluation, which none of the 10 targets exercises (all conditions are empty), and which `filter_compatible` already implements for the cases that do.

## RED
On `2c20f7759` the 10 cases stop at the fixture's `expect_err`, before the Ares comparison.

`PresetCatalogHandle::filter_compatible` already admits all 10 printers with their default process, so the RED alone closes the gap and no production file changes.
