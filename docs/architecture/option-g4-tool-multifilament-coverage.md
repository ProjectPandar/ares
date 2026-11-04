# G4: bounded tool and multi-filament option coverage

Only the four coherent Manual `filament_map`
cases are GREEN. The other five keys remain INCOMPLETE and unexecuted.

## Bounded conclusion

The Rust destination remains the source-cited `ToolOrdering` physical-map
consumers and `GCode` traditional timelapse/object-label/template lifecycle,
not an Ares-owned tool pipeline. Two complete Standard selectors, two
filaments, wall1/infill2 and four Manual maps establish this finite scope only.
The original singleton-selector input remains preserved as O8 UB_ADJACENT.
Automatic grouping, arbitrary vector/project shapes, by-object mask production
and the other five G4 contracts are not promoted by this result.

The Native-only nondeterminism adjudication is
recorded in `g4-manual-map-review.md`; raw failures remain visible and the
whole-output comparator is unchanged.
This slice covers the six keys below. It is a harness
admission slice around upstream configuration consumers, not a change to
production ToolOrdering, WipeTower, GCode or CLI normalization.

## Source boundary

Pinned OrcaSlicer v2.4.2 `8500fcdccaa10b5099ac20d252af3a7c560046f1`.
All paths below are under `src/`.

| Key and definition | Owning consumer and included contract |
| --- | --- |
| `filament_map`, `libslic3r/PrintConfig.cpp:2489-2493`, coInts, default1 | `PrintConfig.cpp:88-89`; `GCode/ToolOrdering.cpp:1275-1312`; `Print.cpp:3282-3284,3443-3448`; `GCode.hpp:103-107`; `GCode.cpp:3579-3633`: configuration maps logical filament IDs to one-based nozzle IDs; consumers subtract1. Manual plate mapping must survive, select nozzle-dependent values and remain visible in complete output. |
| `filament_self_index`, `PrintConfig.cpp:5432-5438`, coInts, default1, nocli | `OrcaSlicer.cpp:3121-3145,3327-3335`; `PrintApply.cpp:1172`; `PrintConfig.cpp:9805-9908`: grouped one-based filament identity plus `filament_extruder_variant` selects a per-filament variant payload. CLI preset loading may reconstruct identity; a project-layout request is not an arbitrary CLI override. |
| `master_extruder_id`, `PrintConfig.cpp:5399-5404`, coInt, default1 | `GCode/ToolOrdering.cpp:1136-1201`; `FilamentGroup.cpp:40-47`; `GCode.hpp:103-107`: one-based master becomes zero-based initial/grouping preference; wipe integration initializes master offsets before explicit map offsets. Include finite two-nozzle master1/2 controls; do not claim every automatic grouping strategy. |
| `nozzle_flush_dataset`, `PrintConfig.cpp:2602-2604`, nullable coInts, default0 | `OrcaSlicer.cpp:3436-3488`; `FlushVolPredictor.cpp:312-329`; `FlushVolCalc.cpp:58-132`: printer variant selection resolves a dataset per nozzle before reconstructing missing flush matrices; dataset0/1/2 chooses standard/dual-standard/dual-highflow resources. A pre-supplied matrix does not prove this consumption. |
| `other_layers_print_sequence_nums`, `PrintConfig.cpp:1121-1123`, coInt, default0 | `ParameterUtils.cpp:6-29`; `GCode/ToolOrdering.cpp:1315-1362`: count partitions an equally sized flattened vector into inclusive one-based layer ranges plus filament order. Last matching range wins. Count0 or an empty sequence is inactive. Invalid count/vector shapes can assert and are not legal endpoint probes. |
| `filament_ramming_parameters`, `PrintConfig.cpp:2833-2838`, coStrings | `GCode/WipeTower.cpp:1591-1602` disables parsing with `#if 0`; `WipeTower2.cpp:1379-1389,1643-1713,2228-2232` parses two percent multipliers followed by 0.25s speed segments. `Print.cpp:3397-3399,3581-3601` selects Type2. `WipeTower2.cpp:1250,1657` additionally requires SEMM and enabled ramming for this string to drive motion. |

Schema defaults or inherited numeric maxima are not physical legal endpoints.
No vector may name nozzle0/out-of-range, invent unsupported variants, or feed
malformed layer ranges to the assertions. Nullable dataset sentinel execution
is deferred: the schema flag alone does not prove a safe selector consumption.

## Rust destination and current gap

