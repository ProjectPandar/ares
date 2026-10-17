# SPEC RR: gated outer/inner wall flow ratios

Status: **SPEC-REVIEW-REQUEST, Native discovery complete; no public RED or
production implementation yet.** Assigned TOP2 only. Base is main
`b3251d537d2b889d5eac27821e7cae5f535e2fdd`; branch
`worker1/wall-flow-discovery-spec`. Preserve the approved W1 stack-27 chain.
If stack-27 lands first, rebase onto the announced main later and refresh
exact-hash review evidence. Coordinator owns integration.

## Owning upstream rewrite boundary

Pinned OrcaSlicer source: `8500fcdccaa10b5099ac20d252af3a7c560046f1`.
All upstream paths below are under `OrcaSlicer/src/libslic3r/`.

- `PrintConfig.hpp::PrintObjectConfig` owns `set_other_flow_ratios`;
  `PrintRegionConfig` owns `outer_wall_flow_ratio` / `inner_wall_flow_ratio`.
  `PrintConfig.cpp:1387-1394,1404-1422` defines the boolean gate (false) and
  floating role ratios (1). Existing Ares typed parsing is not the missing port.
- `GCode.cpp:6485-6489`, inside `GCode::_extrude`, applies the outer ratio only
  to `erExternalPerimeter`, the inner ratio only to `erPerimeter`, and neither
  when `m_config.set_other_flow_ratios` is false. Overhang perimeter is a
  distinct next branch, **not** an outer/inner wall alias.
- Context matters: `GCode.cpp:6469-6471,6485-6515` forms effective volumetric
  flow from geometric flow, print flow, filament flow and gated role flow,
  then computes E. `:6504-6506` is the separate first-layer multiplier; it is
  already 1 in these witnesses, not an extra feature to activate here.
- The effective flow also feeds the existing speed/volumetric lifecycle
  (`GCode.cpp:6554-6562` and following consumers), not merely the final E word.
  Region config is re-applied before entities (`GCode.cpp:6136,6163`).

This is a two-branch Rust rewrite of the cited `_extrude` consumer, not a new
Ares flow engine, geometry generator, configuration policy or estimator.

## Rust destination and non-negotiable architecture

Existing parsed values are in `options/object_fields.rs` (gate) and
`options/region_fields.rs` (ratios), through their process source types.
The production `project_slice` emission path does not read these two ratio
fields. Old helpers in `options/flow_ratios.rs` / `extrusions/options*` are
not a second pipeline to switch to or a fallback.

The exact prospective destination is the existing
`project_slice/gcode_emit/motion/` lifecycle:

1. `options.rs::MotionOptions` and `options/build/fields/assemble.rs` resolve
   the **owning object's** gate, using the existing resolved-object/default
   machinery. `options/build/region.rs::apply` refreshes the two ratios from
   the current resolved region, just as it refreshes print flow and speeds.
   Do not capture only the first region or invent a parallel config lookup.
2. `materialized.rs::emit_with_speed` already distinguishes
   `ExternalPerimeter` / `Perimeter` / `OverhangPerimeter` and constructs
   `features.rs::PathProperties` from the emitted path. Reuse this handoff
   and the existing path flow preparation; do not alter nominal geometry,
   widths, fitting, seam clipping, or entity topology to encode a multiplier.
3. `path.rs`, `path/constant.rs`, `path/variable.rs`, and
   `extrusion.rs::for_length` are the existing downstream flow/E consumers.
   Apply the selected ratio once, at the owning effective-flow boundary so
   the speed, volumetric/cooling metadata and E calculations agree. Preserve
   the upstream arithmetic order and existing print/filament cancellation;
   a final E-only multiply, double application, or unrelated arithmetic
   rewrite is not acceptable. Any needed factoring stays inside this
   lifecycle, not a new pipeline or estimator repair.

These Ares emission structs are temporary compatibility shells around
`GCode::_extrude` / writer concepts. This slice adds their missing consumer;
it does not rename them into an Ares-owned architectural abstraction.
No new parameters, clamps, guards, flags, fallback, or shared-parser changes.
Changed Rust files must remain below 400 LOC. **Do not touch ironing.rs**;
owner requires a separately verified mechanical split first if later needed.

## Completed Native-first discovery

Evidence root: `/tmp/ares-w1-wall-flow-discovery-b3251d537/`.
`native_probe.py`, `native-report.json`, `native-probe.log`, `canonical.sh`,
`inputs/manifest.json`, raw official/seeded outputs, launch receipts and
actual Native ELF/all-thread affinity samples are retained there.

Four frozen input-only archives share every non-config ZIP member byte.
They use the previously audited closed 10 x 10 x 1 mm box, Classic walls,
two wall loops, 0.2 mm layers, no scarf/arc fitting/spiral/ironing/tower.
Print flow and first-layer flow are 1; filament flow stays 0.98. No setting
other than the listed project configuration changes between cases.

