# SPEC / ARD: bounded embedded print_compatible_printers coStrings fidelity

Status: **REVISED SPEC ONLY — re-approval requested before further Native runs,
RED or implementation.** Original spec `310d6b32a` was approved; the first
read-only admission failure and owner-authorized revision are recorded below.
Owner: worker1; coordinator explicitly confirmed this key unclaimed and assigned
it to worker1. Read base: main `b1b958e17f23becfdd9923ee8b5a3c9a25427d1c`.
Upstream pin: OrcaSlicer `8500fcdccaa10b5099ac20d252af3a7c560046f1`.

## Non-negotiable rewrite boundary

This is a bounded test-harness coverage slice of upstream **embedded 3MF typed
configuration reading and serialization**, not a new Ares compatibility engine.
The key's name does not authorize testing or implementing printer compatibility.

Owning sources:

| Upstream boundary | Included responsibility |
| --- | --- |
| `src/libslic3r/PrintConfig.cpp:1832–1834`, `PrintConfigDef` | `print_compatible_printers` is `coStrings`, default empty vector, `nocli`; no finite declared string domain |
| `src/libslic3r/Format/bbs_3mf.cpp:210`, importer dispatch `:1923–1926`, `_BBS_3MF_Importer::_extract_project_config_from_archive:2632–2653` | Read `Metadata/project_settings.config` from the actual archive and hand it to typed `DynamicPrintConfig::load_from_json` |
| `src/libslic3r/Config.cpp:820–875,975–1000`, `ConfigBase::load_from_json` | JSON string-array elements are escaped/quoted individually, joined by `;`, then deserialized according to `coStrings`; array members are not parsed as printer IDs or compatibility expressions here |
| `src/libslic3r/Config.hpp:1118–1158`, `ConfigOptionStrings` | Ordered string-vector storage; `vserialize()` returns the original string elements; `serialize()` uses `escape_strings_cstyle`; `deserialize()` uses `unescape_strings_cstyle` |
| `src/libslic3r/Config.cpp:72–120,146–218` | Existing C-style string-vector serialization/deserialization boundary, not a newly invented encoding |
| `src/libslic3r/Config.cpp:1464–1504`, `ConfigBase::save_to_json` | Vector `vserialize()` becomes a JSON array, retaining empty elements, order and text |
| `src/libslic3r/Format/bbs_3mf.cpp:7722–7727`, `_BBS_3MF_Exporter::_add_project_config_file_to_archive` | Write typed config to `Metadata/project_settings.config` through `save_to_json` |

Explicit Rust destination boundaries:

- Existing `ares-core/src/project/load.rs:118–149` archive settings extraction,
  `ProjectSettingsRaw` and typed `ProjectSettingsBuilder` decoding.
- Existing `ares-core/src/options/project_runtime_options/preset_source.rs`,
  `ProjectPresetSourceOptions::print_compatible_printers: OrcaStrings` and its
  existing Serde serialization (`preset_source/wire.rs`).
- Existing `ares-core/src/options/config_export/value/strings.rs` and configured
  `print_compatible_printers` export entry when observing config serialization.
  Do not change generic string encoders merely to make this coverage pass.
- Proposed executable coverage belongs in a key-local
  `ares-cli/tests/orca_parity/option_coverage/print_compatible_printers.rs`,
  with only necessary plan/metadata/execution delegation in the existing
  option-coverage harness. No production change is authorized by this spec.

The Option harness is a temporary inventory/execution compatibility shell
around the above upstream structures. Its baseline-only `value:null` is not a
source-backed proof of explicit string injection, and its generic omission
wording is not authority to declare all strings complete. Existing Ares typed
storage/serialization is reused, not redesigned or treated as an Ares pipeline.

## Ownership fences — do not cross