The current audit records all six as INCOMPLETE, baseline-only, with no
injected value. Existing production owners remain unchanged:
`options/flush_matrix.rs`, `project/domain/profile_input/overlay.rs`,
`project/effective_config`, `project_slice/gcode_emit/tool_ordering`,
`gcode_emit/wipe_tower`, and configuration exports. These are temporary Rust
rewrite boundaries for the cited upstream concepts, not a new G4 pipeline.

New logic belongs in `crates/ares-cli/tests/orca_parity/option_coverage/`:
`tool_domains.rs` for finite plans/context metadata,
`tool_context.rs` for key-local input preparation/verification, and dedicated
`tool_mapping.rs`, `tool_sequences.rs`, `tool_flush_ramming.rs` E2Es.
Split further only if a file exceeds400 lines. `domains.rs`, `activation.rs`,
`sweep.rs`, `option_coverage.rs` receive only the dispatch/module wiring needed
to use these owners. Do not move or rewrite G2, G3 or the concurrent flow work.
Reuse OrcaRunner export/slice/cache, application proofs, exact ordered stream
comparison, shared Native environment and existing ZIP input binding patterns.
No new production entry point, generic context framework or Native build.

Native export can overwrite internal layout, map and plate sequence values.
Record pre-bind requested/exported values; then bind only the cited owner in
a new input ZIP and submit that identical input to both slicers. Never mutate
shared cache entries or change Native output. Project settings, plate metadata
and Ares effective values must agree where both owners apply. Silent Native
rewrites remain application mismatches, not successful injected coverage.

## Finite contexts and requests

All successful requests are explicit nonbaseline cases. The plan metadata
names the context and deferred behavior. PASS means only this finite scope;
it cannot mean every mapping, string, variant or grouping combination.

### Mapping and master controls

Use existing BBL H2D/X2D two-nozzle profile/export mechanics, both nozzles0.4mm,
relative extrusion, two distinguishable PLA filaments, no support/raft,
layer0.2mm. Set wall filament1, sparse/solid filament2 and nonzero sparse fill
so both logical filaments actually extrude on multiple layers of cube10.
Verify these dependencies and actual per-layer tool usage. Bind Manual mode
at project and plate, not just its serialized project default.

- `filament_map`: `[1,1]`, `[1,2]`, `[2,1]`, `[2,2]`. Distinct nonzero
  temperature/offset payloads witness mapped nozzle selection; compare full
  output and record used-tool/nozzle assignment, not merely a config comment.
- `master_extruder_id`:1 and2, with coherent nozzle/offset vectors and the
  same two-filament geometry. Manual `[1,2]` is an explicit override/control,
  not proof of automatic preference. Add an Auto For Match control per master
  at the plate owner; verify the resulting Native assignment and grouping
  inputs. Require a source-owned observable preference witness before this
  key's bounded classification becomes PASS. If both automatic requests have
  identical behavior beyond serialized value, report that limitation and keep
  the key INCOMPLETE; do not invent a grouping result or shrink the contract.

### Filament variant identity

Use the same two-filament printer layout. Author coherent project variants
post export instead of passing this nocli key as an ordinary override:

- identity `[1,2]` with one supported variant for each logical filament;
- identity `[1,1,2,2]` with Standard/High Flow variants grouped per filament,
  corresponding four-entry temperature payloads211/221/233/243, coherent
  printer variant layout and Manual `[1,2]` mapping;
- retain the grouped identity and reverse mapping `[2,1]` as a variant-selection
  control. Bind the supported nozzle volume types so the two variants differ.

Verify the original grouped index, variant strings and payload lengths in the
input and the mapped temperature selection after apply. Record any CLI
reconstruction separately from the pre-apply index; do not expect the flattened
value to equal an unconsumed arbitrary vector. Both actual filament paths and
complete G-code must agree. Missing suitable upstream variant/profile context
blocks admission; it is not permission to fabricate a fallback variant.

### Other-layer sequence count

Use Ender-3, one0.4mm nozzle, two PLA logical filaments, SEMM, Manual `[1,1]`,
relative extrusion, wall1/sparse+solid2, layer0.2mm, no support/raft.
Use one coherent first-layer order and activate other-layer Customize at the
plate owner. Flat item shape is `[start_layer,end_layer,filament1,filament2]`.

- count0, vector `[2,5,2,1]`: explicit inactive count control;
- count1, vector `[2,5,2,1]`: reverse only layers2 through5;
- count2, vector `[2,5,1,2,4,6,2,1]`: overlapping ranges, last item wins
  on layers4 and5 and extends through6.

