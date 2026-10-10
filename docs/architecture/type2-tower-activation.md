# Type2 wipe-tower activation parity (TiQ2 / Kobra 3 Max)

Status: **OPEN** — closure withdrawn; enable=1 behavior and the responsible
upstream gate remain unproven. No production fix is authorized.

## Evidence correction (2026-10-07)

The closure recorded in `2c489196a` was based on the wrong fixture state.
The original ZIP
`/tmp/ares-orca-cache/19f06bc6065b3213/6e02aea054799098-fae0cc8012f785c5.3mf`
(SHA256 `74ada1cbfe1e28ce7f52007bd4f77f80f3014488ee21fb580a27b7c3f172de3a`)
stores `enable_prime_tower="0"`. The observed `"1"` belonged to an edited
extracted `Metadata/project_settings.config`; that edit was never
successfully repacked into the input ZIP used by either slicer.

The 33995-line Native and 33743-line Ares outputs therefore establish only
that both emit zero tower sections for the **disabled** input. They do not
prove parity for enable=1, front-layer partition values, which upstream
gate suppressed the tower, or that any cited commit fixed the reported
140-section divergence. Those fixed-at-main and causal claims are withdrawn.
Passing `smooth_type2_tower` positive controls does not repair this missing
negative-case evidence.

Before further conclusions, change the original ZIP's configuration,
repack it, reopen the resulting ZIP to verify enable=1, and record its
SHA256. Run Native first, then Ares on those exact bytes. Only a confirmed
divergence warrants the two-layer RED: whole-file byte comparison with
only generated-by normalization, plus a ToolOrdering core test whose
expected state comes from a minimal nix oracle probe under
`tools/orca-probes/`. Pinpoint one divergent site before tightening the
spec and requesting implementation review.

Skirt first-difference localization is deferred until that verification.
Its first observed mismatch is line 86, first layer Z=0.2, feature Skirt.
Compare `_make_skirt`'s pre-offset occupied hull and support_fills points
(`Print.cpp:2646-2760`, especially `:2691`) before deciding whether it
shares pi-arachne's SupportSpots point-insertion cause.

Owner: worker1 (reassigned 2026-10-10); worker2 owns G4 only and does not
change tower activation. Divergence found
during pi-arachne's bridge_over_infill multi-filament E2E (`clusters.md`,
`DIVERGENT — Type2 wipe-tower activation`).

## Symptom

Two stock/multi-filament machines diverge on whether a prime tower is
emitted at all:

- **iQ TiQ2 0.4 nozzle** — 2 filaments, `enable_prime_tower=1`,
  `wipe_tower_type=type2`, `single_extruder_multi_material=0`. Native emits
  Historical report: Native emitted zero tower sections and Ares 140.
  The recorded fixture below actually has enable=0, so it does not
  reproduce that enabled-input report. Reproduction is pending:
  `/tmp/ares-orca-cache/19f06bc6065b3213/6e02aea054799098-fae0cc8012f785c5.3mf`.
- **Anycubic Kobra 3 Max 0.4** — single nozzle,
  `single_extruder_multi_material=1`, 2 used filaments,
  `enable_prime_tower=1` in project settings. Native footer records
  `enable_prime_tower = 0`; Ares keeps it enabled and emits a tower plus
  `first_layer_print_*` bounds covering the footprint. Fixture
  `/tmp/ares-orca-cache/19f06bc6065b3213/86f53a71255e4322-642db0936fa7f47a.3mf`.

The two cases suppress the tower through **different upstream gates** and
must be bounded separately — this spec covers the TiQ2 (Type2) case; the
K3M `normalize_fdm_2` path is the same upstream file but a different
predicate and is noted as adjacent-deferred.

## Upstream boundary (source-cited)

Native `Print::has_wipe_tower()` (`Print.cpp:3290-3302`) is only the
*enable* gate — it returns `true` for TiQ2 (`enable_prime_tower=1`,
`!spiral`, `filament_diameter.values.size()>1`). The actual suppression is
the early return in `Print::_make_wipe_tower`
(`Print.cpp:3400-3404`):

```cpp
m_wipe_tower_data.tool_ordering = ToolOrdering(*this, -1, is_wipe_tower_type2);
m_wipe_tower_data.tool_ordering.sort_and_build_data(*this, -1, is_wipe_tower_type2);
if (!m_wipe_tower_data.tool_ordering.has_wipe_tower())
    return;                       // Don't generate any wipe tower.
```

