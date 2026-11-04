# Wipe inward: printed support, atomic offset paths and execution

Status: Accepted. The port matches all three complete wipe streams, including
M73. The historical RED observations below describe the pre-port checkpoint;
this does not claim complete family closure.

Upstream: OrcaSlicer beta `345efec64c7b95a05ce5cf96df0dde66a158835c`.

## Context and owning source boundary

The unsupported-loop consumer intentionally ports
`GCode.cpp:8889-8933` as `wipe_support == nullopt`. Its prior prerequisite
STOP identified the missing optional support and wipe-offset consumer.
That entire behavior is deferred to this one slice, not to a dead prefix
stub or silently enabled approximation.

Current Ares `wipe_inward` / `wipe_inward_distance` are registry metadata
only. Existing `loop_paths/inward.rs` implements the separate
`wipe_on_loops_destination` behavior: a post-loop nozzle nudge. It is not
this supported offset-path feature. Existing travel wiping combines
`Wipe::wipe` movement with retract-before/after bookkeeping. Calling that
combined sequence for the new zero-retraction wipe would change E state.

This slice rewrites the following upstream chain, not an Ares-owned wipe
planner, a second G-code writer or an independent geometry policy:

- `PrintConfig.cpp:6670-6698`: existing `wipe_inward` bool, default false;
  `wipe_inward_distance` FloatOrPercent, default50%, measured against the
  actual external path width. Reuse registry definitions and constraints;
  project the source values through region options and effective motion
  options. `GCode.cpp:3407-3420`, including the false override at3416,
  applies only to `print.calib_mode()` PA-Line, PA-Pattern, PA-Tower,
  Auto-PA-Line and Retraction-Tower jobs. The current Ares emitter/API
  exposes no calibration mode or job admission, so this branch is
  explicitly unreachable, not an existing calibration
  adapter to invent. Add no calibration bool, stub, hidden mode or new
  pipeline. Recheck the effective region value at actual execution; any
  later exposed calibration rewrite must supply the cited false override.
  No new key, preset refresh or unrelated legacy migration.
- `GCode/WipePathHelpers.hpp:12-18`, `.cpp:21-38::WipeInwardSupport::append`:
  region-scoped `printed_lines` and `inner_lines`. Append actual entity
  polyline lines after extrusion, using the source entity's `as_polyline()`
  rather than a substituted clipped local copy or the generated wipe path.
  Mixed loop/multipath paths qualify as inner support if any path is
  internal, not just their first role.
- `GCode.cpp:8903-8907`: optional context exists only with writer wipe
  enabled, filament wipe enabled, `wipe_inward` enabled, positive requested
  distance and wipe distance greater than the source scaled epsilon.
- `:8918-8921`: in the deferred, non-infill-first perimeter pass, seed the
  context with nonwaiting entities already emitted before infill. Do not
  count future deferred walls or another region as printed support.
- `:8923-8929`: apply the accepted `waits_for_infill` filter, forward the
  optional context into actual extrusion, append each selected entity only
  after it has printed. The region/per-pass allocation and preseed order
  are source semantics, not a global layer cache.
- `:8494`, `:8847-8854`: forward the optional context through
  `extrude_entity` to `extrude_loop`, without changing other entity dispatch.
- `:8729-8801`: share the existing `wipe_on_loops` destination with the
  offset construction; store loop paths in print order; recheck effective
  wipe settings at execution, including calibration overrides. Require
  already printed inner lines and any external path in the loop. Resolve
  percent/literal distance from `paths.front().width`, clamp to that width,
  and use the source support-distance limit. Do not use a config width of
  zero for Auto or substitute nominal width for variable-width paths.
- `:8767-8785`: build an atomic candidate from the stored wipe path using
  seam start/end, actual wipe start after any nozzle pre-move, material-side
  direction, configured maximum wipe length and inner/printed/current
  lines. Replace the stored path only when the source helper succeeds.
- `:8790-8801`: preserve the existing nozzle pre-move, then immediately
  execute an accepted path with `m_wipe.wipe(*this, 0.)`. This must not force
  retraction, reset E, assign restart-extra, perform Z-hop or defer the
  accepted path until another extrusion overwrites it.
