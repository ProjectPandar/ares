# Timelapse witness: source-cited G-code printable-area rejection

Date: 2026-10-07. Status: proposed; implementation not started.
CLI `-102` mapping is a separate CLI-adapter slice after `-100`; it is not
implemented by this spec.

## Observed missing boundary

Preserved `option/timelapse_type/1` input SHA256:
`67a27bafcda838742db058b1edb812055c9470608e47dc83a314357a8e9b233d`.

Historical Native did export `plate_1.gcode`, then returned `-102`
(`CLI_GCODE_PATH_IN_UNPRINTABLE_AREA`, Utils.hpp:74). Its stderr was empty;
the output's `result.json` contains:

```text
Found G-code in unprintable area of multi-extruder printers after slicing. Please make sure the 3mf file can be successfully sliced in the latest Orca Slicer.
```

Fresh cargo-built Ares on main `e5fe2bba8` accepts the same bytes, exits0,
and writes G-code. Binary SHA256:
`d8b1a25d0f348fb41f3145b5f62ea7be668d0be20628391c612b207e06574513`;
output SHA256:
`8e4a35544f1597ae5198ba189f10622de290a4adf94ae1a7e25c27839cdba6d8`.
Build, argv, input/output hashes, exit and public `--dump-moves` artifact
were retained externally. This is a diagnostic witness, **not the separately committed/native-run RED gate**.

The existing core has model/build-volume eligibility and Print validation,
but no corresponding processed-G-code `gcode_check_result` decision.
Do not substitute a model bbox check or reject `timelapse_type=1` by value:
the missing concept is the **emitted non-custom extrusion path** check.
SLA→FFF remains a separate oracle request-not-effective record, not PASS.

## Owning upstream slice

Source: `OrcaSlicer/src/libslic3r/GCode/GCodeProcessor.cpp`:

- `GCodeProcessor::check_multi_extruder_gcode_valid`, :1866 onward.
- :1876 resets `GCodeCheckResult`; :1895-1917 gathers extrusion positions
  and maximum print Z by object-label/filament id. Travel is excluded.
  Custom-role extrusion is gathered separately but not included in the
  active bbox/height checks; commented arc-interpolation paths are inactive.
- :1921-1930 translates the plate polygon by processor XY offset and
  expands its valid bbox by2mm. A missing/invalid polygon leaves the bbox
  undefined rather than introducing a new configuration rejection.
- :1932-1958 checks non-custom group bbox against plate bbox (bit2), and
  group maximum print Z against printable height (bit3).
- :1970-1999, only with multiple nozzles, maps filament to physical
  extruder and checks translated unprintable polygon bboxes (bit0) and
  per-extruder maximum height (bit1). Native uses bbox overlap, not exact
  polygon clipping, and keeps the current boundary/tolerance predicates.

Call boundary: `GCode.cpp:2166-2173`, after G-code emission/processing and
before processor finalization. Nozzle count comes from `nozzle_diameter`;
plate area/height, XY offset, per-extruder excluded polygons/heights and
filament maps come from Print's existing configuration/geometry views.
The caller uses the result, not a new independent Ares policy.

CLI boundary: `OrcaSlicer.cpp:6220-6227` checks the result's error code
**after export** and returns `-102`, with the :158 diagnostic. That is
neither `-100` placeholder parsing nor an invalid enum option. Native
emission followed by rejection is not a successful parity stream.

## Retained emission on -102: source and additional RED

`GCode.cpp:2176-2181` finishes statistics and extracts the processor result;
`:2186-2194` renames the complete `.tmp` stream to the requested output file.
`OrcaSlicer.cpp:6209-6215` selects `plate_1.gcode` and calls export; only then
`:6218-6227` records the check failure and returns -102. The rename at
`:6239-6247` is commented out. `flush_and_exit` at `:398-413` flushes streams
and removes model backup directories, not the exported file;
`Model.cpp:1001-1012` confines that cleanup to `backup_path`.
Therefore the generated G-code remains available even though the final
result is rejection, not a successful parity stream.

Additional unchanged stock-circle input:
`tests/parity/validation/o6-stock-circle-inward.3mf`, SHA256
`8958ba6462946f57d1fc2d6443e6c7fac736a87be5e89dc68d2bbfc8837c1b13`.
Center 50.9 gives maximum vertex radius 54.9, while bbox corner (54.9,4)
has radius 55.04552661206903. Its single validated fresh Native run passed
Model, retained 167943 G-code bytes (raw SHA256
`cabb272d646a6404e88db9af3c1e7dd8966173aff96afb60b856a744948e1f4a`),
then returned -102 / Unix154. Input-authoring provenance is retained in
`tests/parity/validation/o6-stock-circle-inward.provenance.json`; historical
Native captures remain outside the repository, not as repository oracles. The same-named byte-API E2E now acquires complete retained bytes from
runtime Native in `crates/ares-cli/tests/orca_parity/cli_retained_exit.rs`.
The circle slice proved whole-byte emission equality before the export check
was ported; the timelapse RED must retain that assertion against the error's
owned output, as well as exact diagnostic and bit2 (`error_code=4`).

