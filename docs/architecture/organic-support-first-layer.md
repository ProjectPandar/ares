# Organic support at the build plate

## Upstream boundary

In OrcaSlicer 2.5.0 (`40ecc66f`), `Support/TreeSupport3D.cpp:3612–3635`
seeds influence areas, propagates them downwards, places the branch nodes, and draws
Organic branches. `:2490–2584` populates the per-layer influence areas;
`:3827–3832` smooths the linked nodes before meshing. `:3960–3995` slices each branch
between nominal layer planes and clips it to the collision and bed polygons.
`:4270–4305` combines the first layer's branch polygons, smooths and simplifies
its outline, subtracts contact regions, and stores it as a Base layer in
`intermediate_layers`.

`TreeSupport3D.cpp:3671–3677` builds the layer list and clips the stored polygons to
the bed. `Support/SupportCommon.cpp:1497–1543` installs a support layer into the
print object at its print Z; `TreeSupport3D.cpp:3700–3702` supplies the stored
areas to `generate_support_toolpaths`. `GCode.cpp:7015–7016,7114–7118` routes
`support_fills` to the support extruder; `:7695–7701,8094–8097,9011–9084`
turns those paths into G-code during `GCode::do_export` (`:2850,2941`). The
output contract is the ordered G-code, including the first extruded Support contour.

On a compound sloped plate without a raft, the K1 and X1C profiles disagree with
Orca on that contour. Identify which boundary first differs before changing the
Rust port in `project_slice/support_material/tree/organic`. Compare influence
areas, raw and clipped branch slices, stored Base, and the first toolpath input.
Use the MK4 profile as a control to separate geometry and placement from support
settings. The existing Rust implementation is the corresponding upstream rewrite
boundary, not an independent source of support geometry.

## Coverage and scope

Use identical synthetic project input and whole-file Native comparison on K1 and
X1C with support enabled. Preserve output for MK4. Check single-layer rafts as a
separate contact-layer case and recheck two- and three-layer rafts; their first
layer enters a different part of the support layer list. Do not assume these
cases share a cause. Changes to Slim/wrapped support and unrelated travel or
estimation differences are outside this scope.
