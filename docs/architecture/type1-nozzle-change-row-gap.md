# Type1 nozzle-change row spacing

## Upstream and destination

OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`),
`src/libslic3r/GCode/WipeTower.cpp`:

- `4690–4769`, `calc_block_infill_gap`, computes the two pitches for each
  filament adhesion category. `3107–3132`, `set_toolchange`, uses the old
  tool's nozzle pitch when reserving nozzle-change depth and stores the
  ramming length separately from the wipe length. `4832–4854` resets the
  first plan entry's ramming length for solid spacing while scaling the
  reserved depths.
- `3635–3644`, `ramming`, takes the first planned change matching the
  destination tool and rounds ramming length divided by the cleaning-box
  width up to a row count. The upper Y bound uses the planned nozzle depth
  directly (`3674–3676`); for solid infill the row count is rounded down
  from the box depth plus the half-width difference (`3704–3706`).
- `3673`, `ramming`, uses nozzle perimeter width on the first tower layer
  and `3704` also uses it for solid infill. Otherwise the row step is the layer's
  `extra_spacing` multiplied by `get_block_gap_width(m_current_tool, true)`
  (`3673`, `3704`). The getter at `5535–5543` resolves the old tool's nozzle
  gap, not its wipe gap or the new tool's gap. The connector at `3746`
  extrudes by that step. Reverse travel is decided separately at `3763`;
  there is no global material override of the extrusion step.

Carry the resolved nozzle pitch from the existing `replan_changes` in
`crates/ares-core/src/project_slice/gcode_emit/wipe_tower/type1/plan.rs` to
`Generator::change_nozzle` in `type1/nozzle_change.rs`. The existing
width-based calculation is a temporary substitute for that handoff. Keep
row spacing independent of the ramming length used to count rows; look up
the gap for the current tool independently of the destination match. Reuse
category planning, including its reverse-travel gap selection, without
recomputing it in the emitter. Tower footprint planning, the nozzle
perimeter lookup, cursor accounting, cooling and reverse-travel motion are
separate.

## Public behavior

Use a single 2 × 2 × 0.4 mm cuboid with the H2D 0.4 mm presets and a
second PLA slot on the top surface mapped to the other nozzle. In the
second layer, measure the two nozzle-change extrusion connectors and
confirm their lengths against the spacing of the three horizontal rows.
At `prime_tower_infill_gap=150%`, OrcaSlicer emits a 1.25 mm pitch. With
only that setting changed to `100%`, its pitch is 1.0 mm and an additional
row fits. Compare those pitches and row counts across the Native,
public API and CLI output for each input. This is a nozzle-change
substream check, not whole-file G-code parity.
