# Bounded timelapse_type option coverage

Status: bounded harness implemented; directed cold/hot, fresh engineering
and seeded/official pairs passed acceptance checks; review pending.
Worker6 owns harness/docs only. Production changes require
separate coordinator authorization. This is bounded coverage of the upstream
Traditional/Smooth enum in a single-nozzle, single-filament, by-layer,
input enable_prime_tower=0 context; it is not full timelapse printer coverage.

## Source boundary

OrcaSlicer v2.4.2 `8500fcdccaa10b5099ac20d252af3a7c560046f1`:

| Upstream source | Contract exercised |
| --- | --- |
| `src/libslic3r/PrintConfig.cpp:5879-5894`, `PrintConfig.hpp:281-284,1627` | `timelapse_type` coEnum has values `0` Traditional and `1` Smooth, default Traditional |
| `src/libslic3r/Print.cpp:1286-1288` | Smooth/by-object validation is an adjacent boundary, deferred here |
| `src/libslic3r/Print.cpp:3290-3303::has_wipe_tower`, `:3317-3319` | Prime-tower activation has its own gate; smooth alone does not activate a disabled tower |
| `src/libslic3r/GCode.cpp:2131-2142` | Smooth without a prime tower sets a warning bit; this block does not reject the job |
| `src/libslic3r/GCode.cpp:4645-4653,4664-4675` | Printer structure, spiral mode, tower and nonempty template govern insertion; non-BBL layer-change insertion renders the template |
| `src/libslic3r/GCode.cpp:5145-5202,5455-5461,5527-5546` | `insert_timelapse_gcode` supplies layer variables; I3 perimeter/infill and layer-end paths determine insertion order |
| `src/libslic3r/PlaceholderParser.cpp:2150-2191::if_else_output` | Template conditionals evaluate the actual enum and render only the selected text branch |
| `src/libslic3r/GCode.cpp:2166-2173`, `GCode/GCodeProcessor.cpp:1866-1999` | Post-emission printable-area checking owns the retained-path rejection |
| `src/OrcaSlicer.cpp:158,6221-6227`, `src/libslic3r/Utils.hpp:74` | Unprintable emitted paths have Native return_code -102 and its exact diagnostic |

Existing Rust counterparts are option registry/process-source wiring and
`ares-core/src/project_slice/gcode_emit/layers/{boundary,timelapse,layer_end}.rs`.
They remain unchanged. The option sweep is a temporary compatibility shell
around upstream configuration, export and G-code emission; this slice improves
its observable upstream context rather than adding an Ares pipeline.

## Current evidence and proposed scope

The [historical full audit](2026-10-09-option-full-current-audit.md) records
smooth as INCOMPLETE because Native commandRnYKWl rejected an emitted path
with -102. This is not an invalid enum value. At main `626ed1738`, the enum
planner already enumerates both values with complete declared scope; the
missing proof is successful runtime activation and full-output comparison.
The offline reconciliation is `/tmp/worker6-option-current-audit-2026-10-10.md`;
it is not a fresh Native run.

Use one centered cube, one object/instance, a non-BBL I3 Ender-3 0.4 mm
context, one filament, by-layer, ordinary non-spiral layers, no support/raft,
`enable_prime_tower=0`, and standard start/end templates. Both requests differ
only in `timelapse_type`. Set the same nonempty `time_lapse_gcode` in each:

```text
{if timelapse_type == 0};W6_TL_TRAD layer={layer_num} z={layer_z} max={max_layer_z}
{else};W6_TL_SMOOTH layer={layer_num} z={layer_z} max={max_layer_z}
{endif}
```

These inert comments witness a real upstream template invocation and enum
branch without introducing camera/purge travel. For each Native export,
verify the requested/effective enum, template, printer structure, single-tool
cardinality, by-layer and disabled tower survive application. Do not silently
substitute another enum, remove the template or enable a tower to obtain PASS.
Native refusal or rewriting of a request is reported before changing scope.