`ToolOrdering::has_wipe_tower()` (`ToolOrdering.hpp:236`):

```cpp
return ! m_layer_tools.empty()
    && m_first_printing_extruder != (unsigned int)-1
    && m_layer_tools.front().has_wipe_tower;
```

So the tower is emitted iff the **front** `LayerTools` has
`has_wipe_tower`. `has_wipe_tower` is computed by
`ToolOrdering::fill_wipe_tower_partitions`
(`ToolOrdering.cpp:816-920`):

- `:822-831` per-layer `wipe_tower_partitions = extruders.size()`, minus 1
  when the first extruder equals the previous layer's last extruder.
- `:833-835` partitions propagate **down** (`max` of the layer above).
- `:842-847` the real gate —
  `lt.has_wipe_tower |= lt.has_object && (timelapse==tlSmooth || wipe_tower_partitions > 0)`
  (plus raft/wrapping/empty-layer fix-ups below it).

`lt.extruders` is populated by `ToolOrdering::collect_extruders`
(`ToolOrdering.cpp:661-760`) — it pushes **filament-id slots actually
extruded on that layer**, derived from each LayerRegion's non-empty
`perimeters.entities` / `fills.entities` (`outer_wall_filament_id`,
`inner_wall_filament_id`, `internal_solid_filament_id`,
`top_surface_filament_id`, `bottom_surface_filament_id`,
`sparse_infill_filament_id`), not the configured slot count. Support
layers contribute `support_filament`/`support_interface_filament` slots
(`:760-`).

**Possible TiQ2 no-tower mechanism (unproven)**: if the ordered layers
require no tool changes, their partitions may remain zero and the
front-layer gate may suppress tower generation. No native probe has yet
established this state for a correctly repacked enable=1 TiQ2 input.


**K3M no-tower mechanism (adjacent, deferred)**: `DynamicPrintConfig::
normalize_fdm_2` (`PrintConfig.cpp:8688-8730`) forces
`enable_prime_tower=false` when `!smooth_timelapse && !enable_wrapping &&
(used_filaments==1 || (ByObject && num_objects>1))`. `used_filaments` is
`extruders(true)` = `object_extruders` + support extruders
(`Print.cpp:451-547`), region extruders counted only when the matching
density/loops option is non-zero (`PrintRegion.cpp:80-92`). Here
`used_filaments` resolved to `1` even though `sparse_infill_filament_id=2`
was configured — i.e. upstream counts *used* slots, not configured ones.
This is a different suppression site (config normalization, not
tool-ordering) and is deferred to the `normalize_fdm_*` port slice.

## Ares destination boundary

- `crates/ares-core/src/project_slice/gcode_emit/wipe_tower.rs::
  type2_requested` (`:31-41`) — ports `has_wipe_tower()` enable gate.
  Faithful; **not** the divergence source (correctly returns `true` for
  TiQ2, matching upstream).
- `crates/ares-core/src/project_slice/gcode_emit/tool_ordering.rs::
  build` (`:241-243`) — `type2_tower = prime && first_extruder.is_some()
  && layers.first().has_wipe_tower` — ports the `has_wipe_tower()` early
  return faithfully (front-layer check, not `any()`).
- `crates/ares-core/src/project_slice/gcode_emit/tool_ordering/
  partitions.rs::fill` — ports `fill_wipe_tower_partitions`.
- `crates/ares-core/src/project_slice/gcode_emit/tool_ordering/
  collect.rs::object` / `support` — port `collect_extruders` object/
  support extruder-slot selection.
- `wipe_tower/planning.rs::prepare` iterates `ordering.layers` and skips
  `!tools.has_wipe_tower` (`:74`) — ports the tower-depth plan.

## Suspected divergence (to confirm with a RED before implementing)

The source-level gates are candidate comparison boundaries, not measured
TiQ2 state. First establish enable=1 Native/Ares behavior. If it diverges,
probe configuration normalization and ToolOrdering inputs before selecting
one site to change. Candidate sites, in audit order:

1. `collect::object` (`collect.rs:29-67`) pushes a filament id whenever a
   `usage.*` flag is set on `layer.tool_regions`. Upstream pushes an id
   only when the corresponding `entities` list is non-empty *and* (under
   `m_print_config_ptr`, i.e. not ByObject) survives
   `wiping_extrusions().is_overriddable_and_mark`. If Ares marks
   `usage.infill`/`perimeters` for a region upstream leaves empty — or
   does not apply the `extruder_override`/overriddable suppression — it
   will over-populate `extruders`, inflating `wipe_tower_partitions`.
