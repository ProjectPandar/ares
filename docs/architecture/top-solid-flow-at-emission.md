# Resolve top-solid infill flow at emission

## Native boundary

At Orca 2.5.0 revision `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`,
`Fill/Fill.cpp:254–329` declares `SurfaceFillParams` without a top-flow
ratio field. Neither its ordering (`:330–372`) nor its equality
(`:374–404`) compares the option. `group_fills` uses the ordered set
(`:909–915,1127–1131`) and keeps the first region/surface owner of the
coalesced fill (`:1135–1151`). The projected geometry flow and role
speed (`:1067–1083`) do not hide the ratio: `LayerRegion.cpp:48–86`
and `PrintRegion.cpp:33–62` construct that flow from geometry/nozzle
and bridge inputs, not `top_solid_infill_flow_ratio`. Equal geometry and
speed with different top ratios consequently do not split a fill group.

`GCode.cpp:9475–9485` starts from geometric `path.mm3_per_mm`, multiplies
by print flow, then filament flow, then by
`NOZZLE_CONFIG(top_solid_infill_flow_ratio)` for `erTopSolidInfill`.
The selector uses the active filament/layer's process column
(`GCode.cpp:5083–5087,2185`; `Print.cpp:5113–5184`).

## Rust port

Remove the top-only physical column read from
`project_slice/group_fills/params/projection.rs`. Ordinary and concentric
fill entities must carry unscaled top-role path volume to export; keep
existing bottom and other role behavior. Leave grouping and first-owner
selection intact. Use active emission region settings and the already
published process nozzle column in `gcode_emit/motion/options/build`.
In `motion/extrusion.rs`, multiply geometric volume by print flow, then
filament flow, then the top ratio for `TopSolidInfill`, without folding or
reordering the factors. Do not move nozzle assignment publication earlier
or introduce a projection fallback. Keep filament flow on its own index;
other roles and their multiplication phases are outside this port.

## Verification

Use the public Native, byte-in API, and CLI export paths and compare
complete ordered output. Derive a stock H2D input with physical map 2,
unmatched process IDs, and unequal top-flow values; verify that finalized
config agrees and that top paths actually extrude. Include an equal-value
and singleton control and retain the existing local/global modifier and
coalescing tests in `runner/fill_group_flow_ratio.rs`. Preserve the
fixture bytes in the repository. The paired official and seeded
inventories must use identical test IDs, with no exclusions and no
pass-to-fail transitions.
