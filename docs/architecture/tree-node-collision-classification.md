# Tree-node build-plate classification uses collision clearance

## Upstream source and Rust destination

OrcaSlicer `40ecc66f7`,
`src/libslic3r/Support/TreeSupport.cpp:3359-3371::TreeSupport::drop_nodes`,
contains the change from `414857c9d48a1c4b2ee2c709a801e0562e8959b5`.

The Rust destination is
`ares-core::project_slice::support_material::tree::non_organic::dropping::DropPass::drop_circle`.
Its existing node arena and non-organic support adapter remain temporary
compatibility shells around the upstream `TreeSupport` data and operations.
This slice does not introduce a separate support-generation pipeline.

## Included behavior

Classify a next-layer node as reaching the build plate only when its position
is outside `get_collision(0, obj_layer_nr_next)`. Reuse the same zero-radius
collision polygons already used for the next node's outward projection and
radius clamping. Do not classify against the raw layer outlines: the collision
polygons include the configured XY clearance.

Keep the upstream polygon containment boundary convention. Preserve node
creation order, parent links, roof distance, movement limits and radius
calculation. Organic supports, belt-floor termination and other dropping or
merging branches are deferred.

## Failure modes and behavioral verification

A point outside the raw outline but inside the clearance envelope must not
be marked as reaching the build plate. Incorrect classification retains
branches that should terminate near the object and changes the support
footprint and skirt. Points outside both shapes retain their classification.

Use the public project-to-G-code comparisons for the seven painted/manual
support projects and the three single-thread retraction projects. The painted
slim case must retain support extrusion and the runtime Native footprint.
Compare complete G-code output, including skirt, support and retraction
commands, against OrcaSlicer in both reference modes. Keep Native-generated
outputs external rather than embedding output files in the repository.
