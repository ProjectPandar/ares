# Wipe-tower toolchange layer-Z restoration

## Upstream boundary

Port OrcaSlicer `39408fa1f274c70a84ff16cfbe3279be1ab10ae9` (#15441),
using final source `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`:

| Upstream owner | Rust destination |
|---|---|
| `src/libslic3r/GCode.cpp:1262–1271,1625–1629`, `WipeTowerIntegration::append_tcr` | `gcode_emit/wipe_tower/type1/emit.rs` |
| `src/libslic3r/GCode.cpp:1773–1782,1941–1946`, `WipeTowerIntegration::append_tcr2` | `gcode_emit/wipe_tower/emit.rs` |
| `src/libslic3r/GCode/WipeTower.cpp:2725–2731`, `WipeTower::toolchange_Change` | `gcode_emit/wipe_tower/type1/tool_change.rs` |
| `src/libslic3r/GCode/WipeTower2.cpp:1738–1743`, `WipeTower2::toolchange_Change` | `gcode_emit/wipe_tower/tool_change.rs` |
| `tests/fff_print/test_wipe_tower.cpp:659`, tallest-printed-layer toolchange test | Native CLI versus public byte-in slicing API E2E tests |

All Rust destinations above are under
`crates/ares-core/src/project_slice/`. The existing emitter modules are temporary
compatibility shells for the named upstream integration methods, not owners of
an independently designed tower pipeline. Replace their missing restoration
and template-state behavior with the source operations below.

## Required behavior

Both generators place `[restore_layer_z_before_toolchange]` on its own
newline-terminated template line immediately before `[change_filament_gcode]`.
Type1 places it after `[filament_end_gcode]`. Both integrations always supply
the value, including the empty string when no lift is required. Preserve that
empty expansion and its line terminator; do not simulate it with padding or a
whitespace transformation.

A nonempty custom filament-change macro may travel. A physical extruder switch
may also travel through firmware Tn. Changing logical filament on the same
physical extruder without a macro does not satisfy that condition. Use the
existing selected-filament state and 1-based filament-to-physical-extruder map,
not logical filament inequality, to reproduce the upstream pointer/id test.

For Type1, lift only when the tower is compacted below the object,
`tool_may_travel` is true and the live writer Z is below `current_z - EPSILON`.
The source compacted condition is sparse-layer skipping, nonnegative tower Z
and `tcr.print_z - z > EPSILON` (`GCode.cpp:1176–1178`). For Type2, preserve the
upstream priming/needed-toolchange branch and require `will_go_down`,
`tool_may_travel` and `enable_prime_tower`. Do not replace these distinct guards
with a new common policy.

Generate the writer's raw `travel_to_z(current_z,
"Restore layer Z before toolchange")` operation before constructing/rendering
the toolchange. Update live writer Z at the same point, so the macro receives
the same state as upstream. Do not use the retract/travel/unretract wrapper for
this raw Z operation. The existing `motion::tower::writer_z` boundary represents
the owning `GCodeWriter` operation.

A lift must not make subsequent tower extrusion float at object height.
Preserve Type2's existing forced tower-Z restoration and the final return to
object height. Type1 requires the existing upstream compacted-height descent
in `append_tcr`'s start-filament suffix (`GCode.cpp:1615–1622`) after the macro,
return travel and unretraction. Where the Rust compatibility shell lacks that
prerequisite, connect this exact source operation rather than inventing a
second height policy. Preserve the source comment and newline behavior.

No new public API, target-specific behavior, filesystem dependency, option,
feature flag or compatibility fallback is needed. Other than sharing the exact
sparse-skipping eligibility predicate below, tower planning, dimensions,
ramming, temperature policy, synchronization and unrelated no-sparse-layer
travel optimizations remain outside this slice. Keep the preceding planner
boundary rewrite unchanged.

## Included signed layer-distance prerequisite

The height scenario also exposes an existing Rust translation mismatch in
`type1/finish_layer.rs`, not a change introduced by upstream `39408fa1f`.
`WipeTower.cpp:3063–3064` and `3957–3958` compute `int dist_to_1st` from the
iterator distance minus `m_first_layer_idx`, then subtract it from the minimum
of brim loops and chamfer loops. Preserve the subtraction's unsigned wrapping
before narrowing to the signed integer; do not subtract two Rust `usize`
values with checked arithmetic. Represent the absent first-layer index as the
source `size_t(-1)` sentinel. Its distance is therefore `idx + 1` before signed
narrowing, not zero and not an error. Failure modes are a layer before the first
change (negative signed distance), the first-change layer (zero), later layers
(positive), and the absent-index sentinel. Do not saturate or clamp the result.

## Included compacted nozzle-entry prerequisite

The Type1 two-nozzle cases also expose an existing Ares omission from
`82e91bd47` (already present upstream before `39408fa1f`). Port both branches
of `GCode.cpp:1231–1250` at `40ecc66f7`, not a new height policy. When the
compacted tower is below the object and `tcr.print_z` is above the live writer Z,
`compact_intower_nc_travel` makes the nozzle-start travel target the tower Z
directly. Otherwise, travel at the normal layer Z and, when compacted below the
object, use raw `travel_to_z(z)` before unretraction and nozzle-change extrusion.

The Rust boundary is Type1 `emit.rs` plus the existing tower-travel request's
Z target in `motion/tower.rs` and `motion/path/start_travel.rs`. Preserve the
normal nominal-Z target for all other callers. Keep this omission's correction
in its own production commit before the `39408fa1f` restoration change; do not
attribute it to `39408fa1f` or include the other `9ed545` behavior.

## Included sparse eligibility and model-side descent prerequisites

`GCode/WipeTower.cpp:53–57` owns `wipe_tower_sparse_layers_skipped`: the raw
no-sparse option must be enabled, timelapse must not be Smooth, and wrapping
detection must be disabled. `WipeTower` and `WipeTower2` constructors, and
`GCode.hpp:130`, share this predicate. The Rust destination is one predicate in
`gcode_emit/wipe_tower.rs`, reused by the existing plan, generator, integration
and Type1 compacted-height decisions. Preserve the raw exported option;
eligibility is not option normalization. The existing tower-presence predicates
in `wipe_tower::requested` and `tool_ordering/collect.rs` answer a different
question and cannot substitute for sparse-skipping eligibility.

Also include the omitted `82e91bd47` operation at `GCode.cpp:1164–1178` in
Type1 `emit.rs`. Defer the initial tower-Z descent when sparse layers are
skipped, the result is neither priming nor finish-first, and
`current_z - z > EPSILON`. Until the later tower travel, the nozzle is still
over the model. Do not change the captured `current_z` or manufacture a
restoration lift by prematurely descending there. Finish-first has already
travelled to the tower and retains the immediate descent. The subsequent
nozzle-entry and start-filament descents remain required after safe travel.

Type2 `append_tcr2` is distinct: final upstream `GCode.cpp:1753–1757` still
descends immediately on `will_go_down`, after its unconditional tower travel
for that condition. It has no Type1 finish-first/deferred-descent predicate.
Keep this ordering rather than copying Type1's guard into Type2.

### Predicate consumers and observable coverage

| Rust consumer | Upstream boundary | Smooth/wrapping observation and limit |
|---|---|---|
| `integration.rs::Integration::new` | `GCode.hpp:130`, integration's sparse-skipped state | Type1 and Type2 variants require tower extrusion at object Z rather than accumulated compacted Z. |
| `type1/emit.rs::append` | `GCode.cpp:1178,1231–1250,1262–1271` | Type1 variants require no compacted-layer restoration lifts; the normal compacted variant separately checks model-side end-filament Z. |
| `type1/plan.rs::Plan::new` | `WipeTower.cpp:1990`, constructor sparse-skipped state | Type1 variants exercise full-height planning and emission. Exact first-change index and first-layer footprint are not independently compared by the height observer. |
| `plan.rs::Plan::new` | `WipeTower2.cpp:1060`, constructor sparse-skipped state | Type2 variants exercise full-height planning and emission. Exact first-layer speed, cone shape and filament accounting are not independently compared by the height observer. |
| `type1/generator.rs::finish_result` | `WipeTower.cpp:3099–3102,3999–4002`, printed-layer filament accounting | The variants exercise generated sparse results, but exact per-filament consumed length is not independently asserted. Full output and its unchanged comparison result remain available; height success is not an accounting-parity claim. |

The [pre-slice compacted-tower clearance slice](compacted-wipe-tower-clearance.md)
ports the estimated check at `Print.cpp:1431–1478,2267–2275`. The post-plan
check using actual extrusions (`Print.cpp:1499–1584,3639`) remains deferred, as
does `WipeTower.cpp:79`'s sparse-layer combination, which has registry/profile
vocabulary only. There is also no corresponding incremental
`Print::invalidate_state_by_config_options` implementation for `Print.cpp:571`.
Do not change its raw option vocabulary in export, registry or profile-refresh
keys: the effective predicate controls behavior, not configuration identity.

## E2E contract and failure modes

Use the upstream test's eight combinations: Type1/Type2 × one/two physical
nozzles × nonempty/empty custom macro. The public seam is Native CLI and the
public slicing API receiving the same repeatable 3MF bytes. Build the source
scenario: 10×10×10 mm and 10×10×2 mm objects, 0.2 mm layers, filament 2 only on
top surfaces, tower at 30×30 and sparse tower layers skipped.

Add a Type1 two-nozzle case whose end-filament macro sets live writer Z above
the compacted tower. Establish from Native output that the conditional explicit
descent occurs before nozzle-change extrusion, then require the same Ares
observation. This exercises the other nozzle-entry branch without weakening any
of the eight original height contracts.

Add a single-physical-nozzle Type1 custom-macro scenario whose end-filament
macro exposes `{position[2]}`. Establish Native entries after object extrusion
above the short object without the finish-first tower travel; the end macro
must still see at least the highest printed object Z. This exposes an unsafe
early descent even when a later restoration makes the change macro safe.
Also run Smooth-timelapse and wrapping-detection variants with the raw no-sparse
option still enabled: Native must retain tower extrusion at object height and
emit no compacted-layer restoration lifts. Require the same public-API behavior.

Retain input, complete Native/Ares G-code, independent parsed height observations
and the unchanged whole-output comparator's result. First establish that Native
actually exercises toolchanges above the short object and compacted tower
extrusions, rather than letting an empty observation set satisfy a check.

- A macro or physical Tn must not start below the highest already extruded object
  layer after the short object's top.
- Empty macro plus one physical nozzle must not add restoration lifts merely
  because the logical filament changes.
- After a restoration, tower extrusion must still occur below the object's
  highest printed layer; a lift without the corresponding descent is wrong.
- Relative-E parsing, feature tags and XY extrusion must distinguish object
  extrusion, tower extrusion and unretraction without rewriting the output.
- Empty restoration values must still preserve exact source template lines.

Commit this source specification first, then the E2E tests and their failing
runs, then production changes. Run the original 48 whole-output cases, six
planner controls and existing Marlin control unchanged in both final-release
modes, alongside the eight new height cases and the elevated nozzle-entry case.
Enumerate each of the 47 preceding
failures with its before/after first difference: resolved restoration/empty
expansion versus an independently remaining cause. Keep failures visible;
never broaden a comparator or assign every byte to this commit by assumption.

Use locked/offline nextest, workspace/all-targets strict Clippy, rustfmt and an
ordinary browser-WASM core/adapter check. No full-suite baseline rerun, source
or profile repair, mask change, golden update or cache-key change is part of
this rewrite.
