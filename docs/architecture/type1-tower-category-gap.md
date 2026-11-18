# Type1 wipe tower category gaps and depth replanning

## Upstream and destination

OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`),
`src/libslic3r/GCode/WipeTower.cpp`:

- `3136–3195`, `plan_toolchange`: reserve the initial nozzle-change depth
  with the nozzle perimeter as the pitch and the opening equal to tower width
  minus twice that perimeter.
- `4690–4769`, `calc_block_infill_gap`: collect wipe and nozzle-change depths
  by filament adhesion category, carrying maxima down the plan. Select the
  category's pair of line gaps from its ramming/reverse-travel flags, the
  configured extra gap, and the minimum-depth ratio for non-rib walls.
  Replace each change's depths with the result from `set_toolchange` and
  reset extra spacing.
- `3107–3132`, `set_toolchange`, and `5535–5543`, `get_block_gap_width`:
  calculate wipe depth with the destination category's wipe pitch and
  nozzle-change depth with the source category's nozzle pitch. Use
  `ceil(nozzle_length / (tower_width - (nozzle_perimeter + perimeter)))`
  in the replan, not the initial reservation opening.
- `4781–4800`, `plan_tower_new`: build category blocks, align the new rib
  width, replan each change again using the saved category gaps, then rebuild
  the blocks. `3339–3343` selects the reverse-travel branch from configured
  ramming travel time and change length.

Replace the single-pitch depth update in
`crates/ares-core/src/project_slice/gcode_emit/wipe_tower/type1/plan.rs` with
this planner sequence. The block generator remains the consumer of the
resulting per-change depths. First-pass reservation and subsequent replans
have distinct openings. Retain the existing Type1 nozzle mapping boundary;
this does not port a new physical-head grouping algorithm.

## Public behavior and coverage

With the existing synthetic X2D dual-head project and only prime tower
activation changed, the first change crosses heads in category 100. The
0.5 mm wipe perimeter, 1 mm nozzle perimeter and 150% gap produce wipe and
nozzle pitches of 0.75 and 1.25 mm. The latter changes the first recalculated
nozzle depth from 2 to 2.5 mm and the total depth from 9.5 to 10 mm. This
propagates to the first-layer tower footprint and the exported bed-leveling
rectangle. For each input, compare the exported G29 rectangle and the
ordered first-layer outer-wall coordinates (including arc centers) before
the first CP tool change in Native, the public API and the CLI. These checks
cover the planned footprint, not whole-file parity. Generate each project
variant from the existing fixture without committing the derived project
or G-code.

A second derived input sets nominal tower width to 33 mm. The observed
62.8286743 mm nozzle-change length needs three lines in the 31 mm initial
opening but two lines in the 31.5 mm replan opening. At the category pitch
of 1.25 mm, the replanned depth is 2.5 mm, not 3.75 mm. This distinguishes
the opening change from the pitch change.

Wall polygon and gap processing, nozzle-change path emission, object-wipe
ordering and finish-block eligibility remain separate upstream boundaries.
No changes to Type2, first-layer hull construction or G29 rendering.
