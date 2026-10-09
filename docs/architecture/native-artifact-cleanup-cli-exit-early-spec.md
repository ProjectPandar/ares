# B CLI-exit six early rejection families — runtime Native contract

Status: proposed next bounded B slice; implementation waits for approval.
Source owner: upstream CLI adapter around libslic3r validation/model/Print APIs,
not a new Ares pipeline. Production Rust/API behavior is unchanged.

## Selected boundary

The six early rejection fixtures are consumed by
`crates/ares-cli/tests/cli_exit_channels.rs`, NOT cli_retained_exit.rs. The latter
consumes only `path-outside.*` and the O6 -102 input and is deferred as requested.
This slice closes all six existing early-channel cases without touching retained
emission, O6 -102, marker or worker6-owned KSR/replay/processor adapters.

Upstream citations:
- libslic3r/PrintConfig.cpp:8866 DynamicPrintConfig::validate(bool under_cli),
  CLI application/validation reporting OrcaSlicer.cpp:3648-3653.
- libslic3r/Print.cpp:1262 Print::validate; CLI applied config metadata and
  validate dispatch OrcaSlicer.cpp:5981-5987,6062-6096.
- CLI model/plate selection refusal OrcaSlicer.cpp:5723-5724,6103-6104,
  CLI_NO_SUITABLE_OBJECTS; no new model interpretation in Rust.
- Existing project import rejects post_process at OrcaSlicer.cpp:1688-1689;
  this is CLI_POSTPROCESS_NOT_SUPPORTED, NOT execution of a postprocess program.
- Missing input file OrcaSlicer.cpp:1608 (No such file), Utils.hpp:27(-3).
- Utils.hpp:43-44(-18/-19),52-53(-50/-51), CLI generic messages.cpp:115,130-138.
- OrcaSlicer.cpp:398-411 flush_and_exit stdout/POSIX exit and :416-460
  record_exit_reson JSON. :436-437 elapsed prepare/export integer fields reuse
  the already-approved shared narrow wall-clock byte comparator.

Rust destination is existing public Ares CLI invocation and existing OrcaRunner
CLI-result test cache. Current committed Native snapshots are temporary test
scaffolding only; replace their consumption, not underlying core error behavior.

## Exact cases / expected outcome

| Test (same name after relocation) | Input | Native signed / Linux exit |
| --- | --- | --- |
| cli_exit_values_family_matches_native | values.3mf | -18 /238 |
| cli_exit_o3_range_family_matches_native | range.3mf | -18 /238 |
| cli_exit_empty_plate_family_matches_native | empty-plate.3mf | -50 /206 |
| cli_exit_validate_family_matches_native | validate.3mf | -51 /205 |
| cli_exit_postprocess_family_matches_native | postprocess.3mf | -19 /237 |
| cli_exit_notfound_family_matches_native | absent /tmp/definitely-missing.3mf | -3 /253 |

All present3mf inputs remain byte-identical. The missing path remains exactly
that literal and must remain absent; never create a stand-in empty input or
canonicalize an absent path via fallback. Existing stdout-derived return-code
assertion, precise channels and whole result JSON byte comparisons are retained.
Applied layer-height f32 text, density/wall count, generic error string, plate
index and whitespace/order remain exact. Only the previously approved shared
native_wall_clock::result_comparison_bytes masks the two integer wall-clock
literals, not fields/key order/other digits. Raw receipts and cold/hot comparisons
are unmodified. No stdout/stderr normalization or regex mismatch suppression.

## Writable scope / exact deletion inventory

- Move six cases from crates/ares-cli/tests/cli_exit_channels.rs to
  crates/ares-cli/tests/orca_parity/cli_exit_channels.rs.
- Root orca_parity.rs only new module registration.
- Extend existing runner/cache/custom_gcode.rs CLI-result recipe implementation;
  if naming/splitting needed, rename to runner/cache/cli_result.rs and adjust
  ONLY that module registration in runner/cache.rs. No duplicate cache/producer,
  alternate prewarm or uncontrolled generic allowed-nonzero command helper.
  Keep files below400LOC by minimal bounded recipe/input split if necessary.
