# Bounded timelapse_type option coverage

This is bounded harness coverage of the upstream
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
with -102. This is not an invalid enum value. The enum planner already
enumerates both values with complete declared scope; the missing proof is
successful runtime activation and full-output comparison.

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

At v2.5.0-beta `345efec64` those sites are exclusive and each layer renders once (see
[the hook specification](timelapse-layer-change-gate-spec.md)). The 100 markers recorded
below are a v2.4.2 observation of two renders per layer.

Add empty-template controls for both enum values; markers must disappear,
and complete Native/Ares byte comparison still applies. CONFIG_BLOCK equality
alone cannot establish activation.

## Rejection witness and deferred behavior

The -102 observation for commandRnYKWl is a v2.4.2 record. At v2.5.0-beta
`345efec64` a plain slice moves a stored tower that does not fit before slicing
anything (`OrcaSlicer.cpp:6480-6509`; see
[the tower fit specification](slice-wipe-tower-fit-spec.md)), so Native slices
that context successfully instead of rejecting it. An off-bed stored tower is no
longer a -102 witness.

`orca_parity/gcode_printable_area.rs` holds both sides on the tracked input
`tests/parity/validation/timelapse-unprintable-gcode.3mf`:
`off_bed_stored_tower_moves_before_slicing_like_native` compares the moved tower
on complete output, and `skirt_past_the_bed_edge_rejects_after_processing` derives
an input at runtime that still reaches -102, with Native's result confirmed live
and the retained G-code compared in full.

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
   before any production change.
2. GREEN: key-specific harness context/dispatch in
   `crates/ares-cli/tests/orca_parity/option_coverage`, with a dedicated timelapse
   module and minimal shared registration. Preserve the enum planner, both
   values, comparator, rejection controls and every other domain's plan.
   Canonical success accounting must show both values fully compared; empty
   controls and rejection receipts remain separately identified.
3. Required seeded and official directed runs, cold/hot cache, then fresh
   base/candidate seeded and official pairs. Verify all unrelated option plans
   unchanged and list every status/raw-byte change. Default-environment
   workspace, strict clippy, fmt and wasm checks pass; no production changes
   are part of this slice.

Reuse existing OrcaRunner export/slice cache and shared dependency policy:
required missing Native fails; optional missing Native visibly skips. Commit
only inputs and source. Actual exports, Native G-code, stdout/stderr, hashes,
result.json and comparison receipts stay under external artifact/cache roots.
No new runner/cache/API, frozen fallback, ignored test, probe, tolerance or
special production guard is part of this slice.

## Smooth -102 rejection analysis (v2.4.2)

This analysis describes the v2.4.2 source. The rejection it explains no longer
occurs at v2.5.0-beta `345efec64`; see "Rejection witness and deferred behavior".

The RED asserted PASS but received INCOMPLETE:
Traditional full-output PASS, Smooth Native Unix154 / return_code -102.
The rejected main context had enable_prime_tower=1, empty time_lapse_gcode,
by-layer, bed0..220 and wipe_tower_y220. It is not the spec's disabled-tower
input. Print.cpp:1286-1288 does not apply; Print.cpp:3385-3388/3290-3303
activates its single-filament smooth tower. Retained G-code's Skirt line79
(Y224.646) and Prime tower line666 (Y222.328) exceed the expanded bed maxY222
in GCodeProcessor.cpp:1922-1948; OrcaSlicer.cpp:6221-6227 emits the exact -102.

The coverage therefore uses the distinct disabled-tower spec input. No bed/tower coordinates or production
source changed. Both active enum templates emit 100 ordered markers in the
actual outputs; empty controls emit 0. Every success compares complete output
bytes, with only generated-by normalization. Counts are observed, not
generalized to a one-insertion-per-layer requirement.

Two tests cover the slice: the complete-domain assertion and the
conditional-template/empty controls.
The selected `timelapse_type` domain moves from INCOMPLETE (2 generated,
1 compared, 1 rejected) to PASS (2 generated, 2 compared, 0 rejected). No
tolerance, comparator or failure filtering changed; all other option plans,
including both timelapse enum values, are unchanged.
