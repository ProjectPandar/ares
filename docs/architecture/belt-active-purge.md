# Plan active belt purge into a supplied prism

Reference: OrcaSlicer v2.5.0, `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`.
Citations are relative to `src/libslic3r/`; Rust paths are relative to
`crates/ares-core/src/` unless noted. This extends
[belt purge option handling](belt-purge.md) after the inactive multi-filament
admission rewrite.

## Replace the active-purge rejection

`BeltPurge.cpp:48–61`, `Print::has_belt_purge_tower`, detects a belt printer,
`enable_belt_purge_tower`, non-spiral mode, more than one **configured**
`filament_diameter` entry, and a resolved marked print object. This is not a
check for actual tool changes, and `enable_prime_tower` is not part of it.
`Print.cpp:3598–3628`, `Print::process`, calls `_plan_belt_purge` for that case
in non-`ByObject` sequence, after object slicing and extrusion generation.
The planner's no-change return comes after alignment and tool ordering.

At `project_slice/belt.rs::admit`, replace the blanket active rejection with a
bounded active branch: by-layer, one ordinary printable model plus one supplied
marked prism, one instance and one model-part volume each, a common regular
and initial layer height, and two ordinary non-soluble, non-support slots on
one nozzle. Use the already resolved object options and selectors for both
objects. The prism must independently have `flush_into_objects`; the marker
does not confer wiping eligibility. The first scope has no apron, support,
raft, variable layer heights, or other flush recipients. Leave the inactive
admission and independent marked-no-brim rule intact. Do not admit arbitrary
multi-object or multi-instance belt plates as a side effect, or treat a lone
marked mesh as the model-plus-prism case. Unsupported shapes still fail at the
existing validation boundary without G-code.

`Print::validate` rejects multiple marked objects with the belt purge switch
enabled (`Print.cpp:2303–2310`) and checks compatible layers for an active
prism (`Print.cpp:2441–2480,2505–2558`). Reuse the resolved height/geometry
inputs for the supported equal-grid case. The diameter mismatch warning and
the custom-profile checks are wider Native validation behavior; the classic
tower's relative-E and ooze-prevention constraints are not prism constraints.
Use relative E in the first fixture to make extrusion accounting explicit,
not as a new purge activation predicate.

## Rewrite boundary and Rust destinations

