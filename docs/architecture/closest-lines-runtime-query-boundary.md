# AvoidCrossingPerimeters closest-lines rewrite slice

## Source and destination

Port `libslic3r/GCode/AvoidCrossingPerimeters.cpp`:
`MinDistanceVisitor::operator()`, `get_closest_lines_in_radius`, and the
search-radius/closer-distance arithmetic used by `extend_for_closest_lines`
(lines 130–294), plus its float-radius call sites in `avoid_perimeters`.
Projection distance follows `line_alg::distance_to_squared` in the upstream
line geometry helpers. Rust destinations are only:

- `project_slice/gcode_emit/motion/path/avoid_crossing/router/closest_lines.rs`
- `project_slice/gcode_emit/motion/path/avoid_crossing/router.rs`

This is a source-cited travel-boundary query rewrite, not an Ares routing
algorithm. Existing `Boundary` / `EdgeGrid` carriers remain temporary Rust
adapters. No interlocking generation module, guessed widths, geometry-error
fallbacks, or interlocking WIP history enters this subchain.

## Included behavior

- Preserve source float search-radius arithmetic through the query and squared
  radius comparison; do not widen before multiplication.
- Visit each qualifying contour/segment only once across grid cells.
- Test the unrounded double projection distance against the float-squared
  radius, while retaining the source integer projected point for routing.
- Sort by squared distance with the already-existing fixed GCC-compatible
  strict comparator, including equal-distance ordering.
- Keep source float closer-distance predicates in endpoint extension.

Unrelated detour topology, path simplification, travel emission, wiping,
interlocking, UI/OpenGL and filesystem logic are deferred. Do not add another
router or a compatibility fallback.

## Independent E2E RED

Use the unchanged frozen normal-part two-extruder project with interlocking
**disabled** (SHA256
`1dbedfe681a6efdd527c4f032ddfe26cfb9746e817711a5d59bc4889e5751486`), and the
single-volume control (SHA256
`1ac6d319469fbee245bdc9b44b1600980b5777e62a0e84f34fac0f798dc85651`).
These are authored inputs, not Native output. The two-extruder target must
actually use both tools. Native is mandatory, runs twice for determinism,
and supplies each runtime oracle. API and CLI must be raw-byte identical;
both streams must match Native with the existing complete ordered-byte,
generator-only comparator. Keep inputs, commands, Native/replay streams,
API/CLI streams and comparison JSON as repeatable artifacts.

Counterfactual evidence from the interlocking work already shows that
restoring main's router breaks the beam-0 control at line 728 while the
closest-lines port passes. This is only preliminary evidence: reproduce RED
with no interlocking production changes before GREEN.
The single-volume control must stay green.

GREEN changes only the two production files above. Any unrelated failure
must be diagnosed rather than hidden by test normalization, skips or new
unsupported guards. Do not claim an independent RED for unobservable cleanup.

## Acceptance

- Clean spec → frozen Native/API/CLI RED → production-only GREEN on latest main.
- Both focused modes pass, no changed Rust file above 400 lines.
- `cargo fmt --check` and workspace all-target clippy with `-D warnings` pass.
- Both official and seeded complete full suites have zero failures on the
  exact final commit.