- `GCode.cpp:668-675::Wipe::update_path`: preserve print/reverse direction
  and Polyline boundary de-duplication. `:677-756::Wipe::wipe` is the owning
  execution contract: replace only the storage sentinel with last_pos,
  clip to configured wipe length, emit the source markers/feed and segment
  moves with the passed retraction amount (zero here), update last_pos and
  reset the stored path. Reuse the existing writer/formatter and keep the
  ordinary retract caller's before/during/after semantics unchanged.

## Geometry helper slice, included literally

Include the missing offset/support helpers in `WipePathHelpers.cpp:40-884`
and the corresponding header contracts. Reuse the already ported
`sample_path_at_distance:61-92` contract rather than duplicating or retuning
it. The missing helper set is: source constants and `wipe_tolerance`;
`prepare_source`,
`build_offset_polyline`, `wipe_offset_direction`, `starts_by_backtracking`,
`sample_polyline_at_distance`, `store_wipe_path`, `offset_wipe_path`,
`translated_wipe_path`, `segment_is_supported`, `wipe_path_support_score`,
`initial_connector_is_clear`, `support_offset_at_start`,
`executable_path_length`, `material_side_support_lines`,
`wipe_path_stays_on_material_side`, and
`offset_wipe_path_toward_support:677-884`.

Non-negotiable source semantics:

- Preserve the storage sentinel versus executed first point, seam-gap and
  closing-edge joins, atomic rejection and configured wipe-length clipping.
- Retain source rounding/truncation, miter/bevel, reversal, support-distance,
  local material-side, clearance and connector/intersection tests. Audit
  source arithmetic and every existing tree query before reusing it; do not
  create a different distance/intersection kernel or epsilon adjustment.
- Candidate trees are built once per loop and reused across candidates, as
  upstream does. Support comes from eligible earlier inner walls; complete
  safety checks include current and earlier printed walls only.
- Keep preferred, alternate, translated, direct and incoming-wall reverse
  candidates, their exact construction/validation order, strict score and
  length margins, and equal-case preference. These are the upstream
  algorithm's branches, not license for an Ares fallback or heuristic.
- When no candidate is accepted, retain the ordinary stored wipe path
  unchanged, exactly as the source does. Do not fabricate a short inward
  move, reverse all paths, tune a tie epsilon or force an extra wipe.

The source's existing `wipe_on_loops_destination:886-926` implementation
is not rewritten or independently retuned. Its already computed destination
is an input to the cited interaction, not a replacement for offset support.

## Exact Rust destination and compatibility-shell replacement

All production work stays in `ares-core`, portable across browser WASM,
Windows, macOS and Linux, with no filesystem/UI/terminal/native-only code,
new crate or public scheduling API:

1. `options/region_fields.rs`, `process_options/region_source.rs` and its
   wire serializer, plus `gcode_emit/motion/options.rs` and its region
   adapter: project the two existing option values and defaults using the
   existing macro/override pipeline. `RegionOptions` derives these fields;
   do not duplicate raw-option lookups or add parallel option policy.
2. `gcode_emit/motion/layer.rs`'s current regional perimeter-pass adapter:
   own the optional support context per source region/pass; preseed only the
   already emitted normal walls for the deferred pass. Keep the existing
   first-layer/region selection and unsupported-loop filtering unchanged.
3. `motion/perimeter.rs`'s existing entity dispatcher and `loop_paths.rs`:
   transport the optional context and append after real emission. Preserve
   the old complete-collection touching-lines context separately; it is not
   equivalent to a printed prefix. No global cache, stub or unused field.
4. A private `motion/loop_paths/wipe_inward/` module group mirroring the
   named helper boundaries: support collection; offset construction;
   support/material-side validation; candidate selection. Split files
   before400LOC and helpers before the function/arguments/nesting limits;
   retain native candidate evaluation and commit order across those seams.