| Native owner | Behavior and Rust destination |
| --- | --- |
| `BeltPurge.cpp:85–138,324–345`, `Print::_align_belt_purge_layers`, `PrintObject::belt_shift_layer_grid`; `BeltBrim.hpp:157–173`, `BeltBrimBand` | In `project_slice/belt/purge/`, align the already sliced physical layer grid before any tool ordering reads it. Take `h` from the first nonempty marked prism. Use the first tallest non-prism by last `print_z` as reference; compute each residue from its first layer as `z - floor(z/h) * h`. Shift all nonempty grids by the residue difference wrapped to `(-h/2, h/2]`, ignoring `abs(delta) < EPSILON`. The physical `print_z` and belt floor/global offsets shift together; do not move slice planes or XY. Native also shifts support and apron Z; when those enter scope, shift the same per-object data consumed by `belt/brim/` and `gcode_emit/layers/boundary/belt.rs`. |
| `BeltPurge.cpp:146–183`; `Print.hpp:900–950,1191`, `WipeTowerData::tool_ordering` and `Print::tool_ordering`; `GCode/ToolOrdering.cpp:443–477,765–778,833–894` and `GCode/ToolOrdering.hpp:151–210`, `ToolOrdering`, `LayerTools` | Build the shared `gcode_emit/tool_ordering/` on aligned layers, using Native epsilon for merged Z and no priming. When collecting wall/fill tool visits, fully overridable collections do not by themselves demand their configured tool. Reuse the layer's actual visit order and `WipingExtrusions` eligibility. Reject empty ordering as in Native. Start at its initial tool and carry the current tool across layers; if no actual visit changes it, return the aligned geometry and ordering without trimming or removing fills. Never use a first-layer wipe-tower flag, configured slot count, or synthetic tool visits to decide this. |
| `BeltPurge.cpp:198–256,353–368`, `_plan_belt_purge`, `belt_truncate_layers_above` | In `belt/purge/`, scan visits up to the tallest non-prism's `print_z + EPSILON` for the last real change. Bound by support top as well when support is admitted. If there is a change within that bound, trim the first nonempty marked prism's layers strictly above `last_change_z + EPSILON` before allocating any overrides. Do not trim when the bounded scan finds no change. Pass retained geometry to `gcode_emit/layers/schedule.rs`; retain the already-built `LayerTools` as Native does, but do not export layer visits for deleted or non-extruding geometry. |
| `BeltPurge.cpp:187–196,258–315,376–398`; `GCode/ToolOrdering.hpp:41–104`, `WipingExtrusions`; `GCode/ToolOrdering.cpp:3394–3406,3429–3572,3574–3631` | Put the per-visit allocation in `gcode_emit/tool_ordering/belt_purge.rs`, alongside `wiping/` and its existing collection/copy overrides and volume accounting. On each real ordered change request the old→new flush matrix entry times `flush_multiplier[0]` if `purge_in_prime_tower && single_extruder_multi_material`; otherwise request `prime_volume`. Match layers within `EPSILON`. Mark whole eligible collections, not deduced lengths or partial paths. After marking all changes on a layer, omit unclaimed prism fills for copy zero, retaining walls; **then** apply `ensure_perimeters_infills_order`. The order-only overrides are not volume claims. Account for leftover per layer and cumulatively; a shortage never creates other purge moves. |
| `BeltPurge.cpp:400–431`; `Print.hpp:607–638,686–694`, `belt_undo_purge_plan`, `BeltDroppedFill`; `Print.cpp:3288–3299`, `Print::process` | Native restores truncated layers and fills in original order before replanning. In the one-shot core, keep the original sliced data intact and make the request-local aligned Z, retained layers and fill memberships visible to every downstream consumer. Keep stable collection IDs for surviving overrides instead of porting pointer stashes, layer-link ownership and incremental invalidation. |
| `GCode.cpp:2492–2557,3651–3679,4744–4798,7010–7032,7181–7200,7568–7618,11220–11221,11344–11358`, `GCode::{collect_layers_to_print,_do_export,process_layer,set_extruder}`; `GCode.cpp:11714–11745`, `ObjectByExtruder` | Reuse the planner's ordering at `gcode_emit/layers/schedule.rs`, and the existing bucket and wiping/normal visits in `gcode_emit/layers/{tool_buckets/,tool_entry.rs}`. Use the ordinary filament-change macro and writer for each real change, then overridden object extrusions, then normal extrusions. `; PURGING FINISHED` separates passes when that layer has overrides; it is not a one-per-change counter or proof that the volume was met. Feed the same aligned floor and physical Z into `layers/boundary/belt.rs` and the existing belt writer. No standalone purge G-code generator or `WipeTowerIntegration`; preserve the classic exclusion (`Print.cpp:5189–5207`, `GCode/ToolOrdering.cpp:419–440`). |

The inactive-only admission and the classic-only call site for wiping in
`gcode_emit/wipe_tower/` are temporary boundaries, not templates for a second
scheduler. The belt planner must share collection routing and normal export;
it must not instantiate classic tower data just to make the wiping pass run.

## Slice order

1. **First observable slice: an unbrimmed supplied prism with real swaps.**
   Port the bounded admission, grid alignment, common tool order, bounded
   truncation, volume allocation, fill retention and normal G-code visits as
   one vertical slice. The directional matrix and prime variants use identical
   geometry and both volume paths; they must differ in the claimed extrusion,
   not just the configuration echo. An admission-only change or a planner
   whose result never reaches export cannot satisfy the slice. Use the Native
   early-return behavior for a supported pair with no visit change, without
   introducing fake swaps.
2. **Brim and additional ordering cases.** Cover matched prism/model plates
   with apron bands and coincident brim. Align band Z with its object and keep
   the marked prism brimless. Preserve the inactive chain's brim-filament
   selection, including apron-only layers and any real same-Z return switch
   (`GCode.cpp:6155–6267,7568–7618,8134–8162`). Cover no-change and missing-
   absorber cases explicitly before widening admission. Port the Native
   critical shortage warning to the public slice warnings (total
   leftover strictly above 1 mm³, ceiling for total and worst-layer volumes,
   worst physical Z formatted to two decimals, `BeltPurge.cpp:296–315`). Do not
   add warning comments to G-code.
3. **Wider planner cases stay outside the first slice.** A lone marked prism
   with active purge is deferred. Native does not reject it for lacking an
   ordinary model: alignment returns without a reference grid and the planner
   proceeds to tool ordering (`BeltPurge.cpp:109–121,160–183`). Ares refuses
   that shape without emitting G-code. Multiple ordinary objects,
   custom/variable grids, additional absorbers, support, raft,
   soluble/support filaments, mixing, and plate-event interactions need their
   own comparisons before admission. Do not port spiral or by-object purge
   generation; Native does not take the planner there. Automatic prism mesh
   generation, width sizing and placement belong to `ensure_belt_purge_tower`
   in `src/slic3r/GUI/BeltPurgeTower.cpp:35–45,242–264,409–442`, outside this
   backend rewrite. A width setting must not resize the supplied mesh.

