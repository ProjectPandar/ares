# Arachne upward quad-mid ordering

## Upstream boundary and Rust destination

Port `SkeletalTrapezoidation.cpp:1430-1467::generateSegments`: collect upward
quad mids in `graph.edges` order and sort them with the upstream boolean
less-than lambda using the existing GCC libstdc++ introsort implementation.
The destination is
`ares-core::arachne::trapezoidation::transitions::segments::upward_quad_mids`.
This replaces the stable-sort compatibility scaffold around the upstream
concept; it does not introduce an Ares-owned slicing policy.

## Required decisions

- Reuse `geometry::fixed_gcc_sort_by`, the existing crate-visible re-export
  of `geometry::clipper::ordering::gcc::fixed_gcc_sort_by`. Do not copy
  the implementation or expand visibility unnecessarily.
- Preserve graph iteration and candidate filtering. Port the lambda literally
  as a boolean predicate: descending destination radius, flat-edge precedence,
  and ascending distance-before-rise when both edges are flat. Equal non-flat
  candidates compare false in both directions; add no tie key.
- Preserve the upstream ordinary subtraction in the flat-edge distance
  expression (`SkeletalTrapezoidation.cpp:1448-1450`), not a saturating policy.
- Keep the existing GCC sort implementation unchanged. Do not substitute Rust
  `sort_unstable_by`, whose equal-key permutation is not the oracle's sort.

The ordering is observable: upward propagation
(`SkeletalTrapezoidation.cpp:1561-1587`) copies the first available source
beading into an empty destination and ignores later candidates.

## Included and deferred behavior

Include only sorting fidelity at this existing segment-generation boundary.
Do not change beading computation, transitions, propagation, junction arithmetic,
outline preparation, geometry/Voronoi implementation, options, or G-code emission.
The separately identified float strategy-angle conversion defect remains a
follow-up after #165, not part of this slice.

## Test-first evidence and qualification

The unchanged strict Native-double-replay/API/CLI whole-output witness
`wall_generator_arachne_matches_complete_output` was frozen before
production fixes. Its f46214d06 run remains RED. Read-only v70 evidence identifies
identical ordered graph edges and pre-sort inputs, an equal-key pair reordered
differently by Native and Rust, and the resulting exact failing raw junction.
No fixture, assertion, comparator or tolerance is changed for this port.

Qualify the separate commit with rustfmt, workspace/all-target Clippy, and the
frozen seeded directed witness using coordinator controls. Only a directed GREEN
authorizes official FULL, seeded FULL and REQUIRE=1 workspace non-parity gates;
audit every outcome flip before claiming qualification.
