# Modifier-volume PrintRegionConfig admission for object ironing

## Status and observed delta

SPEC for review only; no production edits. Base is main `2fe214545c1a50e88bf321b8221f0eba73a83ad8` plus approved object-ironing chain `8c9a1bc94..32e111a06`. Existing frozen inputs and evidence are under `/tmp/worker2-zaa-ironing-lifecycle-root`.

Native accepts the input-only 3MF with a `modifier_part` carrying per-volume
`ironing_pattern`, `ironing_angle`, and `zaa_*` metadata. On the
no-replacement control (global Concentric ironing, modifier part present),
Native emits 584 Ironing segments while Ares emits zero; Ares API and CLI
produce the same zero-Ironing output. This is a silent modifier-volume
PrintRegionConfig admission/integration gap, before the approved
`Fill.cpp:1630-1640` lifecycle can be exercised.

## Upstream boundary

- `PrintObject.cpp:3602-3662` copies DynamicPrintConfig keys into
  PrintRegionConfig, preserving non-filament options and handling explicit
  feature extruder overrides.
- `PrintObject.cpp:3664-3709`
  `region_config_from_model_volume` starts from the parent/default
  PrintRegionConfig, applies ModelObject and ModelVolume region overrides,
  then resolves/clamps the resulting region config. This is the owning
  boundary for modifier `ironing_pattern`, `ironing_angle`,
  `ironing_angle_fixed`, and `zaa_enabled`/`zaa_dont_alternate_fill_direction`.
- `PrintObject.cpp:3746-3773` builds the default region config and invokes
  `region_config_from_model_volume` for model volumes while collecting
  printing extruders.
- `Fill.cpp:1571-1614` reads each LayerRegion's resolved
  PrintRegionConfig into IroningParams, including pattern and angle.
- `Fill.cpp:1630-1640` assigns the current filler ZAA bit before replacing a
  filler for a pattern change; the replacement's default false is intentional
  and deferred to the lifecycle slice SPEC `9a113e6d5`.

## Ares destination and included/deferred behavior

The Rust destination is the modifier-region resolution path:

- `crates/ares-core/src/project/region_graph.rs:32-176`, especially
  `build_region_graph` and `add_modifier`, which resolve a modifier volume to
  a `region_id` and append its RegionOptions.
- `crates/ares-core/src/project/effective_config/candidates.rs:135-165`,
  where the resolved object's region graph is installed and modifier override
  keys are rejected by `first_unsupported_modifier_key`.
- `crates/ares-core/src/project/volume_regions.rs:32-68` and
  `region_slices.rs:361-374`, which carry the modifier region through bounded
  volumes into printable surfaces.
- `crates/ares-core/src/project_slice/fill_entities/ironing.rs:43-66`, which
  consumes the selected RegionOptions and currently emits no Ironing when the
  modifier-derived region is absent or collapsed.

Included: admit the PrintRegionConfig keys needed by the frozen fixture,
resolve a modifier part to a distinct RegionOptions when its overrides differ
from its parent, retain the region through volume-region graph and surface
assignment, and let existing ironing code consume it. Preserve the lifecycle
slice's intentional replacement default; do not reapply ZAA here. Keep the
existing unsupported-key policy for unrelated modifier options.

Deferred: pattern replacement/filler lifecycle (`Fill.cpp:1630-1640`),
multiple nested modifiers, negative/support volumes, per-extruder feature
fallbacks, layer-range configs, ironing flow/speed policy, and any comparator
or output workaround. No silent skip or fallback may be introduced. Files stay
under 400 LOC.

## Native-first / RED plan after approval

Use the existing frozen three inputs: mixed Concentric parent plus
modifier-part Rectilinear angle45/ZAA=true, mixed-disabled, and global
Concentric no-replacement. Native evidence is already recorded. Re-run Ares
API+CLI only after SPEC approval on CPU13/14, with fresh roots and unchanged
inputs. The controls must emit their Native Ironing streams; the mixed case
must retain the modifier region and reach the existing lifecycle boundary.

If controls still emit zero Ironing, classify the admission gap and submit a
public test-only RED that asserts modifier-region effective settings and
whole-output Ironing presence before unchanged strict comparison. If controls
pass and only mixed differs at lifecycle direction, stop this slice and return
to `9a113e6d5`. No production edit before RED approval.

## Addendum: primary silent skip localization

Read-only tracing localizes the whole-object zero-Ironing result to
`crates/ares-core/src/project_slice/fill_entities/generate.rs:321-325`:
object ironing is guarded by a single `if let (Some(record), Some(input))`
pair from `prepared.predecessor.objects[object_index].records[layer_index]`
and `prelude.records[layer_index]`. The call then obtains exactly one region
through `prelude.region_options(input)` at `:325` and invokes
`ironing::append` at `:327`. Modifier volume regions are represented as
additional `VolumeRegion.region_id` records by `project/region_graph.rs:103`
and `:137-176`, but this ironing dispatch does not iterate those records or
select a top-surface record for each region. When the modifier-part input
changes the perimeter-record selection/availability, the `Some(record),
Some(input)` guard fails and the entire object skips ironing; this explains
Native's 584 Ironing segments versus Ares zero on the global Concentric
no-replacement control while API and CLI remain identical. The same single
record/region selection also cannot expose the modifier's PrintRegionConfig.

The primary fix target is therefore this dispatch seam and its record-to-region
selection, before `fill_entities/ironing.rs:43-66` and before the deferred
`Fill.cpp:1630-1640` replacement lifecycle. The fix must preserve one ironing
operation per applicable top-surface region, carry each resolved RegionOptions,
and retain the existing no-fallback behavior. The already-approved lifecycle
SPEC remains deferred until this admission path emits Ironing for modifier
inputs. No production edits or instrumentation are included in this addendum.
