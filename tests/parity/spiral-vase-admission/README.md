# Spiral vase admission fixtures

Cases for `docs/architecture/spiral-vase-admission-spec.md`
(`Print.cpp:1334-1341`). All three derive from
`tests/parity/sequential-template-layer-count/a1-bylayer.3mf`
(Bambu Lab A1, marlin, one filament, objects `overhang.obj` +
`cube_with_hole.obj`) with `Metadata/project_settings.config` edits; the
apart guard additionally moves the `cube_with_hole` build item
(`95.4375 119.505222` → `40 30`) so the by-object sequential clearance
passes on both sides.

- `two-object-by-layer.3mf` — witness: `spiral_mode=1`, two copies, by
  layer. Native rejects with -51/205 and the `spiral vase mode` diagnostic;
  Ares on the RED tip slices it (exit 0).
- `single-object-by-layer.3mf` — guard: the overhang object (3MF object 2)
  is removed, `spiral_mode=1`. Accepted by Native and Ares.
- `two-object-by-object-apart.3mf` — guard: `spiral_mode=1`,
  `print_sequence="by object"`, objects separated. Accepted by Native and
  Ares.

The guards assert acceptance only: accepted-scene BBL spiral bytes diverge
on the RED tip (single-object A1 first difference byte 81880, Native
`G1 Z0.600` layer step vs Ares continuous Z; single-object X1C byte 72286,
`G1 Z0.400`; two-object by-object-apart even differs on
`; total layer number`, 83 vs 82). Those emission findings are recorded in
the spec's deferred section and are out of this slice's scope.

`crates/ares-cli/tests/orca_parity/spiral_vase_admission.rs` compares Ares
with Native G-code made at runtime by the orca_parity runner (official and
seeded). No Native output is stored.
