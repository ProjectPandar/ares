# Type2 receiving-filament minimum purge

## Source boundary

This is a Rust rewrite slice of `libslic3r::WipeTower2::save_on_last_wipe`,
not a new Ares purge policy or pipeline. Source citations below use OrcaSlicer
`345efec64c7b95a05ce5cf96df0dde66a158835c`. The Ares starting tree is landed main
`8ba63961f0c42cea7b798b8d7e4082a88e334297`.

| Upstream owner | Behavior and Rust boundary |
| --- | --- |
| `PrintConfig.cpp:3401-3410`; `PrintConfig.hpp:1890` | `filament_minimal_purge_on_wipe_tower` is a nonnullable float vector, in mm³, minimum zero, registered default `15`. Existing `options/filament_options/gcode_source.rs` and registry `table/pre_middle_filament.rs` already represent it. No new option is needed. |
| `PrintConfig.cpp:10150,10210` | The key belongs to `filament_options_with_variant`. Existing filament sizing, profile refresh and wire export remain the owners of its applied vector. |
| `Print.cpp:5534-5553`; `GCode/ToolOrdering.cpp:3420-3431` | For SEMM with purge-in-tower enabled, subtract the receiving filament's minimum before assigning infill/object purge, clamp the unassigned remainder at zero, then add the minimum back. Existing Rust `wipe_tower/planning.rs::prepare` preserves this allocation boundary. |
| `GCode/WipeTower2.cpp:1128-1145`; `GCode/WipeTower2.hpp:202` | `set_extruder` stores the per-filament minimum. Existing Rust `wipe_tower/filament.rs::Filament::from_config` already reads it as `f32`. |
| `GCode/WipeTower2.cpp:2187-2248` | `extract_wipe_volumes` floors each destination column by that filament's minimum; `estimate_semm_flush_volume` consumes the result. Existing Rust `wipe_tower/estimate/statics.rs` already contains that separate estimate behavior. |
| `GCode/WipeTower2.cpp:2336-2390` | `save_on_last_wipe` recomputes the finishing change after subtracting finish-layer extrusion, and also floors a non-finishing change when its current wipe volume is below the receiving minimum. The destination is `crates/ares-core/src/project_slice/gcode_emit/wipe_tower/generate.rs::Generator::save_layer_on_last_wipe`. |
| `GCode.cpp:1655-1702,2054-2082` | `WipeTowerIntegration::tool_change` dispatches Type2 results through `append_tcr2`; generated moves, placement and tool identity remain observable. Rust `wipe_tower/emit.rs` remains the consumer, not a place to inject a second purge clamp. |
| `GCode.cpp:10773-10809` | `GCode::update_placeholder_parser_with_variant_params` exposes the remapped option to templates. This is distinct from the Type2 planning floor and is not a new template feature in this slice. |

Paths in the table are under `OrcaSlicer/src/libslic3r/` for C++ and
`crates/ares-core/src/project_slice/gcode_emit/` for abbreviated Rust tower paths.

## Existing Ares behavior and the exact gap

The key is already registered, typed, defaulted, exported, supplied to each
filament, used during SEMM purge allocation, and included in the tower estimate.
Adding another registry member, footer predicate or independent purge calculator
would duplicate those owners rather than port the missing source behavior.

`Generator::save_layer_on_last_wipe` currently recomputes only the change selected
by `finish_after`. It has no counterpart to the upstream `else if` at
`WipeTower2.cpp:2381-2386`. Consequently, a different change can retain a planned
volume below its receiving filament's minimum. This is a source-level gap;
no runtime failure or complete-output match is asserted by this document.

The existing planning/generator structs are temporary compatibility shells around
upstream `WipeTowerInfo`, `ToolChange` and `WipeTower2`. Extend that existing port
at the cited method; do not introduce an Ares-owned tower policy or public API.

## Included behavior and non-negotiable decisions

1. Keep the finishing-change branch: its saved volume comes from the existing
   `finish_layer` result, not a replacement estimate.
2. For a non-finishing change, use the upstream strict predicate:
   current `wipe_volume < minimum[new_tool]`. If it does not bind, leave its
   planned fields bit-for-bit unchanged. Equality is not a reason to recompute.
3. When that predicate binds, perform the same recomputation as the finishing
   branch with zero saved volume. The remaining volume is
   `max(minimum[new_tool], wipe_volume_total - saved_volume)`.
4. Recompute required depth as well as wipe volume. Preserve the existing
   first-wipe-line volume deduction, zero floor, ramming-depth addition,
   perimeter width, layer height, extra-flow arithmetic, and first-planned-layer
   versus subsequent-layer spacing. Use the existing `f32` operation order;
   do not replace the upstream depth calculation with an approximate formula.
5. Preserve immutable `wipe_volume_total`, the five existing planning simulations
   and downward tower-depth propagation. Do not turn the minimum into a new
   total-volume accumulator or repeatedly subtract a prior simulation's saving.