Both Native and Ares must emit the correct nonempty marker branch, exclude
the opposite marker and unexpanded expressions, and agree on ordered marker
locations and resolved layer variables. Compare the complete ordered G-code
bytes and line count with the existing generated-by-only normalization. Do
not infer one insertion per layer: upstream has multiple insertion sites.
Add empty-template controls for both enum values; markers must disappear,
and complete Native/Ares byte comparison still applies. CONFIG_BLOCK equality
alone cannot establish activation.

## Rejection witness and deferred behavior

Retain the historical failed-request identity and raw -102 observation.
Recover that request from its existing external audit artifacts for a fresh
rejection capture; inability to recover it must be reported rather than
substituting an unrelated request and claiming reproduction. Preserve the
existing active `timelapse_extrusion_outside_printable_area_rejects_after_processing`
E2E in `orca_parity/gcode_printable_area.rs`, its tracked input
`tests/parity/validation/timelapse-unprintable-gcode.3mf`, Native retained-output
and exact result.json/validated API diagnostic assertions. This independent
-102 control does not by itself prove parity for commandRnYKWl.

Defer Type2 tower enablement/placement, SEMM, multiple nozzles/filaments,
physical/virtual tool mapping, by-object, BBL camera/purge templates, spiral
mode, warning/viewer flags and physical timelapse quality. Smooth's no-tower
warning is not evidence that these deferred paths are covered.

## RED, GREEN and acceptance

1. RED: use fresh runtime OrcaRunner output to record the existing selected
   enum sweep's admission/comparison result and the successful branch witnesses.
   Keep the actual Native rejection as rejection, not PASS. If main's sweep
   already passes, report the stale classification; do not manufacture RED.
   If new successful-context Ares bytes differ, retain the real RED and stop
   for coordinator review before any production change.
2. GREEN: key-specific harness context/dispatch in
   `crates/ares-cli/tests/orca_parity/option_coverage`, with a dedicated timelapse
   module and minimal shared registration. Preserve the enum planner, both
   values, comparator, rejection controls and every other domain's plan.
   Canonical success accounting must show both values fully compared; empty
   controls and rejection receipts remain separately identified.
3. Required seeded and official directed runs, cold/hot cache, then fresh
   base/candidate seeded and official pairs under coordinator scheduling.
   Verify all unrelated option plans unchanged and list every status/raw-byte
   change. Finish default-environment workspace, strict clippy, fmt and wasm
   checks before REVIEW-REQUEST; no production changes are authorized here.

Reuse existing OrcaRunner export/slice cache and shared dependency policy:
required missing Native fails; optional missing Native visibly skips. Commit
only inputs and source. Actual exports, Native G-code, stdout/stderr, hashes,
result.json and comparison receipts stay under external artifact/cache roots.
No new runner/cache/API, frozen fallback, ignored test, probe, tolerance or
special production guard is part of this slice.

## Directed implementation evidence (not full acceptance)

Spec `dee0ba2c4`, key-local RED `0cadabfb8`, harness GREEN `de45c653a`.
On production main `626ed1738`, the RED asserted PASS but received INCOMPLETE:
Traditional full-output PASS, Smooth Native Unix154 / return_code -102.
The rejected main context had enable_prime_tower=1, empty time_lapse_gcode,
by-layer, bed0..220 and wipe_tower_y220. It is not the spec's disabled-tower
input. Print.cpp:1286-1288 does not apply; Print.cpp:3385-3388/3290-3303
activates its single-filament smooth tower. Retained G-code's Skirt line79
(Y224.646) and Prime tower line666 (Y222.328) exceed the expanded bed maxY222
in GCodeProcessor.cpp:1922-1948; OrcaSlicer.cpp:6221-6227 emits the exact -102.
Source/input/raw SHA audit: /tmp/worker6-timelapse-minus102-source-audit.json.