| Input | Gate | Outer | Inner | Observed Native activity in each mode |
|---|---:|---:|---:|---|
| `unit-control` | 1 | 1 | 1 | 5 positive model layers, 224 positive moves; 20 outer + 20 inner |
| `outer075-active` | 1 | 0.75 | 1 | 20 corresponding outer E words change; e.g. `.29437` to `.22078` |
| `inner065-active` | 1 | 1 | 0.65 | 20 corresponding inner E words change; e.g. `.26856` to `.17456` |
| `gate-off-both` | 0 | 0.75 | 0.65 | All 224 positive moves and complete pre-config body equal unit-control |

All cases retain the effective gate/ratios in Native's emitted config. The
active cases change actual extrusion, not just configuration text. Ordered
positive XY geometry/roles remain the same. The inner case additionally
changes real feedrate (`G1 F892` to `G1 F891`); do not suppress that difference
or assume that a flow fix affects only E.

**16/16 Native slices succeeded:** four inputs x official/seeded x two
independent processes. Fresh serial roots/caches, canonical environment,
seeded CPU18, and actual Native descendants/all sampled threads on CPU18.
Complete repeats and cross-mode output are equal after only generator
identity/timestamp normalization, retaining config, M73, statistics and order.
Paired pre-config/command comparisons above are discovery diagnostics, not
an alternative public parity comparator. No Ares API/CLI was run in this
Native-only discovery, so no public RED or control PASS is claimed yet.

Frozen archive SHA256 (full per-member hashes in the input manifest):

- unit: `d838bbdc267eb3b0c83c519a34dea3328e1ea8f8e28c6d37bc6bb27ffa71e270`
- outer: `481fc3239c14b8df6284576e8ead9029d80adc1f81f1a70b8e9fe55464708c00`
- inner: `e3ea859cd40f53dddd8979c5a97823813289446c7aa758dacc64c35d329a0697`
- gate off: `f4c42d79c667cd857fa9b332b9a148234dfb844f01d4147f50586e40ef099165`

## Proposed public RED, only after SPEC approval

Freeze these exact input-only archives/manifests into the existing parity
fixture layout; never commit Native outputs. Add a bounded public E2E module
with three assigned legs (outer, inner, gate-off) plus the unit control.
Native succeeds first, retains settings and exhibits the positive activity
above. Then require byte-in public API **and real CLI success**, raw API=CLI,
and the unchanged `golden::compare_ordered_bytes_generator_only` against
Native for the **same input**. No paired-input substitution.

Expected baseline outcome per mode: **two genuine full-output FAILs** (outer
and inner) and **two whole-output PASSes** (unit and gate-off). This is an
expectation to verify, not a discovery result. Unsupported-feature rejection
must fail the success assertion, never count as a passing RED. Preserve any
unexpected rejection, control mismatch or other blocker and stop for review;
do not broaden production scope or manufacture the expected failure.

The control with both non-unit ratios proves gate-off inactivity. The unit
control guards neutral enabled behavior. Record actual first mismatch,
errors/exits, positive Native/API activity, raw outputs/hashes, complete
Native repeat determinism, config retention, and actual CLI descendants and
all threads (do not mistake a wrapper or missed short process for evidence).

Serial official and seeded, CPU18, jobs2/nextest threads2, fresh official/
seeded artifacts and caches, required Native/all canonical references.
No full/workspace run, push or merge. After independent exact clean committed
RED review and owner approval only, implement the minimal owning consumer.
GREEN requires all four full comparisons in both modes, raw API=CLI, retained
control bytes, targeted related wall/flow cases and fmt/strict clippy/WASM
checks on the exact clean committed hash. No comparator edits, body/config/
M73 filtering, tolerances, sorting, estimator changes or fabricated differences.

## Included / deferred / RR gate

Included: the shared `_extrude` role consumer for `erExternalPerimeter` and
`erPerimeter`, unit behavior and gate-off inactivity in the existing object/
region lifecycle. The bounded E2E witness is ordinary Classic, not permission
for a Classic-only bypass of the upstream role rule. No independent geometry
or pipeline work.

Deferred as separate source-cited witnesses/slices: overhang/gap/support role
ratios; first-layer/other-role expansion; specialized Arachne/scarf/arc/spiral
and non-unit print-flow composition E2Es; multi-region/object override E2Es;
multifilament, support, raft and tower. Deferred E2E coverage is not an
exemption from the shared role/gate semantics. Preserve adjacent algorithms
and unchanged parameters; do not repair unrelated gaps as a side effect.
If shared wiring requires a broader change, stop and amend scope before
implementation rather than adding a fallback.

TOP1 interlocking is shelved because it overlaps W6; TOP3 junction deviation
is queued, not part of this spec. Request independent SPEC review and owner
choice/approval before public RED. Independent RED approval remains a
separate mandatory gate before production GREEN.