The test-only runtime RED `124cebaf` already demonstrates that the current
base returns successful bytes instead of -102. The strengthened RED will
match the typed failure and assert its code and complete retained bytes,
normalizing only the generator line, with no motion/M73 normalization.
A minimal error-variant declaration may scaffold that RED's compilation;
no check/return behavior is implemented until the RED fails at runtime.

## Rust destination and non-negotiable decisions

Architectural authority: [the proposed ARD](../../architecture/2026-10-07-gcode-printable-area-boundary.md).
Concrete result/check destination:
`crates/ares-core/src/gcode_processor/printable_area.rs`.

1. Add a small rendering-neutral core module owning the rewrite of this
   result/check, under the existing G-code processor boundary. Split into
   files rather than expanding an already-large processor/emit module.
   Existing move processing is a temporary compatibility shell around
   `GCodeProcessorResult::MoveVertex`, not an Ares-owned validation pipeline.
2. Evaluate the existing processor pass's emitted moves before a slicing
   API exposes success bytes. Keep the upstream extrusion/custom-role,
   group, float-position/coordinate-scaling, offset,2mm tolerance, mapping
   and active bbox/height semantics. Do not parse the G-code again solely
   to validate it, copy all moves, or add an unrelated geometry pass.
3. Preserve object/filament grouping and print-Z meaning at the processor
   boundary. Current public `MoveVertex` lacks object-label and print-Z
   fields; retain the needed upstream state/result in the processor's
   owning implementation, rather than incorrectly grouping by layer or
   using position Z as an unexplained replacement. Review the existing
   move/tag producer before selecting the exact private state seam.
4. Surface a distinct typed `SliceError::ToolpathOutsidePrintableArea`
   (carrying the ported active check result when needed), with the exact
   :158 display message. It must not be `InvalidInput` string matching or
   a timelapse-specific guard. Result details follow upstream flags/groups;
   do not invent speculative configuration or viewer APIs.
5. The shared core slice completion boundary, including byte-only,
   processed and profile entry paths, must not return successful G-code
   for a failed active check. Its typed error must carry both `error_code`
   and the complete already-generated `gcode: Vec<u8>`, transferring ownership
   rather than copying the stream. Native preserves the exported output file
   on -102; losing its bytes in core would drop upstream behavior. Core still
   performs no file I/O; the CLI adapter writes the retained bytes and maps
   the CLI status. All four Tier1 platforms share this pure byte/data boundary.
6. The CLI adapter owns typed-error→`-102` CLI mapping and adapter error/status
   tests. This slice must not duplicate it or change unrelated mappings.
   Until that dependent slice merges, core rejection is verified separately
   from CLI status; the generic CLI path is not final `-102` parity.

## Included and deferred

Included: active plate bbox/height and multi-extruder bbox/height rejection
bits0–3, required upstream grouping/mapping/offset data, and typed core
failure. This is a source port, not a broad configuration rejection rule.

Deferred: commented wrapping-exclusion bit4 and interpolation checks;
viewer-only `limit_filament_maps` / physical-unprintable-filament metadata
that does not set the active failure; unrelated renderer/runtime code;
SLA generation; replacing timelapse emission logic; all new CLI mappings.
Existing model eligibility, option validation and role parsing remain
separate source-owned boundaries and cannot substitute for this check.

## RED → GREEN and exit criteria

Before production edits, enumerate isolation failure modes: custom/travel
false positives; too-wide tolerance or polygon-vs-bbox substitution;
incorrect group union; wrong filament map/index; lost XY offset; confusing
print Z with position Z; applying multi-head limits to a single nozzle;
missing byte-only entry checks; and classifying returned failure as PASS.

Write and separately commit an E2E RED using the preserved same input;
run the authoritative Native once with fresh case evidence, require its
post-export `-102` result, and require Ares to return the typed failure
instead of success bytes. Keep all original option/value comparisons.
The historical Native observation alone is not fresh RED evidence.

Use paired source-driven controls for each included predicate: inside/just
outside the native plate tolerance, height boundary, per-head exclusion
bbox/height, mapped filament/object groups, custom-only/outside travel
(which must not reject), single-head behavior and unchanged timelapse0.
Prefer public byte/profile E2E composition, with repeatable input/output
or rejection artifacts; write tests before the port.

After GREEN: workspace nextest, clippy without new warnings, fmt, wasm32,
applicable seeded/official/profile controls. Native rejecting cases report
both rejection outcomes, not ordered-byte PASS. Preserve successful control
streams with generator-only normalization. Submit spec/RED/implementation
and evidence for independent review before merging. The concurrently
running fresh stock baseline remains pinned to `e5fe2bba8`; it cannot be
relabelled as a post-implementation gate.