2. `collect::support` / `raft_roles` contributing a `support_filament`
   slot on layer 0 that upstream's `collect_extruders` support pass
   (`ToolOrdering.cpp:760+`) does not, raising `layers[0].extruders` to 2.
3. `first_layer::generate` / `first_layer::handle_dontcare` /
   `reorder::apply` seeding `extruders` on layer 0 differently than
   upstream's first-layer ordering (`ToolOrdering.cpp:190-241`), so
   `front().extruders.size()>1`.
4. `object_bottom_z` / `max_layer_height` in `build` (`:170-242`)
   differing from upstream's, flipping the `print_z < object_bottom_z`
   raft-seeding `has_wipe_tower` on `front()`.

The RED must show, for the TiQ2 fixture, which of
`layers[0].extruders` / `has_object` / `object_bottom_z` /
`wipe_tower_partitions` differs from the upstream `m_layer_tools.front()`
record, before any production change.

## Current verification slice (2026-10-10)

The enable=1 result is unknown: do not encode zero Native tower sections as an
expected outcome before running the corrected input. Worker1 first verifies
activation through the public CLI complete-output seam, without production edits.
Repack the original project at runtime, change only enable_prime_tower to 1,
reopen the ZIP and record configuration, whole ZIP and member hashes. Preserve
all other settings and geometry. Run live Native first, then Ares on those exact
bytes; preserve channels, exits, result JSON and complete G-code outside Git.
Missing or rejected inputs are an admission blocker, not activation RED.

Localization starts at the first complete-output difference, not tower counts.
Trace `Print::{has_wipe_tower,_make_wipe_tower}`, ToolOrdering front-layer
state, `WipeTower2.cpp` planning/generation and `GCode.cpp::WipeTowerIntegration`
consumers as reached by that witness. Existing Rust `gcode_emit::{tool_ordering,
wipe_tower,motion::tower}` remains the private upstream rewrite destination;
no new Ares pipeline, synthetic suppression or guessed gate is permitted.
If Native and Ares already match, record verification GREEN without inventing a
RED or fix. If they diverge, commit a genuine test-first whole-output RED, record
measured upstream/Rust state at one owning site, and tighten the implementation
spec before changing production. ToolOrdering probes are diagnostic evidence;
an isolated test needs an independently measured Native expectation and approval.

Keep the existing tower-emitting positive control and its full-output assertions.
K3M normalization, Type1, general multi-transform preparation and unrelated skirt
geometry remain adjacent/deferred unless the first difference identifies a
required dependency, which must be reported rather than silently bundled.

Directed Native starts require load<60, jobs2/threads2. Yield all heavy work at
any worker's exclusive full START; current full queue W2→W5→W3→web. No Native
products are committed, and no full slot is implied by this verification slice.

## Included in this slice

- Reproduce TiQ2 no-tower as a behavior RED: Ares emits tower, native
  emits none, identical fixture.
- Source-bounded fix so `layers.first().has_wipe_tower` matches
  `m_layer_tools.front().has_wipe_tower` for the Type2 single-used-tool
  case — whichever `collect`/`first_layer`/`partitions`/`object_bottom_z`
  site is responsible.
- A second control: a Type2 multi-filament print that *does* need a
  tower (two tools co-extruded per layer) still emits it — guards against
  over-suppression.

## Deferred

- The K3M `normalize_fdm_2` `used_filaments==1` suppression
  (`PrintConfig.cpp:8688-8730`) — separate predicate, same file; belongs
  to the config-normalization port, not the tool-ordering boundary.
- `wipe_tower_partitions` propagation for multi-nozzle same-layer
  changes beyond what the RED exposes.
- Type1/`is_wipe_tower_type2=false` WipeTower construction
  (`Print.cpp:3440+`) — untouched.

## Exit criteria

- New `orca_parity`/`loaded_config` test on the TiQ2 fixture asserting
  Ares produces zero `;TYPE:Prime tower` sections and a native-identical
  whole-file ordered-byte comparison with only the generated-by banner
  normalized. Tower counts and ToolOrdering probes are localization
  evidence, not substitutes for that comparison.
- A positive Type2 control keeps emitting the tower.
- No change to `serde_json::Value` fingerprint count.
- Workspace nextest + clippy + fmt + wasm check green; seeded + official
  parity (run sequentially, not concurrently) show no new failure names
  vs base.
