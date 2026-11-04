# #7 Stock config-validation parity slice

Status: proposed source-only spec; no new Native probes, tests or production
changes. Historical errors are admission leads, not new
runtime RED or proof that current Core differs.

## Upstream boundary and purpose

Port/confirm only defined stock rejection behavior owned by
`PrintConfig.cpp::Slic3r::validate(FullPrintConfig, under_cli)` at10337–10568
and `Print.cpp::Print::validate` width checks at1524–1556,1652–1692.
This is a source-cited libslic3r rewrite slice, not a new Ares validation pipeline.
The historical exact1cf stock record contains54 ORCA_ERROR rows; the bounded
candidate cohort here is **20 loaded-config plus8 minimum-width rows**.
Signal failures and unrelated error owners cannot enter this cohort.

Rust destinations already exist: `options/validation/{loaded,aggregate,
firmware_retraction,extrusion_width}.rs`, registry metadata for the cited keys,
`print/validate/extrusion_width.rs`, and their existing public validated-project /
profile-entry orchestration. Reuse these owners and their ordered error map;
no parallel validator, new API/crate, blanket stock rejection or fixture-specific
policy. Existing orchestration is a temporary compatibility shell only where
it does not yet match the named upstream phase. This spec does not assume any
such mismatch exists or justify a production change without independent RED.

## Exact historical cohort, messages and source lines

Inventory authority:
`docs/architecture/2026-10-08-stock-1cf-baseline-record.json` rows with
fresh_status ORCA_ERROR. Error lines below are verbatim payload lines, excluding
only transport LF. Loaded errors historically have the full prefix
`Param values in 3mf/config error: ` followed by LF; compare the whole frame,
including ordered keys, whitespace, numeric formatting and trailing LF.

| Stock rows (nozzle sizes enumerate every included row) | Exact historical payload line(s) | Verified upstream owner |
|---|---|---|
| Anycubic Kobra S1 Max0.4; Kobra X0.4 (2) | `retraction_distances_when_cut: 0 not in range [10.000000,18.000000]` | PrintConfig.cpp:5221–5227 defines10/18; numeric vector check10537–10543; exact message10562 |
| Anycubic Kobra S1 Max0.25/0.6/0.8 (3) | `filament_flush_temp: nil not in range [0.000000,1500.000000]` then `retraction_distances_when_cut: 0 not in range [10.000000,18.000000]` | PrintConfig.cpp:2530–2538 nullable integer definition; coInts10551–10557 and serialization/message10562; retraction5221–5227; std::map ordering10339 |
| Creality K2 0.4 (1) | `retraction_distances_when_cut: 30 not in range [10.000000,18.000000]` | PrintConfig.cpp:5221–5227,10537–10543,10562 |
| Creality SPARKX i7 0.2/0.4/0.6/0.8 (4) | `retraction_distances_when_cut: 28 not in range [10.000000,18.000000]` | PrintConfig.cpp:5221–5227,10537–10543,10562 |
| RH3D E3NG v1.2S0.2/0.3/0.4/0.5/0.6 (5) | `use_firmware_retraction: --use-firmware-retraction is not compatible with --wipe` | PrintConfig.cpp:10390–10394, exact text10394 |
| SeeMeCNC BOSSdelta5000521 0.4/0.5/0.7/1.0 (4) | `extruder_printable_height: 2100 not in range [0.000000,1000.000000]` | PrintConfig.cpp:788–796 defines0/1000; vector10537–10543, message10562; this is NOT printable_height whose bound differs |
| Snapmaker U1 (0.4+0.6) (1) | `bridge_line_width: Bridge line width must not exceed nozzle diameter: 0.600000` | PrintConfig.cpp:10488–10517, exact text10513; percent resolves against maximum nozzle but bound is minimum; printed value is actual width, not bound |
| Creality CR-10 SE0.2, Ender-3 0.2, Ender-3 Pro0.2, Ender-3 S1 Plus0.2, Ender-3 V3 KE0.2, Ender-3 V3 SE0.2; LONGER LK10(0.2), LK10 Plus(0.2) (8) | `Too small line width` | Print.cpp:1549 ordinary-width error and1687 bridge-width error; traversal1662–1691 determines the actual first failing key/branch for each fresh input |

The shared minimum-width text alone does not identify which of the two source
branches executed. The input's effective options/used-nozzle
mapping must establish the branch and first failing key; never invent it from
exit205. Loaded-config invocation/frame is OrcaSlicer.cpp:3647–3653; applied
Print validation is a different phase. CLI transport/status classification stays
with the CLI adapter, not this core slice. Source text explains these historical
non-crash errors; fresh admission still must confirm phase and complete payload.