Worker5 owns preset compatibility filtering, process/machine compatibility
selection and `--load-settings` overlay/compatibility decisions, including the
adjacent `OrcaSlicer.cpp` current/new compatibility-list logic and replacement
of embedded fields. None is included, even if Native CLI machinery happens to
visit those functions. Do not modify those functions, harness hooks or their
fixtures. Do not prove filtering by asking whether a listed name exists or a
printer is permitted.

Also excluded: worker3 printable_height/G2 clearance; worker6 wrapping geometry;
worker2 G4 IDs/mapping; worker5 load-settings areas. Avoid their shared-file
writer windows and retain all existing bridge/filament/G2/APA dispatches.

The existing Native CLI compatibility gate may execute as an admission
precondition only. The owner explicitly authorized the current machine's
`printer_settings_id` as a fixed element of every nonempty witness vector;
compatibility PASS is not a fidelity assertion or a new worker1-owned feature.
No substitution of `compatible_printers`, profile edits, load-settings overlay,
filtering implementation or compatibility validation assertions. Unexpected
field replacement or rejection after this fixed revision still means STOP and
ask the owner; do not iteratively tune names or reorder/drop difficult elements.

## Retained initial admission and owner ruling

The first authorized short `--slice 0` invocation used
`["Embedded Printer B", "Embedded Printer A", "Embedded Printer B"]` with no
current machine name. Native returned **-17**, shell exit239; its `result.json`
says `The selected printer is not compatible with the process preset in the 3mf.`
No G-code was generated. Sources: `OrcaSlicer.cpp:2653–2654` reports/exits
`CLI_PROCESS_NOT_COMPATIBLE`; `libslic3r/Utils.hpp:42` defines that code as -17.
This is genuine upstream behavior, not missing Native or classification RED.
The second planned `--export-3mf` invocation was not executed after the stop.
Evidence: `${TMP_ROOT}/ares-pcp-embedded/admission/{input-receipt.json,native-slice/command.txt,native-slice/exit,native-slice/result.json}`.
The initial mistaken tower-enabled fixture was rejected during offline preflight
before any Native invocation and is retained separately; no Native retry used it.

Coordinator ruled: preserve -17 evidence; revise the seven fixed witnesses so
all nonempty vectors contain the current machine's `printer_settings_id`,
preserving the other order/duplicate/escaped/Unicode elements. Absent and `[]`
stay unchanged. The nonmatching-machine -17 rejection slice belongs to worker5
after load-settings, and is **deferred here**. This is an explicitly authorized
new matrix, not an unnoticed success-seeking retry of the failed sample. Further
Native admission waits for re-approval of this revision. No new probe tools.

Approved revised admission (existing Native only),
`${TMP_ROOT}/ares-pcp-embedded/admission-r2`: both `--slice 0` and `--export-3mf`
exit0. Footer carries the exact serialized M/B/A/B sequence; the actual
roundtrip `Metadata/project_settings.config` vector equals the requested
ordered four elements. `native-observability.json` and command/load/input-member
receipts retain the observations. This proves these existing channels exist,
not that the remaining cases or classification already passed.

The first test attempt (`red-official`, run
`1f7aff1c-b4ca-4f73-887c-5edf099d4610`) stopped on a **harness adapter error**:
direct `OrcaStrings` Serde expects a JSON sequence, not the Native footer's scalar
serialization token. This is neither fidelity RED nor classification RED.
Do not reinterpret that exit100 as desired RED. Retain it separately.
The footer is compared as serialized bytes in the unchanged complete-stream
comparison, while Native writer JSON (`ConfigOptionStrings::vserialize`) is
the independent vector/cardinality witness. In particular `Config.cpp:72–120`
does not quote semicolons in otherwise-simple elements, so parsing its footer
as a reversible vector is not an upstream guarantee. Keep the semicolon sample;
do not change the serializer or assert invented roundtrip-token semantics.
Further Native test execution waited for worker2's full-pair END.

## Established Native-backed classification RED

