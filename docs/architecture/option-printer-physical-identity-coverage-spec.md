# Printer/physical identity coverage — spec

Status: **PROPOSED**.
This is a source-cited **PrintConfig/PrintApply/GCode rewrite coverage slice**,
not a new Ares scheduling, mapping or motion pipeline. No Native admission,
RED test or production change has been executed under this spec.

## Why this group

The [gap audit](option-coverage-gap-2026-10-11.md) found675 live FFF inputs.
There are5 genuinely no-case keys after metadata exclusions; this proposal
closes two
**baseline-only/no-injected-case** directory-local gaps:

- `printer_extruder_id` (1-based variant identity): selects per-nozzle machine
  parameter payloads, potentially changing every retraction/toolchange.
- `physical_extruder_map` (0-based physical permutation): affects startup and
  per-layer template context independently of filament/nozzle assignment.

This is the largest remaining GCode-impact group by source
reach, not a measured divergence. Existing `runner/generated_printer_ids.rs`
already covers an incomplete printer-id example outside option_coverage.
Preserve and reuse that evidence/fixture recipe, do not call it untested.
Master-TPU, nozzle-dataset, ramming and inherits coverage are not reopened.

## Exact upstream ownership and Rust destination

| Upstream source owner | Included behavior | Rust boundary |
| --- | --- | --- |
| PrintConfig.cpp:5385-5397 printer ID/variant definitions; :9004-9055 `DynamicPrintConfig::get_index_for_extruder` | Complete paired identity lookup, repeated IDs for variants, generated IDs when map is incomplete | `ares-core/src/options/project_variants/index.rs` existing selector; no independent resolver |
| PrintConfig.cpp:9634ff `update_values_to_printer_extruders`; PrintApply.cpp:1166-1167 | Variant-one materialization followed by variant-two/stride-two using the upstream publication order | existing `project_variants/printer.rs` and `project/effective_config` phases; no second materialization pipeline |
| ParameterUtils.cpp:46-78 `get_index_for_extruder_parameter` | Parameter ownership/type dispatch; no printer/process identity substitution | existing typed options/config views; verify via byte API, not an internal instrumentation API |
| PrintConfig.cpp:2495-2501 physical map definition; GCode.cpp:2833-2847 lambda `match_physical_extruder_for_each_filament` | Startup first-tools/first-filaments physical permutation | existing `project_slice/gcode_emit` startup/template publication |
| GCode.cpp:4677-4687 `layer_change_gcode` context | `most_used_physical_extruder_id` published from physical map | existing `gcode_emit/layer_gcode.rs` and template runtime |

ConfigOptionVector::get_at (Config.hpp) is upstream first-element semantics,
not permission to add a new mapping policy. Existing scaffold selections and
physical-map accessors remain temporary compatibility shells; if an admitted
Native witness exposes a mismatch, port only the cited owning operation. Do not change readers preemptively.

Tests/fixtures/artifact I/O belong only in
`ares-cli/tests/orca_parity/option_coverage/` and `tests/parity/`. All production
logic stays platform-neutral ares-core; WASM/Windows/macOS/Linux remain Tier1.
No filesystem, subprocess, terminal or Native dependency enters core.

## Bounded context (proposed, not yet admitted)

Use the already-established two-nozzle Manual H2D0.4 setup, **not Pro**:
2 PLA logical filaments with real wall/infill usage, two Standard0.4 nozzles,
by-layer cube, fixed seed/generator metadata and stock machine/toolchange
commands. Keep prime tower, support, raft, TPU, adaptive PA and timelapse off.

`printer_extruder_variant` and `extruder_variant_list` must form complete
per-nozzle Standard/High-Flow groups as in the stock variant table. Keep
`extruder_type`, active `nozzle_volume_type`, filament_map Manual `[1,2]`,
master ID, print/process identity, temperatures and all geometry frozen.
Use the already-admitted G4 payload
`retract_length_toolchange=["1","1","2","2"]` (Standard/High-Flow slots
for physical nozzle1 then2) to make variant selection observable; do not add
unrelated speed/flow probes. All other machine/filament payloads stay frozen
from the stock fixture. Don't assume export retains four slots:
record and verify exported settings before Native admission.

Physical maps use **only complete permutations** `[0,1]` and `[1,0]`.
Use diagnostic comments appended to the existing start/layer templates:
`first_tools[0]`, `first_tools[1]` at startup and
`most_used_physical_extruder_id` at layer_change. Do not reference layer_num
at startup or curr_physical_extruder_id outside its actual upstream context.
Markers supplement, never replace, complete GCode comparison.

Existing APIs are sufficient: export/bind a 3MF using established fixture
helpers, verify requested/effective fields, run Native against those exact
bytes, call validated public byte API and real CLI against the same input.
No duplicate exporter/builder or scheduling implementation is added.
If private fixture coupling requires a mechanical move,
keep it separately compilable; don't break old runner tests to reuse them.