## Native fixture and byte contract

Start with `tests/parity/mesh-edge-pairing/cube-control.3mf` (synthetic
10 × 10 × 2 mm mesh), use 45° X tilt, the existing belt axis remap, 0.2 mm
layers, and one 0.4 mm nozzle. Put the model at `(110, 110, 0)` and a separately
supplied 10 × 10 × 6 mm prism at `(145, 110.07, 0)`. Resolve the ordinary
model's internal solid infill to slot 2 and its walls to slot 1; mark the prism
and independently enable object flushing on it. Provide two 1.75 mm filament
slots and a 2 × 2 flush matrix. Keep `enable_prime_tower` on to test exclusion
of classic tower output. Use a comment-only `change_filament_gcode` such as
`; swap [previous_extruder] -> [next_extruder]` so the native selector emits
its own `T0`/`T1`.

Both Official and Seeded Native sliced the directional matrix, prime and
oversized matrix variants, and the inactive control. The observable checks are:

- Align the first extruding prism layer to `;Z:77.9675`, the same as the
  model, rather than the inactive prism's `;Z:78.017`. In the first active
  prism wall move the emitted machine Z is `110.263`, versus `110.333` in the
  inactive control. Physical `;Z:` is not the emitted machine axis. The last
  active prism extrusion is at `;Z:84.9675`, the last real change, while the
  model continues to `;Z:86.1675`. The inactive prism continues to
  `;Z:86.217`. There are active prism walls but no prism fills before the
  first swap.
- The `-1 -> 0` startup macro is not a swap. The first real macro is `0 -> 1`
  at `;Z:79.1675`, then `1 -> 0` at `;Z:79.3675`. At the first swap `T1`
  precedes prism extrusion near `X65.675 Y1.947 Z111.96`; the completion
  separator precedes model extrusion near `X101.082 Y1.182 Z111.96` with
  that same tool. No variant has classic wipe-tower moves or roles.
- With directional matrix entries `6` and `10` mm³ and multiplier `0.5`,
  the requests are 3 and 5 mm³. The first `T1` wipe uses only walls
  (`E1.42664` mm, about 3.431 mm³); the return `T0` uses walls and solid fill
  (`E2.17857` mm, about 5.240 mm³). Reversing the lookup or ignoring the
  multiplier would change which fills are kept. With matrix use disabled and
  `prime_volume=3`, both use only walls (`E1.42664` and `E1.46069` mm, about
  3.431 and 3.513 mm³). Whole-collection allocation can exceed a request.
  Measure only positive relative E with XYZ motion on the prism before the
  separator, excluding retract/unretract and all ordinary model extrusion.
- An oversized matrix control (`40` and `64` mm³ at `0.5`) requests 20 and
  32 mm³ but deposits only about 4.744 and 5.240 mm³ on those first two swaps,
  with walls and solid fill in both cases. A shortage does not move the
  remaining extrusion to a classic tower. The configuration echo writes the
  scaled matrix separately (`GCode.cpp:8272–8298`, `append_full_config`);
  it is not a measure of what was deposited.

Add public CLI comparisons under `crates/ares-cli/tests/orca_parity/belt_print/`.
Compare the complete ordered output to each lane's own Native output, using
only the existing generator handling. The highlights above explain what the
comparison must detect; do not mask motion, trim sections, or store reference
G-code. Retain the towerless controls on their existing admission boundary;
use the two-object inactive control as a Native diagnostic, not as an Ares
success case until that shape is separately admitted.

## Implementation base

Stack on the rebased inactive multi-filament implementation (or its landed
successor). Both slices touch admission in `project_slice/belt.rs`, share
`project_slice/belt/brim/` and share `gcode_emit/layers/schedule.rs`. Also
coordinate `gcode_emit/layers/{boundary/belt.rs,tool_entry.rs}`: the former
sees aligned floor and band state and the latter must preserve the inherited
brim/tool entry order when wiping passes enter the same traversal. The shared
`gcode_emit/tool_ordering/` and `wipe_tower.rs` boundaries must keep their
inactive and Cartesian behavior. Do not implement in parallel against the
single-slot admission path.