After production advances, the test was rebuilt on `880a6c8d5` (candidate
`a6717131011295206602ab3872a1c2b07b75fb54`). Owner authorized the later
web/docs-only `b0b7905fa` git-only carry to `89d8c07d6`; the Native run uses the
completed 880 archive, not an assertion that a new archive was built on b0.
Read-only directed official run `4d38b40f-14f5-47f7-bcba-98476e2a5100`,
`${TMP_ROOT}/ares-pcp-embedded/red-880-official`, admitted at load1=2.71<30 after
worker6 END and before worker3 full START, 2 nextest threads, fresh cache/dir.
No Native timeout or retries. Retained baseline plus **all seven revised cases**
PASS public Ares typed/Serde checks, live Native footer presence, actual Native
`--export-3mf` vector JSON equality and complete ordered G-code comparison.
Only the final bounded classification assertion fails:
INCOMPLETE versus PASS (baseline plan still contains noninjected null).
Exit100 is therefore classification RED, not the earlier adapter failure or -17.
`context-results.json` and all eight per-case Native/3MF/input/results remain
outside Git. Original assertions, escaped/empty/Unicode samples and deferred
worker5 rejection boundary are unchanged. No GREEN implementation yet.

Cancelled new-tree compilation windows `red-ee277-official` and
`red-ee277-built` never started Native and are not RED evidence. The 880 archive
build completed before worker6's exclusive full window; no Native was executed
in that window. Scheduling/admission receipts and original cancellation logs
are retained rather than overwritten.

## Harness GREEN implementation checkpoint

Only this key's bounded plan/classification metadata is promoted. Its seven
canonical plan values are **expected typed vectors**, not a claim that a
missing raw key was injected: `missing-default` has expected `[]` with raw
`remove_key` action, while `explicit-empty` has `[]` with raw `set_empty_array`.
Metadata exposes this distinction. The retained original baseline remains an
extra E2E control; it does not turn a baseline-only null plan into complete
coverage. `OptionOutcome::executed`'s global null/unproven rules are unchanged.

The sweep delegates only this key to the exact existing eight-witness E2E
routine (seven revised cases plus retained baseline), instead of preset export
with load-settings. All typed, writer JSON and complete-stream assertions are
preserved verbatim; no comparator, tolerance, string parser or production code
changes. An unproven clone remains INCOMPLETE. A value-filtered plan retains the
harness's normal incomplete flag; exercising the extra baseline cannot certify
unexecuted required values. Other bridge/filament/G2/G4/wrapping hooks remain.

Directed official and seeded cold/warm runs each PASS both tests: bounded
classification and selected-key unfiltered-value sweep. Evidence:
`${TMP_ROOT}/ares-pcp-embedded/{green-first-official,green-official-hot,green-seeded-cold,green-seeded-hot}`.
Warm runs share only the original-project export cache; the footer slice and
Native 3MF roundtrip commands are still live/fresh per witness. Do not claim
these are cached Native fidelity results. The key-local module remains under
400LOC. New-tree workspace/strict and queued independent full control pairs
remain required before exact-hash review/integration; this checkpoint is not
approval to push.

## Inventory evidence and honest coverage meaning

`${TMP_ROOT}/ares-apa-g1-bridges/other-plan-inventory-audit.json` is an invariance audit,
not the full inventory: it records only the previous APA key delta. Its full
associated plan file is
`${TMP_ROOT}/ares-apa-g1-bridges/green-first-official/artifacts/option-domain-plans.json`.
There, this key is `coStrings`, source `PrintConfig.cpp:1832`, one baseline case
with `value:null`, `declared_scope_complete:false`, omission
`unbounded domain (string/point/blob) — baseline only`.
`docs/architecture/2026-10-09-option-full-current-audit.md` also records it as
INCOMPLETE. Later worker-owned completions are not inferred from this older
snapshot; this specific key was re-confirmed unclaimed by the coordinator.

Success can establish only the finite **embedded string-list fidelity scope**
below. It does not close the unbounded string universe, arbitrary vector
lengths, catalog filtering, validation of names, or all legacy encodings. Any
PASS metadata must name this scope and its omissions, not claim global schema
completion or printer compatibility. The historical baseline assertion stays;
an absent/default case cannot substitute for explicit cases.

