# B retained-output / O6 inward-circle runtime Native reference slice

Status: proposed spec, implementation/RED waits for APPROVED.
Owner worker4; Native reference migration only, no production Rust/API changes.

## Owning upstream boundary

- libslic3r/Model.cpp:3423-3469 and BuildVolume.cpp:393-396: actual cylinder
  vertices pass radius55 (maximum vertex54.9) although bbox corner(54.9,4) is out.
  This is not a new Ares eligibility/checking algorithm.
- libslic3r/GCode.cpp:2030 GCode::do_export, :2166-2192 invokes processor validity,
  finalizes/updates stats, transfers result and renames the complete exported file.
- libslic3r/GCode/GCodeProcessor.cpp:1866 check_multi_extruder_gcode_valid;
  the existing error-code4 predicate is preserved, not reimplemented or extended.
- OrcaSlicer.cpp:6209-6227 exports plate_N.gcode THEN examines check error,
  record_exit_reson(-102,index+1) and flush_and_exit. Retained export is not deleted.
- Utils.hpp:74 CLI_GCODE_PATH_IN_UNPRINTABLE_AREA(-102, Linux154), CLI generic
  message :158; record_exit_reson:416-460 and integer wall-clock fields436-437.
- Selected nonminimum plate mapping uses PartPlate::compute_origin and existing
  CLI selected-plate identity; explicit plate2, not first/minimum/plate vector0.

Rust destination: existing OrcaRunner CLI-result cache + orca_parity process/core
API E2E module, replacing temporary frozen test scaffolding around these upstream
Model/BuildVolume/GCode/CLI concepts. Not a new pipeline feature/milestone.

## Exact consumers and assertions

1. Current cli_retained_exit.rs::
   cli_unprintable_area_retains_complete_native_gcode_before_exit_154:
   original o6-stock-circle-inward.3mf, Ares Linux154; exact stdout/stderr,
   complete resultJSON, complete retained G-code after generator-only normalization.
2. Current cli_retained_exit.rs::
   cli_unprintable_area_reports_selected_nonminimum_plate:
   existing ZIP transform adds empty plate1 and remaps instance/sequence to plate2,
   X offset131.9=132-0.1, explicit Ares --plate2. Currently fullJSON compare uses
   frozenplate1 JSON textual index replacement; migrate to ACTUAL Native --slice2
   on same transformed byte-identical input. Preserve status/channels/fullJSON
   plate_index2 and nonempty plate_2.gcode. Add exact retained whole-stream
   comparison to its same-execution Native plate2 output (no fallbackplate1).
3. Current core o6_print_volume.rs::
   circle_bed_per_vertex_accepts_inward_cylinder_then_rejects_export:
   calls slice_project_plate bytes/API deterministic metadata, expects Err with
   exact generic error, typed ToolpathOutsidePrintableArea, error_code4,
   full retained bytes equal Native after generator-line removal only.
   Move this Native-backed whole-byte E2E to orca_parity with the SAME name and
   SAME core API call/assertions; only replace Native expected source. All other
   synthetic/stock off-bed/empty-plate/in-bed core controls remain in core unchanged.

Do not retain only a prefix or bbox approximation. Both existing Native gcode
files are COMPLETE retained exports; no prefix/sampling relaxation. Core error4
assertion remains upstream literal since CLI JSON exposes -102, not processorbits.

Generator normalization: keep original CLI exact one valid Ares2.4.2/Orca2.4.2
line with parsed Native date/time; no extra motion/M73/comment/timing normalization.
Core whole comparison retains original generator-only meaning, with same full
motion/body bytes. JSON only unchanged shared native_wall_clock comparator's
prepare_time/export_time integer literals; all other bytes/fields/f32 text/order
remain exact. Raw saved capture/receipt/cold-hot comparison is unnormalized.

## Runtime recipe / retained publication

Extend the existing runner/cache/cli_result.rs ResultKind with bounded retained
-102/default-first and -102/explicit-second recipes (finite plate choice, not
arbitrary flags/exits). Expose only necessary pub(crate) retained reference helper.
No alternate cache, no changes to worker6 cache/plate_selection.rs or its row,
no global stages::run loosen, no new producer or Native build/prewarm.

Command --slice0 on original input, --slice2 on the existing transformed input.
Expected exact Linux154 + JSON return_code-102 + plate_index1 or2. Exact Native
product inventory is result.json + plate_1.gcode or plate_2.gcode, full nonempty
G-code. Accept only matching Process failure after COMPLETE file emission;
propagate Spawn/Wait/Timeout/IO/unexpected statuses/products. Existing0/100/63/
early5 recipes, request components and public early_cli_reference API unchanged.

