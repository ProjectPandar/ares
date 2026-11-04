# SPEC — CLI compatible-machine result metadata

**Upward lands first**; downward is a separate later RED/GREEN checkpoint
under this same SPEC. This document ports an upstream CLI metadata slice,
not an Ares-owned slicing pipeline.

## 1. Upstream ownership and observed residual

Source pin: OrcaSlicer `8500fcdccaa10b5099ac20d252af3a7c560046f1`.
Primary owner is `src/OrcaSlicer.cpp`, with configuration and geometry inputs
from existing libslic3r concepts; the adapter's cross-platform filesystem and
platform-specific output behavior stay outside ares-core.

The master-extruder A1 stock **Bambu Lab H2D 0.4 nozzle** input with upward
target **Bambu Lab H2D Pro 0.4 nozzle** is the live residual (input SHA
`6ac0d84efe4010ee563aebb4a17f13c4f15af896377e693c0125ef5e507f4b67`).
Its -100 channel depends on a not-yet-landed `SliceError::Slicing` layer.
Do not relabel Pro as its input machine.

That layer is not imported here. The first independent upward witness must
instead use main's existing
`SliceError::CustomGCode`/-100 channel on a BBL stock H2D/H2D Pro machine with
nonempty upward data and a fixed custom-G-code error. Require runtime Native
-100 plus nonempty upward metadata; independently prove current Ares's same
exit/stdout/stderr before accepting the missing-header-only RED. Preserve
the original A1 assertions unchanged; after upward lands, the master-extruder
slice can close that original header residual without bundling grouping/map
changes. A failed Native admission does not establish our RED.

Current Ares has the typed input option
`PrinterRemainingOptions::upward_compatible_machine`, and
`project/domain/profile_input.rs::initialize_cli_loaded_project_config` models
1769's create=true materialization. Neither is a result-report implementation.
`ares-cli/src/failure.rs::FamilyFailure` and `record_exit_reason` omit both
fields; late `SliceError` variants carry plate/config numbers but no compatible-
machine report. `project_slice/gcode_emit/template_failure.rs` already carries
-100's other result metadata. These are the existing boundaries to extend, not
replace with another pipeline.

## 2. Exact source contract

### Stored report and JSON output

- `OrcaSlicer.cpp:160–181`, particularly179–180: `sliced_info_t` holds two
  initially empty ordered `std::vector<std::string>` values, `upward_machines`
  and `downward_machines`.
- `:416–460`, particularly428–432: `record_exit_reson` emits
  `downward_compatible_machine` only if downward_machines.size()>0, and
  `upward_compatible_machine` only if upward_machines.size()>0.
- Values are JSON string arrays, not footer tokens, semicolon-delimited strings,
  sets, counts or booleans. Preserve element order, duplicate strings, Unicode
  and empty elements. An empty vector means **absent key**, not [] or null;
  a vector containing one empty string is nonempty and must be emitted.
- Preserve the existing Linux-only result.json behavior, sorted JSON keys,
  tab-indented serialization/trailing newline and existing stdout/stderr/exit
  channels. Original-machine resource loading (`:2517–2536`) has no OS guard:
  use CLI-owned std::fs and component-wise Path::join on all native Tier1
  platforms, with identical source selection and no platform Embedded downgrade.

### Upward: source selection, not final effective printer settings

| Source branch | Upstream boundary | Required meaning |
| --- | --- | --- |
| Embedded original 3MF | 1715–1769; assignment1769 | Read original config's upward_compatible_machine with create=true; missing creates empty default. Resolve original system printer via inherits_group last entry or original printer name (1732–1766). |
| Explicit up-to-date files | 2282–2301 | BBL3MF+up_config_to_date, nonempty uptodate_configs: each machine file must match original current_printer_system_name; assignment2301 replaces the vector, including replacement by empty. It is not the new selected printer's vector. |
| Automatic up-to-date original machine | 2368–2408; assignment2385 | With no explicit uptodate files and no new machine, use BBL/machine_full/<original-system-name>.json if present. Missing file retains original values; existing file supplies the replacement, even empty. |
| Fetch-only original system machine | 2512–2536 (requested2517–2540) | When fetch_upward_values, look up the same original machine_full file. Nonempty original system name gates lookup; missing file retains the vector, present file supplies it (2534). load_config_file errors exit before report assignment. |

The flags and order above are part of the contract, not a generic catalog
search. Missing-file retention is **upstream behavior**, not an Ares legacy
fallback. Do not search another vendor, derive an inverse compatibility graph,
hardcode H2D names, or use whichever machine happens to be final after overlays.