## Fixed witness matrix (declare before running)

Use the established Native-produced classic `cube10.stl` single-logical-filament,
no-prime-tower project retained in `${TMP_ROOT}/ares-pcp-embedded/admission/original.3mf`
(source: prior `case-1D75FV`, adaptive-pa/no-cooling). Keep its fixed process
settings rather than claiming untouched profile defaults. Inject only this key
in `Metadata/project_settings.config` for each
case. No `--load-settings` or `--load-filaments` during witness replay. Keep all
other named archive members byte-exact to the fixed initial project. Retain
raw ZIP/member SHA receipts; changing this JSON key is deliberate, not a claim
of whole-archive equality across cases.

`M` below is the original archive's unchanged typed `printer_settings_id`,
verified as `"Creality Ender-3 0.4 nozzle"`. It is not resolved by preset lookup,
loaded from another profile, renamed, or inserted into any other field. Capture
and assert this exact fixture receipt before executing the revised matrix.

| Case | Embedded JSON value (`M` substituted literally) | Fidelity assertion |
| --- | --- | --- |
| missing-default | key absent | Existing typed default is an empty vector; do not claim absent raw key equals explicitly present empty key |
| explicit-empty | `[]` | Zero elements; distinct from a vector carrying an empty element |
| empty-element | `[M, ""]` | Two elements including the empty string; this is NOT a lone-empty singleton test |
| one-label | `[M, "Embedded Printer A"]` | Whitespace and both elements preserved |
| ordered-duplicates | `[M, "Embedded Printer B", "Embedded Printer A", "Embedded Printer B"]` | Preserve order, duplicate nonmachine labels and length; no deduplication |
| escaped-elements | `[M, "semi;colon", "quote\"slash\\", "line\nnext", "tab\tvalue"]` | JSON string characters/element boundaries preserved through typed read and vector JSON serialization |
| unicode-label | `[M, "打印机 Ω"]` | Preserve UTF-8 text and both elements; no normalization or name validation |

Only `M` is a real fixture machine identifier, admitted by explicit owner ruling.
Other elements remain opaque text, not required to identify real presets. No
assertion establishes name validity or filters compatible printers. The revised
seven cases are fixed before renewed admission; failures must not lead to
further value/count/escaping tuning or dropping a difficult case.

Included observables, in order:

1. Request archive's actual JSON value and all untouched member receipts.
2. Runtime Native typed vector read from this embedded archive, not merely a
   copy of input JSON or the harness's requested map.
3. Runtime Native vector JSON serialization/3MF roundtrip value, with original
   element strings and cardinality. Compare the keyed typed value, not arbitrary
   exporter ZIP timestamps or unrelated exporter fields. Keep entire artifacts.
4. Ares public archive load exposes identical typed `OrcaStrings`; its existing
   JSON representation preserves the same ordered vector. If config serialization
   is exposed, compare it to actual `ConfigOptionStrings::serialize` as a separate
   observable, not as a replacement for the embedded JSON vector.
5. Slice the fixed embedded project without preset selection/overlay, admitting
   the CLI's existing compatibility gate only as explicitly ruled above; compare
   complete G-code via the repository's unchanged ordered comparator/ND path.
   Inspect the runtime footer's `print_compatible_printers` serialization as a
   keyed Native observable, and observe the existing `--export-3mf` JSON roundtrip.
   Whole G-code equality alone
   cannot prove this metadata key was retained; observations 1–4 precede it.

Do not promise an Ares full 3MF writer API that does not exist. The included
Ares serialization observation is its existing typed settings/config surface;
Native's own writer supplies the upstream roundtrip witness. Adding a new
3MF-export API is out of scope. If the necessary Native typed/read-write
observable cannot be obtained with existing runtime facilities, return a
blocked spec/admission result. No source-kernel probe provisioning or new probe
tools are authorized; seek an owner ruling, not an implicit tooling fallback.
Do not substitute frozen JSON or
post-hoc parsing of the input for Native execution.

