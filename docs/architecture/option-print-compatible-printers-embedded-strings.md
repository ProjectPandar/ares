# SPEC / ARD: bounded embedded print_compatible_printers coStrings fidelity

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
  option-coverage harness. No production change is part of this spec.

The Option harness is a temporary inventory/execution compatibility shell
around the above upstream structures. Its baseline-only `value:null` is not a
source-backed proof of explicit string injection, and its generic omission
wording is not authority to declare all strings complete. Existing Ares typed
storage/serialization is reused, not redesigned or treated as an Ares pipeline.

## Scope fences — do not cross

Preset compatibility filtering, process/machine compatibility
selection and `--load-settings` overlay/compatibility decisions, including the
adjacent `OrcaSlicer.cpp` current/new compatibility-list logic and replacement
of embedded fields. None is included, even if Native CLI machinery happens to
visit those functions. Do not modify those functions, harness hooks or their
fixtures. Do not prove filtering by asking whether a listed name exists or a
printer is permitted.

Also excluded: printable_height/G2 clearance; wrapping geometry; G4 IDs/mapping;
load-settings areas. Retain all existing bridge/filament/G2/APA dispatches.

The existing Native CLI compatibility gate may execute as an admission
precondition only. The current machine's
`printer_settings_id` is a fixed element of every nonempty witness vector;
compatibility PASS is not a fidelity assertion or a new feature.
No substitution of `compatible_printers`, profile edits, load-settings overlay,
filtering implementation or compatibility validation assertions. Unexpected
field replacement or rejection after this fixed revision still means STOP; do
not iteratively tune names or reorder/drop difficult elements.

## Initial admission and witness revision

The first short `--slice 0` invocation used
`["Embedded Printer B", "Embedded Printer A", "Embedded Printer B"]` with no
current machine name. Native returned **-17**, shell exit239; its `result.json`
says `The selected printer is not compatible with the process preset in the 3mf.`
No G-code was generated. Sources: `OrcaSlicer.cpp:2653–2654` reports/exits
`CLI_PROCESS_NOT_COMPATIBLE`; `libslic3r/Utils.hpp:42` defines that code as -17.
This is genuine upstream behavior, not missing Native or classification RED.

The seven fixed witnesses are therefore revised so all nonempty vectors contain
the current machine's `printer_settings_id`, preserving the other
order/duplicate/escaped/Unicode elements. Absent and `[]` stay unchanged. The
nonmatching-machine -17 rejection slice belongs with load-settings and is
**deferred here**. This is a deliberate new matrix, not a success-seeking retry
of the failed sample.

With the revised matrix, both `--slice 0` and `--export-3mf` exit 0. The footer
carries the exact serialized M/B/A/B sequence; the actual roundtrip
`Metadata/project_settings.config` vector equals the requested ordered four
elements. This proves these existing channels exist, not that the remaining
cases or classification already passed.

Direct `OrcaStrings` Serde expects a JSON sequence, not the Native footer's scalar
serialization token, so the footer cannot be decoded through it. The footer is compared as serialized bytes in the unchanged complete-stream
comparison, while Native writer JSON (`ConfigOptionStrings::vserialize`) is
the independent vector/cardinality witness. In particular `Config.cpp:72–120`
does not quote semicolons in otherwise-simple elements, so parsing its footer
as a reversible vector is not an upstream guarantee. Keep the semicolon sample;
do not change the serializer or assert invented roundtrip-token semantics.

## Native-backed classification RED

The retained baseline plus **all seven revised cases** pass public Ares
typed/Serde checks, live Native footer presence, actual Native `--export-3mf`
vector JSON equality and complete ordered G-code comparison. Only the final
bounded classification assertion fails: INCOMPLETE versus PASS (baseline plan
still contains noninjected null). That is classification RED, not an adapter
failure or -17.

## Harness GREEN

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
classification and selected-key unfiltered-value sweep. Warm runs share only the original-project export cache; the footer slice and
Native 3MF roundtrip commands are still live/fresh per witness. Do not claim
these are cached Native fidelity results. The key-local module remains under
400LOC.

## Inventory evidence and honest coverage meaning

In the option-domain plan inventory this key is `coStrings`, source `PrintConfig.cpp:1832`, one baseline case
with `value:null`, `declared_scope_complete:false`, omission
`unbounded domain (string/point/blob) — baseline only`.
`docs/architecture/2026-10-09-option-full-current-audit.md` also records it as
INCOMPLETE. Later completions are not inferred from this older snapshot.

Success can establish only the finite **embedded string-list fidelity scope**
below. It does not close the unbounded string universe, arbitrary vector
lengths, catalog filtering, validation of names, or all legacy encodings. Any
PASS metadata must name this scope and its omissions, not claim global schema
completion or printer compatibility. The historical baseline assertion stays;
an absent/default case cannot substitute for explicit cases.

## Fixed witness matrix (declare before running)

Use the established Native-produced classic `cube10.stl` single-logical-filament,
no-prime-tower project (adaptive-pa/no-cooling). Keep its fixed process
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

Only `M` is a real fixture machine identifier.
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
tools; no implicit tooling fallback. Do not substitute frozen JSON or
post-hoc parsing of the input for Native execution.

## Deferred behavior

- The nonmatching-machine -17 rejection behavior
  (`OrcaSlicer.cpp:2653–2654`, `Utils.hpp:42`), preset compatibility decisions,
  filtering, machine selection, name existence checks, inherited profile
  resolution and load-settings merge. Passing the admitted fixed-machine gate
  does not claim decision coverage.
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

1. Establish read-only Native route/receipts. Required missing Native FAIL;
   optional missing Native visible SKIP. Invalid configured producer, nonzero,
   timeout or missing output is failure, not absence. Prove keyed typed retention
   before comparison. A field erased by CLI/profile logic, unexpected -17 after
   this fixed revision, or absence of keyed footer/roundtrip observability is an
   admission blocker, not permission to enter load-settings scope or invent a probe.
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

No success-seeking retries beyond the existing
bounded comparator path, no input tuning, no geometry repairs or filtering.
Rust tests use `cargo nextest`; touched modules <=400LOC; strict Clippy/fmt/WASM
and workspace gates retained. Raw/Native/cache/receipts remain outside Git.

Complete seeded candidate controls must all PASS; official FAIL set must be a
subset of baseline FAIL. Enumerate all raw Ares changes, distinguish live
Native-derived processor inputs from fixed-input generator changes, retain
all named member-byte SHAs and disclose Native nonexecution separately from
Nextest skips.