`OrcaSlicer.cpp:2575–2654` uses process/printer compatibility and may reject
before assignment. Only `:2656` commits the chosen vector to
`sliced_info.upward_machines`. A failure before this point must not acquire a
header merely because a vector was read earlier. Actual process compatibility,
machine-switch decisions/-17/-16 and profile replacement remain their owning
rewrite slices; this work consumes their established stage outcome, not changes
it. In particular PCP embedded print_compatible_printers coverage is unrelated
to these output headers.

### Downward: actual candidate acceptance, not a configuration vector copy

`downward_compatible_machine` is an output of checking, not an input option to
add to the option registry. It is not the inverse of upward_compatible_machine.

- `PrintConfig.cpp:10926–10935`: CLI downward_check defaults false;
  downward_settings is the ordered candidate-file list. CLI reads the flag at
  `OrcaSlicer.cpp:1361–1363`.
- `OrcaSlicer.cpp:1137–1187,4082–4090`: absent explicit candidates use
  BBL/cli_config.json's printer/<model>/downward_check/<original-system-name>
  list and BBL/machine_full files. Missing/malformed discovery JSON follows the
  upstream empty-list/diagnostic behavior, not a guessed catalog list.
- `:4092–4117`: load each candidate and require machine/system metadata; the
  reported name is config_name, not filename or model ID. Preserve candidate
  enumeration order and multiplicity. Note4103–4104 actually pass the load's
  `ret` in the wrong-kind/from branch, not a freshly selected error code; do
  not silently repair that upstream behavior. Valid machine/system sources
  are the initial admission boundary; broader loader-error parity is deferred.
- `:4118–4173`: construct printer_plate_info_t from printable/excluded/wrapping
  regions with the upstream diagonal-corner differences and integer casts;
  intersect multi-extruder bounding rectangles and take the minimum printable
  height under the exact area-count/height-count condition. This is not the
  polygon-containment or print_height validation algorithm.
- `:3892–3953,4063–4071`: plate_obj_size_infos supplies the all-plate object
  bounding box and, when eligible, the estimated wipe-tower/brim extent.
  Owning geometry comes from `PartPlate::get_objects_bounding_box` and
  `estimate_wipe_tower_size` concepts; do not use only the selected printable
  object, or silently ignore other plates/tower extents.
- `:4177–4233`: a candidate fails permanently if **any plate** exceeds width,
  depth or height (`>`; equality passes), or fails the exact exclusion test
  (both X and Y exceed the residual dimensions), or enabled wrapping test
  (Y exceeds depth minus wrapping_depth). Preserve boolean conditions and
  numeric conversions; do not reuse another stricter test because it looks
  similar. The selected slice plate does not narrow this all-plate check.
- `:4234–4246`: append every surviving candidate name in original order and
  commit to sliced_info.downward_machines. Disabled check, zero candidates or
  all failed candidates leave the report empty and the result key absent.

## 3. Rust destination and non-negotiable boundary

### ares-core: source selection, computation and stage-correct data

Proposed destination: dedicated small modules under
`ares-core/src/project_slice/compatible_machines/` for stage state/downward
geometry, and `ares-core/src/profiles/` for the original-system machine source
selection. Reuse typed printer options, existing profile JSON parsing and
Project/plate geometry rather than parse profiles or calculate dimensions in
ares-cli. No filesystem, environment access, terminal output, OpenGL or GUI
PartPlate runtime may enter core. Each touched/new Rust module stays <=400LOC;
split the existing error/report code if extension crosses that threshold.

Core owns a minimal typed report of two ordered string vectors. Availability
of original-system machine JSON, ordered explicit/up-to-date machine inputs,
optional cli_config discovery bytes and ordered downward candidate JSON arrive
as **byte/data inputs** from adapters. Source control flags/name provenance
must remain distinguishable; an absent system file is not an existing empty
vector, and a parse failure is not absence. Reuse existing byte-oriented profile
entry boundaries where possible; only a minimal report/resource input seam is
permitted if those boundaries cannot carry this upstream information.

Report state starts empty; upward becomes populated only after the equivalent
2656 stage, downward only after4246. Later error transport, including -100,
carries the reached report snapshot through existing SliceError/result
metadata into the CLI. Do not compute a report afresh in failure.rs from final
printer settings, or eagerly attach future-stage values to early errors.
An error at profile-source load before2656 has no copied upward report. An
error during downward checking may already have upward but not downward.

