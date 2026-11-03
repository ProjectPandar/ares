# ARD/record: stock rejection parity on current main

Read base: `7a3216f677477d0603a959a18762d5a3ddcf8cb4`.
Upstream: OrcaSlicer `8500fcdccaa10b5099ac20d252af3a7c560046f1`.
Evidence root: `${TMP_ROOT}/ares-stock-rejections-7a/`.

## Decision and scope

Fresh observation of the nine stock `INPUT_REJECTED` rows and nine worker3
read-only observations pending main confirmation establishes **18/18 exact
core validated-slicing rejection payload matches**. No new core detector or
message difference is established. This is a docs-only record, not permission
to implement a new Ares pipeline, modify fixtures, expand validation-only API
scope or change production behavior. Coordinator accepted this conclusion.

Remaining CLI exit/status mapping belongs to worker4. Do not change exit
tables, `FamilyFailure`, `ExitRecord` or transport framing under this record.
The historical1001-row stock matrix is not rewritten or promoted to GREEN.

## Source-cited ownership

| Family | Cases | Upstream owner | Existing Rust destination |
| --- | ---: | --- | --- |
| Relative-E reset |7| `Print.cpp:1732-1738::Print::validate` after `Print::apply` (`:1260`) | `print/validate.rs::validate_layer_templates/layer_gcode` |
| Absolute-E before-layer reset |2| `Print.cpp:1740-1745::Print::validate` | Same applied-print validator |
| Mixed-nozzle bridge width |1| `PrintConfig.cpp:10488-10517::Slic3r::validate`, loaded-config invocation `OrcaSlicer.cpp:3647-3653` | `options/validation/extrusion_width.rs`, public profile entry validation |
| No eligible print-volume objects |4| `Model.cpp::Model::update_print_volume_state`, `OrcaSlicer.cpp:5717-5724`, message `:137` | `project_slice/state.rs` eligible-object guard |
| Embedded postprocess |4| CLI loading guard `OrcaSlicer.cpp:1682-1690`, message `:131` | `project_slice/validated.rs::validate_post_process` |

The last eight cases reject in **validated slicing**, not in
`Print::validate`. Their validation-only `validate_project_plate` calls return
`Ok`; this is an observed API-scope distinction, not a missing core detector.
The existing orchestration remains the compatibility shell around the cited
upstream CLI/model checks. Do not misattribute them to PrintConfig validation.

## Inputs and fresh admission

- Seventeen original saved matrix3MF files were copied without byte changes.
  The nine `INPUT_REJECTED` copies also match historical result input hashes.
- Snapmaker's failed Export produced no3MF, and its three temporary preset
  JSONs were deleted. Its input is explicitly
  **`RECONSTRUCTED_STOCK_SELECTION_NOT_ORIGINAL_PRESETS`**, not an original
  archive replay. Machine: `Snapmaker U1 (0.4+0.6 nozzle)`; process:
  `0.20 Standard @Snapmaker U1 (0.4+0.6 nozzle)`; filament:
  `AliZ PLA @System`. The default `Snapmaker PLA` is incompatible with the
  mixed-nozzle printer; the original stock compatibility/ranking selector
  chooses AliZ. No fixture or numerical parameter was tuned.
- Eighteen fresh Native invocations, once per case, no cache/retry, seed0 and
  pinned Ares/Native CPU0. Whole-case deadline300s covers per-case preparation,
  core calls and Native capture; owned groups/descendants are terminated on
  timeout. All eighteen completed without TIMEOUT and without rejected G-code.
- External public-API probes link core7a3216f67 with the original dependency
  lock and vendored boostvoronoi patch. Repository Rust was not modified.
  Isolated failure modes were recorded in `ADMISSION.md` before probe code.
- Native digest:
  `59fb88a2cb441713c2f572cb06e30e4ab58c22a676cacca1812372b9575e73ff`.
  Resources independently rehashed:
  `286fab66becc4c87ebb72dc8e66ec9d515e7f440d530bacebeb1f51407bb8a69`.
- Full stdout/stderr/result metadata, commands, inputs and hashes are retained.
  Old worker3 Native evidence is read-only historical context, never fresh.
  Initial external Clippy/admission failures were before Native execution and
  remain preserved; they are not successful evidence or Native retries.

## Case inventory

