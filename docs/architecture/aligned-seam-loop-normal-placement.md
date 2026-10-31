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

Included is the Neutral case of the non-Enforced `place_on_loop` alignment
branch at2058-2062. Every candidate is Neutral in the current supported input
path, not because painted candidates may safely use the same algorithm:

- Upstream `Format/bbs_3mf.cpp:357` names the triangle seam-paint attribute
  `paint_seam` (`CUSTOM_SEAM_ATTR`); the importer reads it at4041 and reconstructs
  `seam_facets` at5389-5390.
- In Rust, `project/load/graph.rs::load` sends both the root model and referenced
  component models through `project/xml.rs::deserialize_xml` with `XmlRole::Model`.
  Its XML validation checks attributes on both start and empty elements through
  `validate_attributes` and `project/xml/attribute.rs::validate`, which dispatches
  to `project/xml/model.rs::validate_attribute_namespace`. That function's model
  attribute whitelist admits `paint_supports`, `paint_color` and `paint_fuzzy_skin`,
  but not `paint_seam`. A model carrying that seam-paint attribute is rejected as
  `SliceError::InvalidInput` before deserialization/slicing, not silently imported
  as Neutral.
- `project/model_xml.rs::Triangle` and `project/domain/mesh.rs::ProjectMesh`
  retain support, color/MMU and fuzzy-skin annotations, but no seam-paint field.
  Those other annotations are not converted into seam paint.
  `project_slice/seam_candidates.rs::SeamCandidate` has no Enforced/Blocked state;
  `project_slice/seam_placement/alignment/preparation.rs::prepare_layer` generates
  candidates from perimeter collections, without a seam-annotation import channel.

Thus actual seam-paint annotations cannot currently reach this alignment code
through the supported project loader. Both source allowed-arc walks consequently
traverse a complete Neutral loop and return unlimited arcs; the solution is not
clamped to a painted boundary. Admitting/importing seam painting must first get a
follow-up public RED and a source-cited annotation/placement slice, rather than
silently reusing this Neutral-only assumption.

Explicitly deferred:

- The **Enforced alignment branch at2063-2075**, including its sharp-angle return,
  `protect = max(flow_width, max_nozzle_diameter)` and source default, measured
  corner protection via `enforced_corner_influence`, and the corner-weighted
  `t = max(t, 0.4 * corner)` pull before mapping back onto the painted loop.
  The Neutral port does not implement or claim parity for that branch.
- **Blocked-candidate arc limits** in `place_on_loop::allowed_arc`, as well as
  Enforced-run limits, exact paint ends and oversampling extensions. These require
  retained Enforced/Blocked annotations; unlimited Neutral arcs do not emulate
  their behavior.
- `SeamContour::Resolver`, Precise Seam warnings/import, spline/numeric changes,
  candidate fallback and geometry changes. This slice does not admit or emulate
  those adjacent upstream behaviors.

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
