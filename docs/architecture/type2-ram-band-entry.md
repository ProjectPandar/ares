# Type2 ram-band entry and preheat timing

## Owning upstream boundary

Port the following OrcaSlicer2.5.0 `libslic3r/GCode/WipeTower2` entry behavior:

- `WipeTower2.hpp:341–347`, `toolchange_entry_pos`: offset the tower-local entry
  to the first wipe row beyond a positive planned ram band for non-SEMM gap walls.
- `WipeTower2.cpp::tool_change`,1330–1390: pass the selected planned ram depth
  and first-layer state to that entry calculation.
- `WipeTower2.cpp::compute_wall_skip_points`,2453–2469: use the same entry
  calculation and each plan layer's first-layer state for the wall opening.
- `toolchange_Unload`,1498–1511: for a boundary wipe start, move inward at the
  entry Y before climbing to the ram start; exclude this boundary case from
  ordinary continuation from the preceding sparse infill row.

Rust destinations are `wipe_tower/generator.rs::toolchange_entry_pos`,
`tool_change.rs::tool_change`, `generate.rs::compute_wall_skip_points`, and
`unload.rs::unload`. These remain private parts of the platform-neutral
`WipeTower2` rewrite, not a new Ares planner or a public API extension.

The per-tool ram reservation, positive-depth restart and quantized reserved-row
completion are prerequisites. Their geometry and coefficients are unchanged.

## Causal seam

The quantized-row completion restores the first simulated ram row endpoint but
leaves the first full-stream difference at the preheat insertion: the same
`M104 S265 T1` is still one motion line earlier than Native. The upstream
`GCodeProcessor::ExportLines::insert_lines` backtraces from the future T1 elapsed
time, so later tower-entry motion can affect that earlier output position.

The retained source-qualified trace still enters unload at `(0.25,0.25)` in
Rust versus `(0.25,2.0)` in Native. Native moves inward through `(0.75,2.0)`
before the common ram start `(0.75,1.0)`. Rust also erroneously executes the
ordinary sparse-row continuation in this boundary case. These are real input
motion differences before T1, not reasons to alter the preheat window or move
an emitted M104 by a fixed number of lines.

## Included behavior

1. Pass the existing selected change's ram depth alongside its wipe volume and
   required depth; retain zero depth for the existing unload-only state.
2. Apply the existing `wipe_start_offset_after_ram` formula only under the
   upstream non-SEMM/gap-wall/positive-depth guard.
3. Compute wall skips from the same function, ram depth and plan-layer flag.
4. Mirror the boundary inward travel and exclude boundary starts from ordinary
   sparse-row phase alignment. Preserve other unload branches.

No configuration, temperature value, preheat consumer, float precision, cooling,
role restoration, footer or comparator change is included. If an unexplained
preheat difference remains after the input alignment, retain it rather than
compensating its output location.

## Test-first verification

Commit this specification and run the existing two public TiQ2 E2E witnesses
RED on the prerequisite production tree. Then port the entry behavior. Keep
fixture bytes, whole-file line order and line-count assertions, marker/config/
footer assertions, and generated-by-only normalization unchanged.

Repeat the same110-identity tower/TiQ2/SEMM/Type2/BBL scope in both runtime modes;
require no passing-to-failing transition. Report the original two tests and the
first complete-stream mismatch without inventing a comparison rule. Qualify
read-only GDB streams and inspect the entry/inward/ram states; record all75
cooling handoff IDs only as downstream evidence. Run formatting, workspace
all-targets clippy and the core wasm32 check. No Native/probe rebuild is needed.
