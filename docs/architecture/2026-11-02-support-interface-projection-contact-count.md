# Support interface projection contact-layer accounting

## Rewrite boundary

Authority: OrcaSlicer v2.5.0-beta, commit
`345efec64c7b95a05ce5cf96df0dde66a158835c`.

- `src/libslic3r/Support/SupportParameters.hpp:47–50` stores requested
  interface counts and records whether separate contacts exist.
- `src/libslic3r/Support/SupportCommon.cpp:220–233` derives interface
  projection counts: separately printed top and bottom contacts each consume
  one requested interface layer. Organic bottom contacts consume none because
  they are projection seeds. Base-interface counts are clamped to the remaining
  counts; interface-only counts are the derived counts minus those base counts.
- `SupportCommon.cpp:305–354` uses these reduced counts to choose the top and
  bottom projection slabs.
- Rust destination: `crates/ares-core/src/project_slice/support_material/interface_grid.rs`,
  before delegation to `interface_grid/projection.rs`.

This is a port of upstream interface generation, not a new support pipeline.
The existing raw `InterfaceCounts` remains the support-parameter representation;
its direct use as a projection count is the temporary scaffold being replaced.

## Measured boundary

The existing per-object support fixture supplies two matching object geometries.
Native debugger captures and temporary Ares captures were compared using integer
vertices. Comparison normalizes polygon order and closed-ring starting vertex
only; it does not round coordinates, reverse orientation, or discard vertices.

| Boundary | Result |
| --- | --- |
| Projection area and downward propagation, object Z0.2–2.4 | 48 comparisons equal |
| Bottom-contact, top-contact, and base object-trim inputs/outputs around the notch | 40 comparisons equal |
| After interface generation, before support-layer installation | 12 of 30 records differ |

The differing records are base and bottom-interface polygons at support
Z1.4000000029802322, Z1.6000000029802321, and Z1.9586956550245698, for both
objects. Other captured records match. At Z1.4, the base notch starts at scaled
X -4487272 in Native and -4687272 in Ares: a 0.2 mm difference.

Native `SupportCommon.cpp:340` reports top and bottom projection counts of 2
at all three differing support heights. Before the fix, the Ares dump reports
3 for both counts at these heights. At support Z1.4, its bottom slab reaches
Z0.8000000029802323 in the support fixture and Z0.500000000745058 in the raft
fixture. The Rust projection consumes requested counts without the contact-layer
deduction.

Native `SupportMaterial.cpp:3385` also confirms that BASE at
Z1.4000000029802322 overlaps object Z1.5999999999999999. Its bottom Z is 1.25.
The overlap predicate and upstream-authentic Flow float promotion are not the
cause. The triangle disappearing between object Z1.2 and Z1.4 occurs in both
engines.

## Non-negotiable decisions

1. Derive local projection counts at the ordinary Grid/Snug interface-generation
   boundary. Use `n > 0` for the upstream contact predicate and subtract the
   separately printed contact layer only when present, preserving the Organic
   bottom-contact exception. Clamp base-interface counts to the derived counts;
   derive interface-only counts by subtraction. No saturating arithmetic is needed.
2. Preserve raw requested counts for toolpath admission and other consumers.
3. Do not change overlap arithmetic, introduce epsilon adjustments, or change
   Clipper2 globally. The object-trim square join and its effective miter limit
   already agree: `SupportMaterial.cpp:100,3391` and
   `ClipperUtils.cpp:419–425,504–530`.
4. Preserve the upstream Organic exception in this derivation, but do not reroute
   the separate Organic generator. Contact generation, base generation, layer
   installation, and toolpath algorithms are outside this fix.

## Verification

Use the existing strict Native/API/CLI support and raft witnesses without
weakening their assertions. Before implementation, both witnesses fail under
`cargo nextest run` (0 passed, 2 failed). After the port,
verify the divergent polygons against the Native captures, rerun both witnesses,
and report any remaining full-stream differences separately rather than calling
partial parity GREEN. Run the support/raft family with zero PASS-to-FAIL changes,
plus rustfmt and Clippy. Diagnostic scripts and raw dumps remain outside the
repository as temporary scratch artifacts; no product debug hook is retained.


Accepted completion is geometry-scoped GREEN. The preset-refresh footer keys
(`compatible_process_expression_group` and `pellet_flow_coefficient`) remain an
explicit strict full-output residual until the preset-refresh slice supplies
them. Do not mark either full-output target as PASS while that residual remains.

### Observed result

On base main `444364eaf`, the port makes all 30 Native pre-toolpath polygon
records match, including object, source kind, Z, height, and bottom Z. The
12 differing records now match. These captures come from the uninstrumented
Rust executable; no product debug hook is retained.

For both target fixtures, API and CLI outputs are byte-identical. With only the
existing generator-identity header normalization, each entire prefix before
`CONFIG_BLOCK_START` matches Native, including motion and M73 lines. The strict
full-output residual remains the preset-refresh footer keys. Neither full-output
target is reported as PASS, and no comparison assertion is changed.

The 202-test support/raft family changes from 112 passed / 90 failed to
114 passed / 88 failed, with zero PASS-to-FAIL changes. The two improvements are
`brim_ears_explicit_width_support_no_brim_matches_native` and
`support_ironing_raft_offset_native_strict_red`. Both runs use the same pinned
Native authority and verified source profile resources. Rustfmt and strict
workspace, all-target Clippy pass.