5. Existing `travel/wipe_path.rs`, `travel/wipe_sequence.rs` and writer
   state adapters: expose the actual `Wipe::wipe(length, ...)` movement
   operation for the zero-length call, sharing the current path construction
   and formatter. The combined retraction sequence is a temporary
   compatibility shell around this upstream contract, not a new execution
   engine. Preserve ordinary callers, origins/offsets, markers, feed cache,
   last scaled position and path reset without forcing retraction/reset-E.

Before implementation, complete a source operand/ownership audit of path
orientation after clipping/scarf, nearest/intersection query units and
rounding, actual path width and the zero-length writer call. If a distinct
unported prerequisite beyond this named chain appears, STOP and report
before changing another source boundary. This proposal does not silently
resolve such a prerequisite or add a debug carrier trace.

## Included, deferred and public qualification gate

Include the whole optional prefix, supported offset helper and real
execution chain above. It is generator-neutral at the existing entity
boundary: preserve loop/multipath role eligibility and actual path width.
The initial public witness may be Classic; that does not establish enabled
Arachne parity. Arachne's separate `PerimeterGenerator.cpp:3108` unsupported
producer remains deferred, not implicitly implemented by wipe support.

Also defer seam-selection/scarf solver changes, `wipe_on_loops` geometry,
overhang/support generation, Flow/arc/formatter changes, ordinary travel
policy, cooling/estimator/M73 fixes, preset/config-footer repairs and UI
controls. Preserve the Classic deferral port and disabled/nullopt
behavior; no legacy mode, silent ignore or independent wipe policy.

The public witness uses at most three inputs. They were frozen before Native execution: unchanged base30x30x.8 box
geometry with hollow walls, enabled Inner/Outer with earlier inner support,
an otherwise identical disabled control, and an enabled single-wall case
with no possible earlier inner-wall anchor. Only `wipe_inward` differs in
the disabled control; only `wall_loops` differs in the single-wall control.
All non-project-settings ZIP members remain byte-identical to the base.

Require actual public wipe movement/markers, direction relative to the
already printed walls and infill/deferred pass order, not private candidate
scores or TYPE-only assertions. Native2 fresh stable then validatedAPI1 /
CLI1 per agreed input, no retry/producer switch/geometry search or probe.
A failed premise or earlier unrelated first difference means STOP, not
another candidate. The RED precedes literal implementation.

## Frozen public RED and ordinary-path controls

The committed input-only generator/provenance lives in
`tests/parity/wipe-inward/`. No Native output is embedded. The existing
Native stream harness first qualifies a fresh stable Native pair and
checks public movement premises before validated API and CLI execution.

| Case | Input SHA256 | Strict result / actual Native observation |
| --- | --- | --- |
| Enabled Inner/Outer, three walls | `e160622e1c76a54796dee510a8970e65d28c145960b20315f0a32ec88db432bf` | RED; four immediate zero-E interior wipes after closed outer loops with earlier inner walls; API/CLI have none |
| Disabled control, same geometry/settings | `2c904340328ca20f97e29d046a0b211730db12f1683238ca093842da49b95ed1` | Complete PASS; four ordinary negative-E wipes on the preceding closed outer boundary |
| Enabled single-wall/no earlier anchor | `b11d24c6414ddefd57e6a31452dd1d5e197f3864cdb5ad57e18a017c2a1b2a12` | Complete PASS; no Inner-wall extrusion, four ordinary negative-E boundary wipes, no interior shift |

Exactly one execution per frozen input, positive case first with STOP gates:
Native6 fresh exports / validated API3 / CLI3, three stable Native pairs,
all raw API/CLI outputs identical. All Native movement premises passed
before the public comparison. There was no retry, extra probe, input
variant, alternative producer, debugger, carrier trace or production edit.
These are actual printed movement and earlier-role/closed-ring observations,
not private candidate identities, support-prefix fields or solver operands.
The reported counts are evidence, not coordinate/count assertion goldens.

