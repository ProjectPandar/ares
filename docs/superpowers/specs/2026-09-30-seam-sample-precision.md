# Seam sample scalar precision

## Upstream boundary and destination

Rewrite `OrcaSlicer/src/libslic3r/TriangleSetSampling.cpp:49-70`,
`sample_its_uniform_parallel`'s sampled position construction, at
`ares-core/src/project_slice/seam_placement/sampling.rs::weighted`.
Eigen converts double barycentric weights to its Vec3f scalar before
multiplication. Preserve this narrowing boundary rather than multiplying
each vertex coordinate in double and narrowing the product. The existing seam planner remains a
temporary compatibility shell around upstream SeamPlacer concepts.

The adjacent included boundary is
`GCode/SeamPlacer.cpp:341-345::GlobalModelInfo::calculate_point_visibility`,
at `seam_placement/visibility.rs::GlobalVisibility::at`: evaluate both
distance contributions separately before adding the weight.

## Reproduction and scope

WEMAKE3D TinyBotV1's strict complete-output smoke fails. Although its first
difference is an M73 marker, its only non-timing movement difference is the
first layer's seam corner and consequent wall/travel sequence. A diagnostic
native build chooses the same first-layer corner as the official binary;
its unrelated speed differences disqualify it as an acceptance reference.

Included: sampled-position scalar conversion, visibility-weight grouping and the complete-output
printer regression. Deferred: other raycasting, sampling, negative volumes,
seam alignment behavior, time processor behavior, remaining printer/option
coverage and untouched-default-profile parity. No timing compensation,
epsilon tie-breaker or weakened output comparator is permitted.

## Acceptance

The public 3MF-to-complete-G-code comparison must fail before and pass after
against the selected official Orca 2.4.2 executable. Recheck related seam
option values and printer cases. Run fmt, Clippy, workspace Nextest and Tier 1
compile checks; independent six-aspect review must approve after any repairs.
Commit and push code, tests and evidence documentation together.