- Existing native_wall_clock.rs consumed unchanged; no copied comparator.
- Delete18 frozen Native products: under tests/parity/validation/cli-exit,
  {values,range,empty-plate,validate,postprocess,notfound} x
  {stdout,stderr,result.json}, only AFTER real runtime GREEN.
- Architecture spec/conclusion update only.

Do not delete/change the five3mf inputs. Do not touch custom-gcode input assets,
S4 inputs, path-outside.{native.gcode,result.json,stdout,stderr}, cli_retained_exit,
core o6_print_volume, marker, KSR/replay assets/tests, .config or any production
Rust/tool recipe. worker6 plate_selection and its cache.rs registration remain
exclusive; same shared file changes only own CLI-result registration.

## Cache and missing-path protocol

Use the existing native-cache-v2/producer.digest/cli-result namespace, identity,
locks, staging and atomic publish, request/output SHA validation. Add only typed
bounded early rejection kinds (-3/-18/-19/-50/-51), not arbitrary nonzero support.
Native --slice0 outputdir must produce only result.json for these early cases;
verify exact platform status and payload return_code, propagate any Spawn/Wait/
Timeout/IO/other process failure. Global command/stages semantics remain unchanged.

Existing-file identity remains input SHA/name plus finite command recipe. The
missing-file request has a distinct declared-absent input kind: identity binds
its exact absolute path and absence. A hit cannot hide mandatory Native producer
availability or a path that has since appeared; check Native availability and
missing-input condition before consulting a stored -3 result. No fake file SHA,
empty-file substitution, canonicalize fallback or legacy golden path.

Every case asserts immediate hot reference path equals cold path and captured
stdout/stderr/full rawJSON equality. Retain actual Native/Ares stdout/stderr,
fullJSON/status/receipt/command evidence as repeatable runtime artifacts. Cache
requests bind producer/native/resources/replay identity already supplied by
coordinator controls. Required mode missing deps fails named before hit; optional
None prints existing visibleSKIPPED and returns without any frozen comparison.

## Test-first / honest RED

Before migration, add an independent child regression selecting the exact
existing cli_exit_values_family_matches_native (or notfound) test with required1
and NativeBIN unset. Old frozen consumer currently ignores producer availability
and canPASS; parent expects childFAIL with named Native declaration, generating
an actual runtime RED. Assert running1 selector so disappearing test cannot pass.
Record original commit/log independently; do not count compile errors as RED.

Then relocate cases, use live bounded recipe, update child selector without
changing failure expectation, run all six + policy with real complete env and
cold cache. Delete18 outputs only after GREEN, then repeat cold/hot in a new
cache to prove no dependency on deleted Native output/receipts. Retain deliberate
absent-file path assertion and all original channels/JSON/exit comparisons.

## Gates and artifact acceptance

- seeded and official targeted six+policy, shared wall-clock contracts unchanged;
  six cases+one policy=7, plus the two shared contracts=9 when explicitly selected.
  Count actual names/results rather than treating a skipped Native call as parity.
- Existing full seeded/official baseline/candidate controls, exact named old/new
  results and common raw comparison; Native ND gets exactinput/raw/receipt proof,
  not failure-count inference or relaxed assertions.
- Candidate required workspace, clippy workspace/alltargets/allfeatures-Dwarnings,
  actualCLIwasm-Dwarnings and fmt under coordinator oneheavy/16thread scheduling;
  deterministic/main carry only with explicit coordinator ruling and actualhash.
- Explicit PrintStatistics and three KSR producer vars from current v2 controls
  when using a private wrapper; do not change worker6 KSR code to satisfy setup.
- Exact-hash REVIEW, approval, newest-main rebase, ff-only push.

Deferred: two cli_retained_exit -102 controls/full exported stream + selected
nonminimum plate; corresponding four path-outside products and core O6 partial
stream; marker source-kernel/browser adapter and last authorized Native build.
No new Native build required by this early-error slice. No milestone/plan added.
