# Spiral vase object-start Z-only line — source-cited spec

Status: SPEC approved (coordinator ACK of proposal r2, mechanism-level);
freezes the boundary for the tests-only RED, then a bounded GREEN on the
W1 branch chain (base tip `2a20c7dff`). Lifts the single-object part of
the BBL spiral emission deferral recorded in
`spiral-vase-admission-spec.md`.

## Witness (discovery on main `28539c7fb`, BBL A1 single-object spiral)

`a1-single` — A1, one object (`cube_with_hole.obj`), `spiral_mode = 1`,
by layer (input sha `a8d0ed5f032f22a3b8968c9c648f621b913323abebaa417077f359b112b0ae38`,
the committed `tests/parity/spiral-vase-admission/single-object-by-layer.3mf`):

- Native (official and seeded, fresh binaries): exit 0; the complete
  unweakened comparator FAILs at **byte 81880 (line 2523, column 1;
  expected 116362 bytes, actual 115892 bytes)** against Ares.
- Whole-file diff anatomy: apart from the normalized generator header,
  exactly **47 Native-only lines** — each a bare `G1 Z<z>` (no F, no XY),
  `z` stepping `0.600 → 9.800` by 0.2, each immediately after an `M624`
  object-start marker on layers whose object start needs no XY travel
  (single closed loop: layer N−1's last point equals layer N's first
  point). Everything else byte-identical, including the spiral
  continuous-Z extrusions (`G1 Z0.648 X…`), the rewritten layer-change
  `G1 Z0.600 F42000`, the hop triples on traveling layers, and all
  comments.

Discriminating probe: the same scene with `z_hop = 0`
(sha `27501792b3793d29974f773defb264d1de8342c87ebf8d7cc2189ef4ee8cd935`)
diverges identically (byte 81702, same class) — the producer is
hop-independent.

Mechanism probe (`a1-single-nospiral`, spiral off, Native-only): the same
writer machinery emits per layer `G17` / `G3 Z<z+hop> I J P1 F42000` /
`G1 X… Y… Z<z+hop>` / bare `G1 Z<z>` at the object start — the
object-start Z-only line class is writer-level, not spiral-inserted.

Secondary witness of the same class: X1C single-object spiral byte 72286
(`G1 Z0.400`), recorded in the BBL timelapse discovery.

## Upstream boundary (pin `8500fcdccaa10b5099ac20d252af3a7c560046f1`)

Why a Z-only line is written with no XY displacement:

1. **Set**: `GCode::change_layer` sets `m_need_change_layer_lift_z = true`
   every layer (`GCode.cpp:5708`), after the spiral-mode layer-change Z
   (`:5701-5706`, `m_writer.travel_to_z(z, "move to next layer (N)")`).
   That emission does not clear the flag.
