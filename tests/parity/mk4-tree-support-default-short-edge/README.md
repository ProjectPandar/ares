# MK4 organic default support short-edge fixtures

Stock Prusa MK4 0.4-mm overhang project with the established support family,
`tree(auto)`, default (Organic) style, wall count 0, support width 0.42 mm,
and interface top layers 4. Frozen from the single-key support matrix and
its strict one-key control; geometry and all other configuration are equal.

- `default.3mf` — `support_base_pattern=default`; SHA-256
  `13b069f79fb3aebdddf9fd077298e80fed821247d1ee9bd855e35178bb058964`.
- `rectilinear-control.3mf` — only `support_base_pattern=rectilinear`; SHA-256
  `339351a56db7c4df91e5f13aa2d358442ae50d6085e1ffa2a485e91e86ce0e00`.

Upstream `SupportCommon.cpp:1791-1801` routes Organic default support to
`tree_supports_generate_paths`. Its shrink helper (`:486-570`) sets
`ShortestEdgeLength=double(delta*0.005)` from the positive function width
for both outer contour and holes (`:508,:531`), negating only the outer
Execute delta. The Ares signed closure delta disabled outer short-edge
filtering and also multiplied in f32 rather than double. Neutral dual-side
probes traced the first emitted difference to the anchor on print_z 2;
travel Z2.4 is its lifted height.

Both cases use `native_stream_case::check` for Native double-replay and
public API/CLI complete ordered byte comparison, normalizing only the
generator line. No Native output or probe is committed.