The coordinator authorized the distinct approved spec input while retaining
that rejection. No bed/tower coordinates or production source changed.
Fresh required seeded cold directed3/3 PASS, official cold4/4 PASS, seeded
warm4/4 PASS and official warm4/4 PASS. The4-test selections also run the
existing public API rejection
E2E. Both active enum templates emit100 ordered markers in these actual
outputs; empty controls emit0. Every success compares complete output bytes,
with only generated-by normalization. Counts are observed, not generalized
to a one-insertion-per-layer requirement. The original-context runtime
rejection control reproduces -102, and its entire ZIP member byte/hash map is
identical to RED, not a replacement negative fixture.

Logs: /tmp/worker6-timelapse-green-{seeded,official,seeded-warm,official-warm}.log.
External proof roots use the same prefixes with /artifacts; marker/comparison
receipts are timelapse-type-bounded.json. Original-rejection-input-audit.json
under /tmp/worker6-timelapse-green-seeded proves the unchanged negative input.
Plan-source-invariance-audit.json there records unchanged domain-generation
source, not fresh serialized plan equality; directed key-local tests do not
write the full plan snapshot. The fresh full-pair check below supplies the
serialized plan comparison.
The historical commandRnYKWl receipt remains historical; fresh replay proves
the newly captured RED input without claiming recovery of that older capture.

## Fresh acceptance on production73e78279d

Rebased spec `2e2cdba3b`, RED `80e04502e`, GREEN `aae55037e`;
tested candidate `0b4bf0115`, base `73e78279d`. No production source changed.
Earlier production626 gates and interrupted d4 engineering are historical
only and are not used as merge evidence.

Fresh default-environment workspace:8291/8291 PASS,5 existing nextest skips,
11 SLOW cases ultimately PASS, no FAIL/TIMEOUT. Strict workspace/all-targets/
all-features clippy with -D warnings, fmt and core/wasm32 checks each exit0.
Logs and exit/process receipts: /tmp/worker6-timelapse-engineering-73e/
{workspace-default,clippy,fmt,wasm}.log. Workspace environment removes every
ARES_/ORCA_ variable; jobs2 and test-threads2.

| Fresh leg | Result | Original log under /tmp/worker6-timelapse-pairs-73e/ |
| --- | --- | --- |
| seeded base |918/919 PASS,1 FAIL,0 skip,exit100 |seeded-base.log |
| seeded candidate |922/922 PASS,0 FAIL,0 skip,exit0 |seeded-cand.log |
| official base |918/919 PASS,1 FAIL,0 skip,exit100 |official-base.log |
| official candidate |922/922 PASS,0 FAIL,0 skip,exit0 |official-cand.log |

Both modes' sole status change among existing tests is
option_coverage::sweep::option_coverage FAIL→PASS. Base's selected
timelapse_type domain is INCOMPLETE (2 generated,1 compared,1 rejected);
candidate is PASS (2 generated,2 compared,0 rejected). Base runs its own test
tree with explicit ARES_PARITY_OPTIONS=1/ARES_OPTION_KEYS=timelapse_type,
not transplanted candidate tests. The base failure is the preserved original
Smooth -102 rejection above, not Native nondeterminism. Each leg executes
the full orca_parity test set; only its option sweep is key-filtered.

Three added tests PASS in both modes: the complete-domain assertion, the
conditional-template/empty controls, and the original Smooth -102 control.
The two candidate failure sets are empty subsets of their respective base
failure sets; both strict audit scripts exit0. No tolerance, comparator or
failure filtering changed, and no Native retry was needed.

The serialized650 option plans are identical between base and candidate in
both modes, including both timelapse enum values. Common stream identity
uses label plus all input ZIP member byte hashes:seeded623 and official612
common keys have zero Ares raw SHA changes, zero generator-normalized Native
SHA changes and zero manifest status changes. Capture counts are retained;
SHA comparison uses distinct sets. Only timelapse inputs differ, as required
by the approved bounded context; added template controls are separate keys.
This is not a fresh execution of every request in all650 domains.

Audit receipts: /tmp/worker6-timelapse-pairs-73e/
{seeded-comparison,official-comparison,plan-diff,seeded-base-timelapse-audit}.json.
All Native outputs, receipts and logs remain outside the repository.
