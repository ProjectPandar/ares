# By-object spiral per-object transition_out — source-cited spec

Freezes the boundary for a tests-only RED, then a bounded GREEN.

## Witness (BBL A1 by-object spiral)

`byobject-apart` — A1, `overhang.obj` printed first, then
`cube_with_hole.obj`, `spiral_mode = 1`, `print_sequence = "by object"`
(input sha `2dbf797da35adf51f085c397cfa959364fd7411a73ef52ec687d04dd480e7342`,
the committed `tests/parity/spiral-vase-admission/two-object-by-object-apart.3mf`):

- Native (official + seeded) and Ares both exit 0; after generator-line
  normalization the complete comparison's first difference is **line 5:
  `; total layer number: 83` (Native) vs `82` (Ares)** — identical in
  both modes (169868 vs 169138 bytes).
- Native: 83 layer blocks = 51 cube + **32 overhang**; Ares: 82 = 51 +
  31. The missing block is `overhang.obj`'s top-layer **flat taper
  pass**: after the ramp block (`G1 Z6.000` base sync → `G1 F…` →
  `G1 Z6.001…6.200 X… E…`), Native re-emits the same layer
  (`layer num 31/81`) flat — no layer-change Z, walls
  `E0.02795 / E1.10666 / E0.64961 / E0.34494`, `M73 P71`, closing
  `G1 X118.562 Y152.79 E0.00000` — then the end-of-object sequence
  (retract, WIPE, `G1 Z6.6`, `M204 S10000`, `G1 Z7`, stop label,
  travel `G1 X50 Y40 Z6.2 F42000`). Ares transitions to cube directly
  (`G1 Z7 F42000` / `G1 X50 Y40 Z7 F42000`); filament totals follow
  (571.86 mm / 1.38 cm3 vs 569.73 / 1.37).
- The plate-final object (cube) emits ramp + flat taper byte-identically
  on both sides — the gap is specific to non-final objects' own final
  layers.

## Upstream mechanism (pin `8500fcdccaa10b5099ac20d252af3a7c560046f1`)

- `SpiralVase.cpp:120-121`: `transition_out = last_layer &&
  use_relative_e_distances`. `:154-165`: on the last layer every
  extruding line is emitted twice — the ramped line into `new_gcode`,
  plus a cloned flat line with `E × finishing_e_factor`
  (`finishing_flowrate + (1-factor)*(1-finishing_flowrate)`) into
  `transition_gcode`; non-G1 lines dual-push (`:205-208`); return
  `new_gcode + transition_gcode` (`:216`). A1
  `spiral_finishing_flow_ratio = 0` → closing `E0.00000`.
- **Per-batch `last_layer`**: under by-object (sequential) printing each
  object is exported through the single-object pipeline
  (`GCode.cpp:3772+`), whose filter computes `bool last_layer =
  in.layer_id == layers_to_print.size() - 1` (`GCode.cpp:3805-3806`) —
  **true for every object's own final layer**. The by-layer pipeline
  (`GCode.cpp:3698-3705`) computes the same expression over the single
  whole-plate batch; under spiral, by-layer multi-copy is
  admission-rejected (`Print.cpp:1334-1341`), so there per-object-final
  ≡ plate-final.

## Ares boundary

### `is_last_entry` consumer enumeration (review requirement)

| # | Site | Role | Upstream correspondence | GREEN action |
|---|---|---|---|---|
| 1 | `layers/schedule.rs:285-287` | computes `last_entry` = plate-final entry of the merged schedule | none (Ares-only plumbing feeding #7) | becomes dead → removed |
| 2 | `layers/schedule.rs:329` → `layers.rs:134` → `layers/tool_entry.rs:40,73,287` → `layers/drive.rs:315` → `layers/object_entry.rs:36` | pass-through chain for `last_entry` | none (plumbing) | becomes dead → removed |
| 3 | `layers/object_entry.rs:125` | `is_last_entry = Some((object_index, layer_index)) == last_entry` | none (plumbing predicate) | becomes dead → removed |
| 4 | `layers/object_entry.rs:154` — **the only consumption**: `final_layer: is_last_entry && layer_index + 1 == object_layer_counts[object_index]` | gates the spiral `transition_out` clone stage | `GCode.cpp:3805-3806` (per-object-batch `last_layer`, by-object) / `:3698-3705` (per-batch over the single plate batch, by-layer; per-object-final ≡ plate-final under spiral) → `SpiralVase.cpp:120-121` | **changed** to `final_layer: layer_index + 1 == object_layer_counts[object_index]` |

No other consumer exists (`final_layer` readers are the spiral
machinery only: `ProjectSpiralVaseLayer.final_layer` →
`gcode_spiral_vase.rs::layer_state` → `SpiralVaseTransitionOutState`;
inert when spiral is disabled). The legacy `gcode.rs:127` path already
computes `final_layer = layer_index + 1 == layers.len()` per batch —
consistent with upstream, untouched.

### Fix shape

`layers/object_entry.rs:154`: drop the `is_last_entry` conjunct. The
per-object expression alone mirrors `GCode.cpp:3805-3806` under
by-object and degenerates correctly under by-layer spiral (single
object by admission). The change orphans the whole `last_entry`
plumbing (#1–#3), which is removed with it — no compatibility shims.

The header `; total layer number` (83 vs 82) counts the layer blocks
and is a symptom; it corrects itself once the pass exists.

## Included

- Per-object `final_layer` under by-object sequential printing: the
  `transition_out` flat taper pass for every object's own final layer
  (here `overhang.obj`'s), with its downstream end-of-object sequence
  and the header layer count.

## Deferred (separate classes found in the same scene)

- M73 P-progress placement (4 sites, first at line 1437: Native inserts
  the progress line one extrusion earlier).
- Final-layer ramp feed (line 3374: Native `G1 F3713` — no overhang
  slow-down on the transition layer — vs Ares `G1 F1857`; the only 2×
  feed divergence).
- Non-BBL re-verification beyond existing fixtures.

## RED plan (tests-only)

`crates/ares-cli/tests/orca_parity/spiral_byobject_transition.rs`,
registered in the `case_support` grouped module (root `orca_parity.rs`
untouched); no new fixture — the witness reuses the admission slice's
committed `two-object-by-object-apart.3mf`:

| Case | Expectation |
|---|---|
| `byobject-apart` (witness) | Native double replay deterministic; RED FAILs at the Ares comparison — first difference line 5, `; total layer number: 83` vs `82` |
| native preflight | dual-replay determinism qualification (passes on RED) |

Runtime Native in both official and seeded modes via the canonical
control set; complete unweakened comparator; no Native artifacts
committed.

## Ways it could fail (checked by the cases)

- Firing transition_out on non-final layers or on non-spiral scenes
  (regression via the existing `spiral_*` suite); wrong taper factor
  (E values of the flat pass); missing dual-push of the header lines
  (`layer num`, M73 L, labels) into the transition block; wrong
  end-of-object sequence after the pass (retract/wipe/lift/travel Z);
  the header count staying 82; the legacy `gcode.rs` path regressing.

## Gates

| Stage | Gates |
|---|---|
| RED | the two cases above, official + seeded: witness fails only at the line-5 comparison; preflight passes; `cargo fmt`; strict clippy; directed nextest selection |
| GREEN | witness byte-exact both modes; every `spiral_*` parity case green in both modes (the plumbing removal must not disturb the schedule); `cargo fmt`; strict clippy |
