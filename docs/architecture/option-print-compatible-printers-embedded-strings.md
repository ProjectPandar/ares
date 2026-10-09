# SPEC / ARD: bounded embedded print_compatible_printers coStrings fidelity

Status: **SPEC ONLY — approval requested before RED or implementation.**
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

If an observable Native route requires entering compatibility decisions or
changes this field through process loading/overlay, **STOP and ask the owner**.
No substitution of `compatible_printers`, no profile edits, no adding/removing
names to pass admission, and no new compatibility rules to rescue this spec.

## Inventory evidence and honest coverage meaning

`/tmp/ares-apa-g1-bridges/other-plan-inventory-audit.json` is an invariance audit,
not the full inventory: it records only the previous APA key delta. Its full
associated plan file is
`/tmp/ares-apa-g1-bridges/green-first-official/artifacts/option-domain-plans.json`.
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

Use one established classic `cube10.stl` project with a single logical filament,
no prime tower, default layer/process settings. Start from a Native-produced
3MF and inject only this key in `Metadata/project_settings.config` for each
case. No `--load-settings` or `--load-filaments` during witness replay. Keep all
other named archive members byte-exact to the fixed initial project. Retain
raw ZIP/member SHA receipts; changing this JSON key is deliberate, not a claim
of whole-archive equality across cases.

| Case | Embedded JSON value | Fidelity assertion |
| --- | --- | --- |
| missing-default | key absent | Existing typed default is an empty vector; do not claim absent raw key equals explicitly present empty key |
| explicit-empty | `[]` | Zero elements; not one empty string |
| lone-empty | `[""]` | One empty element; distinct from `[]` |
| one-label | `["Embedded Printer A"]` | Single string with whitespace preserved |
| ordered-duplicates | `["Embedded Printer B", "Embedded Printer A", "Embedded Printer B"]` | Preserve order, duplicates and length; no deduplication or catalog lookup |
| escaped-elements | `["semi;colon", "quote\"slash\\", "line\nnext", "tab\tvalue"]` | JSON string characters/element boundaries preserved through typed read and vector JSON serialization |
| unicode-label | `["打印机 Ω"]` | Preserve UTF-8 text; no normalization or name validation |

These are opaque configuration strings, not real preset names and not user
input to a compatibility predicate. No assertion says that these printers
exist or are accepted. The seven cases are fixed before admission; failures
must not lead to value/count/escaping tuning or dropping a difficult case.

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
5. Where the existing Native route can slice the untouched embedded project
   without compatibility selection/overlay, compare complete G-code via the
   repository's unchanged ordered comparator/ND path. Whole G-code equality alone
   cannot prove this metadata key was retained; observations 1–4 precede it.

Do not promise an Ares full 3MF writer API that does not exist. The included
Ares serialization observation is its existing typed settings/config surface;
Native's own writer supplies the upstream roundtrip witness. Adding a new
3MF-export API is out of scope. If the necessary Native typed/read-write
observable cannot be obtained with existing runtime facilities, return a
blocked spec/admission result and seek approval for a source-kernel probe
before provisioning or extending tools. Do not substitute frozen JSON or
post-hoc parsing of the input for Native execution.

## Deferred behavior

- Any preset compatibility decision, filtering, machine selection, name
  existence check, inherited profile resolution or load-settings merge.
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
   before comparison. A field erased by CLI/profile logic is an admission blocker,
   not permission to enter worker5's scope.
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
