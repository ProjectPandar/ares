# Aligned seam placement on a continuous loop-normal field

## Source boundary and Rust destination

This is a Rust rewrite slice of OrcaSlicer v2.5.0-beta
`345efec64c7b95a05ce5cf96df0dde66a158835c`,
`src/libslic3r/GCode/SeamPlacer.cpp`:

- `edge_normal` and `place_on_loop`, approximately1490-1627;
- the non-Enforced branch of `align_seam_points`,2058-2062: `t >= 1`
  keeps the current candidate; otherwise `place_on_loop` maps the angle-blended
  fitted target onto the loop before finalizing the seam.

The destination is
`project_slice/seam_placement/alignment/loop_normal.rs::place_on_loop`, called by
`alignment.rs::finalize_string`. The current free-space blended target is only a
temporary compatibility shell for this upstream final-position concept. Replace
that stored target with the upstream loop-normal solution, not with a new Ares
placement heuristic. The already separate immediate-lower-layer restart port is
not changed by this slice.

## Non-negotiable decisions

- Keep candidate selection, KD traversal, scores, string discovery/alternative
  starts, spline fitting, angle blending and ordinary loop geometry unchanged.
- For non-sharp fitted seams, use the source continuous normal-field solve:
  f64 positions widened from candidate f32 values; normals from the nearest
  distinct neighbors; linear blending of vertex normals on each edge; roots of
  `cross(target-P(u),N(u))=0` restricted to the source closed unit interval.
- Preserve the source two-front traversal, forward-side arc tie, closest-foot
  pruning bound, strict distance comparison, minus-root-before-plus-root order,
  and source coefficient-degeneracy threshold of1e-12. Do not tune epsilon,
  add a tie key, or rotate/sort the contour.
- Preserve upstream degenerate-loop/no-solution returns and zero vertex-normal
  handling. These are source behavior, not a legacy algorithm fallback.
- Keep the sharp-angle branch at the candidate, as upstream does. Return the
  mapped XY as f32 and retain the selected candidate's Z.
- The existing later `placement`/`split` projection is not a replacement for this
  normal mapping. Do not modify that projection, scaling, emission or seam-gap
  trimming to compensate for a different aligned target.
- The leaf is platform-neutral, private, allocation-free per mapping, and uses
  no filesystem, terminal, native viewer or public diagnostic interface.

## Included and deferred behavior

This slice originally covered the Neutral case of the non-Enforced
`place_on_loop` branch at2058-2062. Unpainted candidates still have unlimited
allowed arcs. The [facet-paint slice](seam-paint-spec.md) now imports the
unqualified triangle `paint_seam` attribute into `ProjectMesh::seam_facets`.
In its supported Aligned domain, `SeamCandidate` retains Enforced and Blocked
states, the enforced corner pull precedes mapping, and `allowed_arc` clamps a
mapping that has left both allowed directions to the closer endpoint. Inputs
outside the facet-paint slice's supported domain are rejected, not treated as
Neutral. Precise Seam modifiers are not enabled by admitting the attribute.

## Public witness and acceptance

Reuse the existing runtime-Native/API E2E witness:
`option_coverage::seam_string_immediate_lower_layer_restart::back_seam_fits_layer_immediately_below_upward_gap`,
with its unchanged `rear_low_motion_peak.3mf` input and strict whole-object block
comparison. There is no new input, private solver test, copied expected output,
forced corner or acceptance tolerance.

The restart source port changes the API block298B->327B against Native325B:
fitting is restored, but exactly three lines still differ at the initial target,
its short extrusion and final seam-gap endpoint. This follow-on slice must match
all325B, including entry motion, feed, lead-in and gap; an unchanged or partially
matching output is not parity closure. Directed regressions and source checks are
necessary but are not the integration-wide parity gate.