The bounded upward checkpoint requires only the first CLI E2E
witness, comparing complete runtime Native and Ares channels/result JSON.
Coverage of source paths1769/2301/2385/2534 remains a subsequent CLI E2E
requirement, not an isolated/unit-test substitute. Future witnesses must exercise
the public source controls and compare runtime Native outputs in full; absent
public controls remain deferred. This does **not** include implementing
unrelated up-to-date overlays or compatibility decisions merely to
exercise a branch. No four-branch acceptance claim is made by this checkpoint.

### ares-cli: resources/filesystem and JSON serialization only

Extend `FamilyFailure`'s existing result transport and
`record_exit_reason` (or a focused result writer submodule) to consume the core
report and conditionally serialize both arrays. CLI resolves/reads the exact
resource paths and explicit candidate files; it may translate flags/files into
core requests, but may not choose surviving machines or recompute geometry.
For an active downward public path, use only source-owned downward-check/
downward-settings controls, not a user-supplied final output vector.

Use one existing error/report route for ordinary and dump-moves/selected-profile
entries. Empty/default context must preserve existing early error bytes.
Success-side result.json expansion, sliced_plates, new error families/codes,
up-to-date profile overlay implementation and standalone GUI downward-check
mode are adjacent behavior, not license to grow this slice. A bounded early-
exit report port must not be described as complete success-report parity.
Core report computation remains portable on browser WASM/Windows/macOS/Linux;
Linux result-file emission stays an adapter detail.

## 4. Included versus deferred and staging

Included: correct late family-error headers; original embedded/system-machine
source selection with all four cited branches; conditional JSON omission and
ordered array fidelity; core-owned active downward candidate calculation and
report transport, using supplied source data/existing geometry. The H2D Pro
-100 public CLI witness is the first required end-to-end closure, not the sole
proof of both directions.

The active downward calculation is a separate RED/GREEN checkpoint within
this source slice. **Do not certify downward support from an empty
control or omit it from a completion claim.** If the existing all-plate/tower
preparation or resource entry cannot expose the source-owned input, mark that
checkpoint BLOCKED with an explicit narrower follow-up; do not
invent dimensions, add fallback selection or silently mark it GREEN.

Deferred adjacent behavior: arbitrary preset compatibility filtering/-17,
full machine-switch/up-to-date overlay parity, unrelated custom-G-code fixes,
complete successful result.json, all CLI error-family additions, GUI-only
runtime, and global option/domain completion. Existing Ares entries remain
temporary compatibility shells wherever the cited upstream prefix is absent;
they are not an alternative pipeline design.

## 5. Test-first failure model and gates

Before implementation, write source-backed E2E failures for these risks:

1. Missing both headers at late -100 despite a nonempty reached report.
2. Empty vector emitted as []/null; single empty element wrongly omitted;
   ordering, duplicates or Unicode lost; direction swapped.
3. Reading the new/effective printer instead of original system machine;
   ignoring explicit/update/fetch replacement; keeping embedded values when an
   existing replacement is empty; treating malformed source as absent.
4. Headers leaking into pre2656 rejection; losing upward on a later failure
   before downward commit; early -3/-18/-19 controls changing unexpectedly.
5. Downward inferred from upward/catalog names; accepting a candidate that
   fails a nonselected plate; mishandling equality/int casts/excluded regions,
   wrapping gate or multi-extruder intersection/minimum height.
6. Ignoring eligible tower/brim extent or using validation-time geometry
   instead of the source downward-check prefix; losing candidate order.
7. Missing/invalid configured Native falsely marked SKIP, partial channels
   accepted, or normalization removing the new fields to make comparison pass.

First RED: use the independent stock BBL custom-G-code error witness; runtime pinned Native must prove -100 plus nonempty upward.
Prove matching exit family/stdout/stderr first,
then retain full result.json bytes with the missing fields as the only intended
repair boundary. If other semantic discrepancies occur, diagnose instead of
calling them this slice's RED. Do not delete either field in the comparator.

After an independently committed genuine RED, implement core
calculation/stage transport plus CLI writer. Keep the original A1
assertions and the whole result comparison; only existing integer prepare_time/
export_time adaptation is allowed. Then add bounded active-downward RED before
its implementation; cover surviving/rejected/all-rejected, multiple plates,
exclusion/wrapping and multi-extruder/tower source conditions with fixed inputs.
Use public CLI E2E with complete runtime Native comparison as the primary
verification mechanism. For this bounded upward checkpoint only the first
witness is required. Missing source controls remain subsequent E2E work;
do not add implementation-following isolated/unit tests as substitute coverage.

No new native probe/kernel tool or committed Native product is needed.
Required Native absence/invalidity is FAIL; optional absence is visibly
SKIPPED.

Run workspace via cargo nextest, strict Clippy/fmt/WASM and the full
independent BASE/CAND seeded/official controls; keep every raw difference.