Source SHA256 checked locally:
- PrintConfig.cpp `f635101adb6da3ee657656c6fcaa9449871b7c17446b5e2eb59b560a0eb836aa`
- Print.cpp `c9f66cd1750d12842f80249c99d70a43e976ee035b73dd8c0b1269de07ecc586`

## Existing evidence and input provenance limits

Historical reports already find eight width rows at parity and nineteen loaded
rows matching existing controls; do not label these new defects.
`stock-configuration-rejection-parity-record.md` additionally reports the mixed-
nozzle Snapmaker exact payload matching both Core API paths on7a3216f67.
That input was **RECONSTRUCTED_STOCK_SELECTION_NOT_ORIGINAL_PRESETS**, not an
original failed-Export archive. Preserve this label; do not advertise a remaining
Snapmaker gap merely from the older report's open observation.

The20 failed-Export rows lack original exported3MF/staging presets in the
historical cache. Use the unchanged stock catalog selector,
record machine/process/filament identities and serialize the effective config
before validation using an existing input-capture test seam; reconstructed
selection is not an original-byte replay. Do not repair presets, change default
filament selection, substitute AliZ/other material by hand or tune a valid case.
If original selection cannot be reproduced, classify it INCONCLUSIVE instead of
claiming exact-stock parity. Eight width inputs survive historically; reuse
unaltered bytes with hashes, but fresh Native executions remain required.

## Included and excluded behavior

Included: these28 rows only; numeric vector range/nullable-int serialization,
firmware-retraction/wipe conflict, mixed-nozzle bridge bound, ordered aggregate
first-message semantics and applied width rejection phase/message. Successful
complete-output controls demonstrate validation does not reject legal inputs.

Excluded from the54 ORCA_ERROR: ten SIGSEGV/UB-adjacent selectors; three
Cubicon SIGABRT/empty-vector exceptions; four eligibility/no-object rows
(Model/BuildVolume owner); four embedded postprocess rows (CLI import owner);
five invalid custom-template rows (GCode/PlaceholderParser owner). Also exclude
all nine INPUT_REJECTED and ten VENDOR_INCOMPLETE rows, input-read failures,
crashes, assertions, UB, timeouts and missing vendor/catalog inputs. Relative/
absolute-E reset validations are already separately recorded INPUT_REJECTED,
not this ORCA_ERROR cohort. No expansion to spiral, MM/fuzzy, adaptive, ZAA,
material metadata or new option families.

## TDD and acceptance

Failure inventory before any isolated test: wrong selection/provenance; wrong
validation phase; nil-int sentinel confused with nullable-float NaN; wrong vector
serialization; metadata bound copied from similarly named option; reordered
keys or overwritten earlier message; maximum/minimum nozzle confused; width
zero auto-valid/equality boundary mishandled; support/bridge enablement or key-
major region traversal wrong; nonzero Native failure mistaken for parity PASS;
crash mislabeled validation; historical result attributed to current HEAD.

Plan:
1. Admit each unchanged/reconstructed input once with exact commit, input,
   settings/catalog/binary/resources/argv/seed/status/output hashes outside the
   repository.
   Capture full rejection stderr/result metadata and no produced success G-code.
   Signal/UB/read/vendor/timeout outcomes are excluded, retained and non-PASS.
2. Compare existing public Core validation and validated slicing against the
   precise phase-owned payload, not generic family descriptions. A legal Native
   rejection is the expected behavior; a Core mismatch supplies runtime RED.
   Do not add expected UnsupportedProjectFeature assertions. For legal success
   controls compare complete streams, normalizing only generated-by whole line.
3. If there is no mismatch, submit a conclusion-only exact-input parity record;
   **no production patch**. If mismatch exists, commit the unchanged failing
   public-API E2E/input first, then port only the cited missing validator behavior
   in its existing Rust owner; rerun unchanged RED→GREEN. A new source boundary
   or input-capture mechanism beyond existing seams requires a revised spec.
4. Preserve failed attempts; never retry-to-PASS, alter input to obtain a desired
   failure, widen API validation-only scope to CLI/model checks, or reorder core
   stages to reproduce a Native crash. No new stock whole-sweep result is claimed
   from a28-row report, and rejection parity does not turn ORCA_ERROR into raw
   valid-input PASS.
5. For production changes run fresh ordered same-main seeded pair/common whole-
   byte audit, required-Native workspace, clippy/fmt and core+WASM checks.
6. Source-cited conclusion/ARD.

Exit: all admitted rows have independently bound phase/payload matches or an
honest excluded/inconclusive disposition; success controls unchanged; no claim
of a new detector absent RED, no Native artifact committed, no expanded pipeline.