6. Index the minimum by the receiving filament, not the outgoing filament,
   physical nozzle, global maximum or first vector entry for every change.
   Keep existing vector access/default behavior; do not redesign variant routing.
7. The floor in `save_on_last_wipe` is not gated by SEMM or purge-in-tower.
   Those flags gate the earlier `Print.cpp` allocation, not this later branch.
   A non-SEMM Type2 tool change can therefore require the floor as well.
8. Keep the registration type/default/minimum, membership, template exposure,
   footer serialization and every unrelated option unchanged. No CLI, comparator,
   Native-provider, fixture-loader or generator-normalization changes belong here.

The production destination is core-only and platform-neutral. No filesystem,
terminal, OpenGL, UI or browser-specific behavior is introduced. Existing
`ares_core::slice_with_profiles` remains the public byte-oriented entry point.

`wipe_tower_filament` uses `0` for automatic selection and otherwise the 1-based
logical-filament range from `Print.cpp:798-800`, independent of physical extruder
count; when the selector is consumed, Ares retains an error above the logical
filament count instead of upstream's debug-only assertion and release-mode
undefined behavior.

## Deferred adjacent behavior

- Type1 `GCode/WipeTower.cpp` allocation/emission, Bambu carousel/nozzle grouping,
  prime-volume modes, flush-matrix reconstruction and multiplier selection.
- Rewriting infill/object/support allocation, soluble/support eligibility or
  `WipeTower2::first_toolchange_to_nonsoluble_nonsupport`. The existing Rust
  `generator.rs::first_nonsoluble_change` has a separate selection boundary;
  this slice does not change it. The separate [Type2 finish filament port](type2-tower-finish-filament.md) covers that selector.
- New per-layer variant/nozzle remapping, template functionality, profile refresh,
  key membership or vector resizing. Preserve existing float-vector semantics.
- Ramming, cooling, interface-temperature, rib geometry, sparse-layer grouping,
  final purge and G-code placement/formatting algorithms. Their existing output
  remains visible to the full comparison; deferral is not permission to mask it.
- Viewer data and native/browser platform certification. WASM compilation alone
  cannot establish browser slicing behavior.

## Public comparisons to propose before implementation

No tests are added or executed by this document. Confirm the source boundary
before registering new cases. The following cases distinguish the missing branch
from a footer-only or estimate-only change.

| Case | Required observable behavior |
| --- | --- |
| Non-finishing minimum binds | A real non-Bambu Type2 project has at least two relevant changes in a layer; a non-finishing receiving filament has minimum `35` and planned purge `5`. Native must print the corresponding extra tower purge/depth, not just export `35`. Force the same explicit tower-finishing filament on both sides so the separate automatic finish-selection difference is not the cause. |
| Destination identity | Swap distinct receiving/outgoing minima, such as `[5,35]` and `[35,5]`, without changing geometry or tool order. Bind the emitted purge to the actual receiving filament; a global or outgoing-filament clamp must fail. |
| Nonbinding zero/equality | For the same non-finishing change with planned purge `5`, compare minima `0` and `5`. The new branch must not run or disturb previously matching complete output. |
| Existing finishing branch | Retain a finishing-change case whose finish-layer extrusion would otherwise reduce its remaining purge below its minimum. Preserve its existing floor and complete output while adding the other branch. |
| Default and gate distinction | Record applied default `15` separately from explicit values. Include a non-SEMM/purge-disabled Type2 binding case and a SEMM/purge-enabled control; do not incorrectly make the later floor conditional on the earlier allocation gate. |

Start from retained real project geometry such as
`tests/parity/print-order-as-obj-list/class-default.3mf`, already used by Type2
comparisons. This is a candidate input, not a claim that its unmodified settings
exercise every row above. Admit a case only after Native demonstrates its tower
type, tool order, receiving filament and binding/nonbinding condition. Preserve
mesh/container provenance and original geometry hashes; apply identical explicit
configuration changes and matched real profile/catalog/selection inputs to
Native and the public Rust API. Do not fake geometry, patch Native output or
adjust settings merely to hide an earlier unrelated mismatch.

Existing `wipe_tower2_rib` and `wipe_tower2_final_purge` tests compare sections,
boxes or tail statistics. They are adjacent controls, not substitutes for complete
ordered-byte assertions in the new cases. Require generator-identity-only
normalization; keep all moves, tool changes, E values, M73, statistics, headers
and footer lines. Save the complete Native/API/CLI outputs and independently
report any local tower witness and whole-output result. CLI comparisons must
supply the same profile/catalog inputs, using existing adapters only.

Every case must retain input/config/profile hashes, exact tested Ares and Native
source/payload identities, official/seeded mode, commands and original complete
artifacts, plus the first difference with byte/line/column and surrounding text.
An unavailable or unidentified Native provider means comparison not executed, not a
byte mismatch or a pass. If Native does not reach the intended branch or an
unrelated earlier difference dominates, report the blocker before production
changes; do not weaken an assertion or widen this rewrite slice to remove it.