## Deferred behavior

- Worker5's nonmatching-machine -17 rejection behavior
  (`OrcaSlicer.cpp:2653–2654`, `Utils.hpp:42`), preset compatibility decisions,
  filtering, machine selection, name existence checks, inherited profile
  resolution and load-settings merge. Passing the admitted fixed-machine gate
  does not transfer ownership or claim decision coverage.
- Lone-empty singleton `[""]` without `M` is deferred with that rejection branch;
  the revised empty-element witness only covers `[M, ""]`.
- JSON scalar legacy vector syntax, nested arrays, non-string/mixed/null
  elements, invalid UTF-8/JSON, malformed/truncated ZIP and archive path rules.
- Arbitrary vector cardinalities/string sizes, controls outside the fixed cases,
  whitespace normalization and Unicode normalization policy.
- Other coStrings keys, cross-key behavior, physical printer/firmware effects,
  object/plate overrides and vendor-specific preset semantics.
- Generic Config string-encoder repairs or production rewrites. A genuine
  discrepancy gets its own source-owner admission before any repair.

## SPEC → admission → RED → GREEN and stop rules

After approval only:

1. Establish read-only Native route/receipts. Required missing Native FAIL;
   optional missing Native visible SKIP. Invalid configured producer, nonzero,
   timeout or missing output is failure, not absence. Prove keyed typed retention
   before comparison. A field erased by CLI/profile logic, unexpected -17 after
   this fixed revision, or absence of keyed footer/roundtrip observability is an
   admission blocker, not permission to enter worker5's scope or invent a probe.
2. Write Native-backed E2E tests BEFORE implementation. Record possible failures
   first: absent/empty conflation, singleton-empty collapse, order/duplicate loss,
   delimiter splitting, quote/backslash/control corruption, UTF-8 corruption,
   key silently replaced, request-only proof, unfiltered comparison not reached.
   Preserve original baseline and full comparator assertions. First admission
   failure is not classification RED.
3. Independently commit classification RED only after all included Native/Ares
   fidelity and applicable complete-output checks pass and the declared bounded
   classification still fails. If a fidelity mismatch exists, STOP; no fabricated
   classification RED and no harness GREEN around it.
4. Harness-only GREEN: key-local finite cases and bounded metadata; minimal
   domains/sweep/summary delegation if needed. All other inventory plans unchanged;
   partial/unproven scope remains INCOMPLETE. No promotion from baseline/null or
   missing Native. Do not delete/reinterpret original assertions.
5. Verify cold/hot official and seeded witness paths, actual keyed application
   and runtime Native JSON/vector provenance, unchanged whole comparator policy,
   selected-key sweep without value filters and the complete latest-tree controls.
   Produce repeatable external read/write/typed vector artifacts and inventories.

Native runs require directed load1<60, 2 nextest threads; full-stage admission
load1<30 and shared heavy/browser queue ownership. Build paired archives in fresh
independent target directories, complete rebuilds and different binary SHAs
verified BEFORE paired execution. No success-seeking retries beyond the existing
bounded comparator path, no input tuning, no geometry repairs or filtering.
Rust tests use `cargo nextest`; touched modules <=400LOC; strict Clippy/fmt/WASM
and workspace gates retained. Raw/Native/cache/receipts remain outside Git.

Complete seeded candidate controls must all PASS; official FAIL set must be a
subset of baseline FAIL. Enumerate all raw Ares changes, distinguish live
Native-derived processor inputs from fixed-input generator changes, retain
all named member-byte SHAs and disclose Native nonexecution separately from
Nextest skips. Main production advances require new-tree validation; purely
web/docs/test carry requires explicit owner authorization. Bundle SPEC/RED/GREEN
for exact-hash combined review before integration. This document authorizes
nothing until approved; no Native runs or Rust implementation are part of the
current spec submission.