Every row below matches Native's exact rejection payload through validated
slicing. `V` additionally matches through validation-only APIs; `S` rejects
only through the relevant validated-slicing orchestration. Numeric suffixes
identify evidence directories; full stock names/hashes are in the linked record.

| Printer | Family | APIs | Directory |
| --- | --- | --- | --- |
| Anycubic Kobra3 0.2 | Eligible objects |S|`00`|
| Anycubic Kobra3 0.4 | Eligible objects |S|`01`|
| Anycubic Kobra3 0.6 | Eligible objects |S|`02`|
| Anycubic Kobra3 0.8 | Eligible objects |S|`03`|
| Prusa MK3S 0.25 | Postprocess |S|`04`|
| Prusa MK3S 0.4 | Postprocess |S|`05`|
| Prusa MK3S 0.6 | Postprocess |S|`06`|
| Prusa MK3S 0.8 | Postprocess |S|`07`|
| Snapmaker U1 0.4+0.6 | Loaded bridge width; reconstructed |V|`08`|
| Anycubic Kobra Max 0.4 | Relative E |V|`09`|
| Anycubic Kobra Plus 0.4 | Relative E |V|`10`|
| Anycubic Vyper 0.4 | Relative E |V|`11`|
| Dremel3D40 0.4 | Absolute E |V|`12`|
| Dremel3D45 0.4 | Absolute E |V|`13`|
| Raise3D Pro3 Plus0.4 Dual | Relative E |V|`14`|
| Raise3D Pro3 Plus0.4 Left | Relative E |V|`15`|
| Raise3D Pro3 Plus0.4 Right | Relative E |V|`16`|
| Wanhao D12-3000.4 | Relative E |V|`17`|

## Exact messages and authoritative channels

For the seven relative-E rows, full Native stderr equals this payload plus LF:

```text
Relative extruder addressing requires resetting the extruder position at each layer to prevent loss of floating point accuracy. Add "G92 E0" to layer_gcode.
```

For both Dremel rows, full stderr equals this payload plus LF:

```text
"G92 E0" was found in before_layer_change_gcode, which is incompatible with absolute extruder addressing.
```

Their Native result metadata contains the generic CLI_VALIDATE_ERROR message,
not the precise Print::validate payload. Preserve both rather than comparing
the generic family description to a detailed core error.

Snapmaker's entire loaded-config stderr frame is checked exactly:

```text
Param values in 3mf/config error: 
bridge_line_width: Bridge line width must not exceed nozzle diameter: 0.600000
```

`PrintConfig.cpp:10509-10513` resolves percent width against maximum nozzle
diameter but compares bridge width against **minimum** nozzle diameter. Here
absolute width0.6 exceeds minimum nozzle0.4; the printed0.600000 is **absolute
width**, not the allowed bound. Both public profile validation and validated
profile slicing match this payload.

For Kobra3, Native stderr is observed empty. Fresh `native/result.json`
contains the authoritative exact `error_string`:

```text
One of the plate is empty or has no object fully inside it. Please check that the 3mf contains no empty plate in Orca Slicer before uploading.
```

For MK3S, stderr is likewise observed empty; fresh result metadata contains:

```text
post_process is not supported under CLI.
```

Their stdout contains only the generic exit summary. These eight messages are
proven by fresh metadata, not invented from source, exit codes or historical
stdout. Kobra3 Native rejects eligible-object count0 before Print::apply;
MK3S rejects embedded nonempty postprocess during CLI loading, including `[""]`.
Both are matched by existing validated-slicing guards, not Print::validate.

## Repeatable evidence and conclusion

[Machine-readable record](stock-configuration-rejection-parity-record.json)
preserves all eighteen names, exact messages, API results, source attribution,
input provenance and evidence hashes. External root also contains
`fresh-message-stage-audit.json`, `audit_messages.cjs`, `audit-messages.exit=0`,
`fresh-original17.json`, `08/reconstruction.json`, `fresh-identity.json`,
`resources-identity.json` and the full per-case raw captures.

Result: eighteen exact validated-slicing payload matches; ten matching
validation-only rejections; eight explicitly attributed validated-slice-only
guards; zero new core discrepancies, zero timeout and no retry. No production
patch follows from this record. CLI mapping remains worker4's task; unrelated
historical rows, UB/fatal Native cases and all-values/domain coverage remain
outside this audit.