Enabled first strict difference is byte2026, line98, column2: Native
`;WIPE_START` versus API/CLI `; stop printing object cube10.stl id:0 copy 0`.
The external M73-command-only first is also original98/98, diagnostic81:
the exact first owning immediate-wipe marker, not an earlier unrelated
motion or timing difference. The strict comparator retains every M73 and
all feed/E/labels/footer/whitespace/trailing bytes; only generator normalizes.
All removed M73 records, maps and full unfiltered contexts are retained
externally; no Fword filter or diagnostic acceptance is used.

The disabled control's four complete generator-only streams agree at
29877 bytes /1009 lines, SHA
`cf566ab42bb2c9255995a662752d0c8ef76152a3ffd1d5d1c6becf3a8cda6c49`,
retaining48M73 records. The no-anchor control agrees at28377 bytes /937lines,
SHA `8899818d9c85319daa2f0cf1f47415b998442009ffb38681cf8349ef97a5a3c2`,
retaining28M73. Both include footer and terminal/trailing bytes. At that
pre-port checkpoint the enabled case was strict RED; no family closure was
claimed.

## Implementation gate

The RED preceded implementation. The port is split into separate commits: complete source helper modules and their source
projection/nesting corrections first, then GCode transport/execution. Every file
stays at most400LOC; no unused prefix stub, placeholder, epsilon tuning or
independent wipe policy. Before/after runtime scope is agreed and frozen
before production edits. The helper and transport split is not permission
to claim GREEN for unconnected source or spend another Native budget.

Later frozen paired breadth must include disabled/zero-distance/no-wipe,
inner-first/outer-first, holes/contours, mixed/variable-width paths, Auto
width, seam gaps/scarf, `wipe_on_loops` interaction, first layer, region /
extruder ownership and the three deferral cases. Follow the bounded
existing tests; no unannounced family expansion. Require0P→F and
M73-inclusive whole ordered bytes, including E/feed, markers, labels,
footer and trailing whitespace. Normalize only the canonical generator
line; any external M73-only diagnostic retains maps, all removed commands
and full contexts and is never acceptance. No coordinate/count golden or
private Native operand fiction.

## Strict GREEN

This remains a separate slice; it does not reopen or bundle the deferral
rewrite. Only the enabled Inner/Outer wipe case changes FAIL to PASS; no
prior PASS changes to FAIL. The three deferral cases remain complete PASS.

The three wipe inputs use fresh stable Native / validated API / CLI with raw
API=CLI. All generator-only complete streams per case are equal, retaining
every M73, ordered movement, role/wipe marker, E/feed, label/footer,
whitespace and trailing byte:

| Case | Bytes / lines | M73 | Complete SHA256 |
| --- | --- | --- | --- |
| Enabled Inner/Outer | 29987 /1017 | 49 | `5ea08b1d9073111a0f9ae3e038c7faffc812136b7c1d5ec50511fd205f80b431` |
| Disabled control | 29877 /1009 | 48 | `cf566ab42bb2c9255995a662752d0c8ef76152a3ffd1d5d1c6becf3a8cda6c49` |
| Single-wall/no anchor | 28377 /937 | 28 | `8899818d9c85319daa2f0cf1f47415b998442009ffb38681cf8349ef97a5a3c2` |

The enabled output now has the same four observed immediate zero-E interior
wipes as Native; both controls retain ordinary boundary wiping without
interior offsets. These observations are not count/coordinate goldens.
Remaining family failures stay open; this is not full-family parity.

The source audit follows integer `Line::Scalar` through the floating AABB
result tuple: the closest point is truncated before promotion to double.
The helper reuses the existing integer query and nearest result. All shared
geometry changes are visibility-only; kernel bodies are unchanged. The
ordinary retraction file is equivalent after expanding its mechanical wipe
movement extraction; the extracted movement tokens are identical. Existing
loop-destination sampling/clipping/rotation is byte-identical. The new
zero-length operation has no retraction/reset-E/restart-extra/Z-hop shell.
The optional prefix is region/pass scoped, preseeds only already printed
normal entities for deferred walls, and appends original entities after
actual emission. Actual path width, source candidate order and atomic
acceptance/rejection are retained.

Every changed module is at most400LOC. Clippy corrections preserve source
evaluation order. No suppression, EPS/margin/tie tuning or
kernel/formatter/estimator change is used.