## Proposed admission matrix

Run Native first; values are requests, not presumed legal:

| Case | Printer IDs / variants | Physical map | Witness obligation |
| --- | --- | --- | --- |
| P0 | Complete stock IDs `[1,1,2,2]`, aligned Standard/High-Flow variants | `[0,1]` | Baseline uses both logical filaments/nozzles; effective complete identity preserved |
| P1 | Reverse physical owner IDs `[2,2,1,1]`, preserve the repeated paired variant payloads | `[0,1]` | Source lookup chooses opposite parameter groups; Native retraction/toolchange E/F payload changes, not only config-header comments |
| P2 | Incomplete ID map `[1,2]` with complete four-entry variants | `[0,1]` | Upstream generated-group identities; reuse existing generated-printer-id admission concept, prove effective selection and complete output |
| M0 | Same complete P0 identity | `[1,0]` | Native startup/per-layer physical-ID markers change while filament_map and printer IDs remain fixed |
| M1 | Same P1 identity | `[1,0]` | Demonstrate physical permutation is independent from printer parameter identity and logical filament_map |

P1 changes only printer IDs; variants/payload values are frozen, permitting a
causal parameter-selection comparison. M0 changes only physical map versusP0;
M1 changes only physical map versusP1. ZIP entries and unrelated project
settings must be byte-identical across each controlled pair, except the
template markers shared by all cases. Record hashes of every retained input.
If a fixture normalizes away a requested difference, classify APPLICATION-
MISMATCH and stop; never credit export-only or defaulted behavior as coverage.

Native failure/crash is retained with actual exit/signal/stdout/stderr and
noGCode fact; it is not a valid golden or a parity PASS. Stop and request a
revised bounded case before implementation if admission fails. No out-of-range,
empty, duplicate physical map, unmatched variant or invented rejection policy
is included. They require separate Native-led safety/validation specs.

## TDD and fail-closed artifacts

1. Native admission forP0/P1/P2/M0/M1, run sequentially. Freeze producer identity, exact input hashes, effective
   vectors, both-tool usage, affected motion/marker excerpts and complete files.
   Confirm P1 native non-header motion difference before using it as a witness.
2. Write/commit the bounded E2E tests before any source correction. Prove each
   case with Native full output, validated API and real CLI. Red or Green is
   an observed result: a newly authored case may already pass; never force RED.
3. Any real divergence: retain all raw evidence, cite the first upstream owner
   and make only the minimal Rust rewrite correction. Tests
   already committed; do not relax expected values, add guards/fallbacks or
   replace equality with a subset/field-selected comparator.
4. Register the directory-local authored domain so a None-valued generic
   baseline cannot be reported as this bounded domain's PASS. Reuse the
   existing domain dispatch/OptionOutcome convention; no new acceptance flag.
   Every required case must compare; no omission/ORCA_ERROR counted as PASS.
5. Engineering after implementation: nextest workspace --no-fail-fast,
   strict workspace/alltargets/allfeatures Clippy, fmt and core/WASM.

Per-case repeatable artifact: input.3mf + hash, request/effective proof,
producer and command identity, complete Native/API/CLI GCode and result/output
channels, comparison report and activation facts. Required Native absence
fails closed. Generator-only existing normalization and existing result
wall-clock exemptions stay exactly unchanged. No extra tolerances, dropped
sections, posthoc filters, golden Native outputs in core or suppression attrs.

## Exit criteria and explicit deferrals

- All5 admitted cases pass strict whole output for API and CLI; Native facts
  prove P1 motion selection and M0/M1 physical-placeholder activation.
- `printer_extruder_id` and `physical_extruder_map` become source-cited authored
  bounded coverage in the requested directory, not global exhaustive domains.
- Existing generated IDs, process identities, master A1/A2 and all existing
  comparison assertions preserved; no field omission through new derived data.
- Any production change is only the owning upstream port, not an Ares feature.
  Existing scaffolds retained only as necessary compatibility shells.
- All new/touched files<400LOC; mechanical moves separated; original failed,
  interrupted and VOID evidence retained.

Deferred: arbitrary ID/vector shapes, physical-map singleton broadcast and
invalid permutations, all variants/nozzle types, Auto grouping/AutoMatch,
TPU/TPU-AMS, printer/process or filament variant redesign, stride-two exhaustive
values, timelapse physical contexts, GCodeProcessor physical-map metrics,
active Type2 tower, SEMM profile closure, nozzle-dataset closure,
inherits and GUI/network/model metadata. No postprocessor/renderer pipeline.

Scope: only Native admission and test-first bounded coverage for this matrix.
A production mismatch requires a source-cited repair proposal before code
changes.