2. **Consume + emit**: `GCode::_extrude`'s travel guard
   (`GCode.cpp:6369-6384`). Upstream's comment at `:6367-6368` names the
   zero-XY case: *"Add m_need_change_layer_lift_z when change_layer in
   case of no lift if m_last_pos is equal to path.first_point() by
   chance"*. The guard fires on the flag alone; `travel_to(first_point)`
   (`:7350-7500`) emits no move for a zero-length travel (`travel.size()
   >= 2` false) but flushes the pending `; start printing object` +
   `M624` labels (`add_object_change_labels`); then
   `writer().travel_to_z(m_nominal_z, comment, /*force=*/true)`
   (`:6384`) bypasses `will_move_z` (`GCodeWriter.cpp:811-848`,
   `:916-929`) and emits `G1 Z<print_z> F<travel_speed_z×60>` **even
   though `m_pos.z() == m_nominal_z`**. The flag clears here (`:6385`;
   other clear-site: `travel_to`'s 2-point path, `:7483-7486`).
3. **Rewrite**: the pipeline `generator & spiral_mode & cooling &
   fan_mover & output` (`GCode.cpp:3748-3751`; per-layer
   `spiral_vase_enable` at `:4599-4615`, applied `:3698-3705`).
   `SpiralVase::process_layer` (`SpiralVase.cpp:86-141`): base
   `z =` first Z-only `new_Z` − Σ `dist_Z` of the layer's Z-only
   non-extruding G1s (= previous layer's print Z; layer 4: 0.800 − 0.200
   = 0.600); `:131` filters retracting/short-extruding lines; `:136-141`
   **keeps every remaining Z-only G1** and rewrites its Z via
   `GCodeReader::GCodeLine::set` (`GCodeReader.cpp:323-357`) — fixed
   3-decimal in-place splice (`0.600`) preserving the rest of the raw
   line. Both the layer-change Z and the object-start Z are rewritten.
4. **Final form**: layer-change line keeps `F42000` (feed differs from
   the layer-local current); the object-start line ends bare through the
   CoolingBuffer duplicate-F collapse family (`CoolingBuffer.cpp:823`,
   `:901-960`) — the same collapse producing the bare unlift `G1 Z.6`
   lines Ares already matches byte-exactly.

State at the first diff layer (layer 4, print_z 0.800): writer
`m_pos.z = 0.800`, `m_nominal_z = 0.800`, `m_need_change_layer_lift_z =
true`, `m_last_pos == first_point`, spiral base z = 0.600. Layer 5
identically: 1.0/1.0/0.800.

## Ares destination boundary

- `crates/ares-core/src/gcode_spiral_vase/project.rs::append_layer_z` —
  primary fix: remove the `if !already_emitted` dedup; port
  `SpiralVase.cpp:136-141` literally (rewrite and keep **each** Z-only
  G1 of the layer).
- `crates/ares-core/src/project_slice/gcode_emit/motion/path/start_travel.rs:148-157`
  — the writer-side forced-Z branch (`!needs_travel &&
  layer_change_travel`) already exists; touch it only if the RED bytes
  prove the line is still missing after the filter fix — and then only
  to align the flag lifetime with `GCode.cpp:5708`/`:6367-6385`
  (cleared only at the `_extrude` guard / `travel_to`'s 2-point path)
  and the after-`M624` line order.
- **Caution (review condition)**: the dedup may exist to mask
  over-emission of Z-only lines elsewhere. GREEN must run every
  `spiral_*` parity case in both Native modes; any regression must be
  root-caused to the real producer — restoring the dedup is forbidden.
- Forbidden: any "if spiral, insert `G1 Z`" special-case outside the two
  mechanism sites.

Not the queued `motion/scan.rs`, `format.rs`, brim/*, jerk,
seam_placement/alignment, `config_export/value.rs` (W6),
`fill_entities/grid.rs`; not the W1-touched
`options/config_export/writer.rs`, `gcode_emit/motion/state.rs`,
`gcode_emit/layer_gcode.rs`.

## Included

- The 47-line object-start Z-only class: exact placement (after the
  `M624` marker), value (spiral base z), formatting (3-decimal set
  splice; final bare F-form).
- Guards: currently byte-identical content — the spiral continuous-Z
  extrusions, the rewritten layer-change `G1 Z F` line, hop triples on
  traveling layers — must stay byte-identical.

## Deferred

- The two-object by-object-apart divergence (`; total layer number`
  83 vs 82) — separate class, separate analysis.
- `spiral_mode_smooth` XY interpolation beyond existing coverage.
- Non-BBL spiral re-verification beyond existing passing fixtures.

## RED plan (tests-only, on `2a20c7dff`)

`crates/ares-cli/tests/orca_parity/spiral_vase_object_start_z.rs`,
registered in the `case_support` grouped module (root `orca_parity.rs`
untouched); fixtures under `tests/parity/spiral-vase-object-start-z/`
(the z_hop=0 variant; the witness reuses the admission dir's committed
`single-object-by-layer.3mf`):

| Case | Expectation |
|---|---|
| `a1-single` (witness) | Native double replay deterministic; RED FAILs at the Ares comparison, first difference byte 81880 (line 2523: expected `G1 Z0.600`, actual `; FEATURE: Outer wall`) |
| `a1-single-zhop0` | Same class, first difference byte 81702 |
| native preflight | dual-replay determinism qualification for both cases (passes on RED) |

Runtime Native in both official and seeded modes via the canonical
control set; complete unweakened comparator (`check` asserts API and CLI
byte equality); no Native artifacts committed.

## Ways it could fail (checked by the cases)

- Rewriting only the first Z-only line per layer (the current dedup) or
  dropping the layer-change Z rewrite; wrong base z (print z instead of
  base, off-by-one layer); wrong decimals (writer-trimmed `.6` instead
  of the set-splice `0.600`); wrong placement (before the `M624` marker)
  or wrong F-form; regressions on traveling layers or the admission
  guards; over-emission on non-spiral layers (the masked-producer
  scenario the review condition warns about).

## Gates

| Stage | Gates |
|---|---|
| RED | the three cases above, official + seeded: witness and z_hop=0 fail only at the Ares byte comparison (first difference byte 81880/81702); preflight passes; `cargo fmt`; strict clippy; directed nextest selection |
| GREEN | both cases byte-exact in both modes; every `spiral_*` parity case still green in both modes (regressions root-caused, dedup stays removed); `cargo fmt`; strict clippy; no FULL without COORD |
