# Type1 nozzle-change perimeter width

OrcaSlicer `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`, `src/libslic3r/GCode/WipeTower.cpp:51,2185–2190`: the Type1 tower's `set_extruder` resolves nozzle-change line width by exact `float` key: 0.2 → 0.5, 0.4 → 1.0, 0.6 → 1.2, 0.8 → 1.4 mm. For a nozzle outside the table use twice the already resolved ordinary perimeter width. Ordinary width stays nozzle diameter × 1.25. Keep the existing diameter selection and use the same lookup as `wipe_tower/estimate/statics.rs`.

## Upstream use and Rust boundary

All references to `m_nozzle_change_perimeter_width` in `WipeTower.cpp` and `WipeTower.hpp` map to the following Type1 boundaries in `crates/ares-core/src/project_slice/gcode_emit/wipe_tower`:

| Upstream | Rust and scope |
|---|---|
| `WipeTower.hpp:567`, member initializer; `WipeTower.cpp:2185–2190`, assignment | `type1/plan.rs::Plan::new` stores the resolved nozzle width alongside the existing ordinary width. |
| `WipeTower.hpp:350`, `nozzle_change_extrusion_flow`; `WipeTower.cpp:3122,3184,3658,3700,3730,3742` | `type1/plan.rs::nozzle_lines` and `type1/nozzle_change.rs::change_nozzle` substitute the resolved width in their flow calculations, including the 0.2 mm bridge-height flow. |
| `WipeTower.cpp:3111–3132`, `set_toolchange`; `:3181–3192`, `plan_toolchange` | `type1/plan.rs::replan_changes` uses tower width minus (nozzle width + ordinary width), `Plan::add` uses tower width minus twice nozzle width. Preserve separate openings and use the nozzle width for initial row pitch. |
| `WipeTower.cpp:4730–4758`, `calc_block_infill_gap`; `:5535–5543`, `get_block_gap_width` | `type1/plan.rs::CategoryDepth::gap` uses the resolved nozzle width as the nozzle gap base and twice that width as the alignment threshold; `replan_changes` uses it when the old tool has no category gap. Ordinary gaps and category order do not change. |
| `WipeTower.cpp:3636,3659,3673–3674,3704–3707,3740`, active nozzle emission | `type1/nozzle_change.rs::change_nozzle` uses the width for the writer, offsets, pitch, solid rows, and endpoint checks. The existing Rust category and row-count algorithms are not replaced by this lookup. |
| `WipeTower.cpp:4842,4847`, first-layer `plan_tower_new` adjustment | `type1/plan.rs::apply_spacing` already scales depth and recalculates ordinary wipe length; it does not store a separate nozzle-change length. The Rust emitter derives its nozzle count from depth in `change_nozzle` and `tool_change`. Changing that representation or porting the separate length reconstruction is outside this width lookup. |
| `WipeTower.cpp:2464,2466`, older `nozzle_change`, and `:3565,3567,3576`, reverse-travel block | These upstream blocks are compiled out. Do not enable them. The existing `type1/nozzle_change.rs` and `type1/tool_change.rs` count and reverse-travel calculations still consume the same resolved nozzle width rather than doubling the ordinary width. |

This extends [Type1 category replanning](type1-tower-category-gap.md) without changing its stages. Do not change Type2, physical nozzle mapping, category selection, flushing, or unrelated Type1 trajectory and template behavior.

## Public verification

At runtime derive a two-color Type1 tower project from stock Bambu Lab X2D 0.6 mm presets and synthetic geometry, assigning the two logical filaments to different physical heads. Require Native to emit an actual nozzle-change section with different old and new nozzle IDs, and compare emitted nozzle-change extrusion from the API and CLI against Native. The 0.6 mm input distinguishes a 1.2 mm nozzle-change width from the 1.5 mm result of doubling the ordinary perimeter. Compare the same Type1 inventory before and after in both Native modes, including the existing 0.4 mm planned-footprint controls. The targeted emission check does not claim complete Type1 stream parity where other generation behavior differs.