Verify both count and vector in project/plate effective configuration. Native
per-layer ordered extrusion/tool-change evidence must distinguish the active
ranges, overlap priority and unaffected first/outside layers. Full ordered
output remains the acceptance oracle. No negative/random count, partial item,
out-of-range filament or empty-range assertion probes are included.

### Flush dataset

Use two-nozzle profile and two colored filaments above, coherent explicit
printer variant IDs/strings, per-nozzle flush multipliers1 and minimum-flush
inputs. Remove `flush_volumes_matrix` from the bound input to activate CLI
reconstruction; retain a separate explicit-matrix control demonstrating that
this path is not the effective owner when a complete matrix is supplied.

Inject `[0,0]`, `[1,1]`, `[2,2]`, `[1,2]`. For each request verify mapped dataset
values, input matrix absence, resulting Native per-nozzle matrix dimensions,
diagonal zeros and reconstructed off-diagonal values. Verify Ares effective
matrix and strict complete output. The mixed request must distinguish segment
selection, not only retention of a dataset comment. Keep min/max flush and
color dependencies in proof artifacts. Resource-backed results come from the
runtime producer; resource absence is a blocker, not a model-free success.
Unsupported selectors, nil selectors and arbitrary variant cardinalities are
deferred, not certified by this three-resource finite set.

### Ramming strings

Use the two-logical-filament one-nozzle SEMM context above, tower enabled,
Type2, `enable_filament_ramming=1`, purge-in-tower enabled, coherent cooling /
loading settings and both tools genuinely changing. Author per-slot vectors:

- both slots `120 100 6.6 6.8`;
- both slots `140 110 7.2 7.6`;
- slot1 `120 100 6.6 6.8`, slot2 `140 110 7.2 7.6`.

Verify requested strings after export/binding and effective slot selection,
real ramming extrusion and changes in Native tower moves/used extrusion between
speed requests. Every successful comparison includes the complete ordered
file. Add Type1 or `enable_filament_ramming=0` as an explicit retention-only
control; it does not substitute for Type2 active consumption. Disabled Type1
code must never be described as a live ramming parser. GUI graph suffixes,
empty/malformed strings, all cooling/unload paths and non-SEMM multitool
ramming-volume/flow are deferred.

## RED, GREEN and acceptance

Implement vertical groups: mapping/identity/master, sequence, then dataset /
ramming. Before production or classification edits, commit each group's runtime
Native-backed E2E RED: explicit admitted requests, application and consumer
proofs, strict full-stream comparison, then expected bounded PASS. Existing
baseline-only metadata must fail the final expectation. If application,
Native execution or bytes fail earlier, preserve the original artifacts and
report the exact stage/first difference. Do not relabel a
real Ares discrepancy as a metadata RED; no production fix is in scope.

GREEN only authors these six plans and activates/verifies their owned inputs.
Change the `filament_map` / `filament_self_index` negative metadata assertions
only when their finite contracts really close; retain bounded-positive and
explicit deferred-scope assertions. Other keys' assertions remain unchanged.
An unchanged baseline fixture is never credited as an injected G4 case.

Required seeded and official cold/hot directed runs must account for every
successful request and control. Unfiltered selected-key sweeps must execute
all authored cases through the same context preparation/verification and
report exact generated/compared/rejected counts; no value filter, short-circuit
or metadata promotion around an unexecuted case. Context-only controls outside
canonical plan values are enumerated separately. Save repeatable input,
requested/exported/effective proofs, per-layer/tool/flush/ramming observations,
raw streams and comparator output as runtime artifacts, not Native
G-code goldens. Missing Native uses shared requiredFAIL/optionalvisibleSKIPPED.

Inventory diff must prove that only these six plans changed; preserve all
other plans, including concurrent G2/G3/flow updates. Fresh base/candidate
seeded/official controls compare individual statuses and distinct common Ares
raw SHA sets, retaining repeated capture counts separately. Unexpected Native
nondeterminism is reported with unchanged-input repeats, not normalized away.
Default environment workspace via nextest, strict clippy, fmt and core/adapter
WASM checks must pass. No whitespace, duplicate-command, M73, residual or
footer filtering; generated-by banner is the only whole-output normalization.

Deferred: global numeric/string/vector domains; arbitrary nozzles and painted
multi-volume inputs; dynamic mapping, every auto-flush grouping strategy/TPU
combination; physical/printer/print extruder-map domains; invalid internal
layouts; nullable dataset selectors; Type1 ramming semantics beyond retention;
unrelated stock-signal crashes and existing Native nondeterminism. A finite
G4 result is not a new650-domain complete execution or general parity claim.