Reuse v2 producer/native/resources/replay identity, inputSHA/name + selectedplate/
command recipe/outputname identity, lock, isolated staging/command sealed capture,
SHA checks and atomic publication. Native availability checked before cachedhit.
Each of the two request shapes has immediate hot lookup path/raw bytes equality;
original input/core API case may share exactly same captureddefault entry, not a
second independently manufactured oracle for the same authoritative execution.
Evidence raw stdout/stderr/status/fullJSON/fullGcode/receipt/command captures kept
in verifiable temp/published paths. Required missing producer namedFAIL evenwarm;
optional visibleSKIPPED returns without frozen comparison. No legacy fallback.

## Exact deletion and input provenance

After REAL Native GREEN, delete six frozen Native products:
- cli-exit/path-outside.{stdout,stderr,result.json,native.gcode} (four)
- o6-stock-circle-inward.native-emitted.gcode
- o6-stock-circle-inward.oracle.json (mixed historical Native command receipt).

KEEP o6-stock-circle-inward.3mf unchanged; o6-stock-circle-cylinder.3mf/off-bed
and all hand-authored synthetic inputs remain untouched. oracle.json is not a
live expected-code consumer; it mixes Native exit/result/wholeGcode hashes with
source construction metadata. BEFORE deletion copy only INPUT-AUTHORING fields
into o6-stock-circle-inward.provenance.json: parent/input SHA, only changed XML
entry, translation=-0.1, center50.9, maximum vertex54.9, bbox corner/radius,
mesh SHA and source fixture names. Preserve those values exactly; no Native exit,
expectedJSON/Gcodehash, local/tmp capture paths, singleattempt/paritypass flags
or frozen output reference in the new provenance. It must NOT serve as oracle.

Update existing O6/native-artifact architecture conclusion and live docs links
that point to deleted receipt/file (2026-10-08-stock-circle-verification and
2026-10-07-gcode-printable-area-rejection) to provenance/runtime capture boundary.
Do not restore historical roadmap text or delete authored provenance/inputdata.
No new plan/milestone.

## Writable boundary

- Move cli_retained_exit.rs two cases into orca_parity/cli_retained_exit.rs;
  factor existing plate2 ZIP authoring/generator helper into sibling module if
  needed for400LOC. Preserve both original test names and exact transforms.
- Move ONLY the Native-backed core whole-output case from o6_print_volume.rs
  into that runtime-native E2E module. Other core controls/geometry/processor
  assertions and production code remain untouched. ares-cli tests can call
  the same ares_core byte API; no native/file IO enters core library.
- root orca_parity module registration, own cache/cli_result recipe implementation
  and necessary own public enum/adapter exports only; no worker6 registerchanges.
- Six product deletions, one input-only provenance document, source-cited existing
  docs + thisspec conclusion. Files stay <=400; move proof if structural split.

Deferred: marker fixed-build last group, channel-result unrelated families,
worker6 KSR/replay/processor/plate adapters, production errorcodebit4/predicates,
CLI extra flags/success families and other input datasets. No Native recompilation.

## Independent runtime RED → GREEN → deletion

First standalone policy regression launches EXACT old full retained CLI test
with required1/NativeBINunset, requires running1 selector and named Native
failure. Old frozen case currently succeeds, so parent ACTUALLY FAILS at runtime;
compile errors/fixture-notfound are NOT RED. Commit/log independently, neveramend/
squash RED. This approval does not authorize any production bug fix.

Then move cases and fetch runtime Native with selected identity as above. Same
core byteAPI must remain exact on a live Native execution; do not turn its Err or
code4 assertions into a CLI-only surrogate. Run original3 cases +policy and
existingCLI-result20 + sharedcontracts without widening any comparator. After
all three true runtime comparisons GREEN, delete six outputs, leave input-only
provenance and repeat freshcold/hot bothseeded/official. If unexpected products,
whole-stream/selectedplate mismatch, or known Native ND appears, STOP/report
actual failure and sameinput/raw/receipt evidence; no freeze/retry/mask workaround.

## Acceptance / scheduling

- Required seeded full candidate allPASS; official candidateFAIL name set subset
  of same-round baselineFAIL names, not count-based inference; saved common
  AresRAW and Nativecanon audit, direct retained whole bytes saved independently.
- Default candidate workspace nextest, workspacealltargetsallfeaturesclippy-Dwarnings,
  actualCLIwasmclippy-Dwarnings, ares-wasmwasmcheck, fmt per coordinator gates.
- Required cold/hot/availability capture and proof defaultfirst vsselectedsecond
  request/output identity remain different, JSON/Gcode semantically sameplate.
- One heavy jobs2/inc0/16threads, honor BROWSER/quiet and exact-parent stop rules.
  Source fixed while running; full v2 private producer env includes stats/KSR vars.
- Exact executedhashes/caches/artifacts/ND restrictions, explicit authorizedcarry
  for git-only main rebases, REVIEW+notify/APPROVED/ff-onlypush+notify.

Do not implement or delete until this bounded spec is approved. This is retained
Native test artifact elimination, not completion of all B or marker authorization.
