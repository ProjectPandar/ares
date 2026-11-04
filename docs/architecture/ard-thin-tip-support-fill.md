# ARD: Thin-tip support fill connection parity

Source boundary and acceptance: [spec](../superpowers/specs/2026-10-04-thin-tip-support-fill.md).

Non-negotiable decisions:

- Port actual OrcaSlicer v2.4.2 `FillBase.cpp::connect_base_support` and
  `mark_boundary_segments_overlapping_infill` execution semantics. Its default
  `std::pair<double, double>` stays `(0, 0)` when the collision helper returns
  false; the caller still evaluates the contour's departure distance. The Rust
  `Option` must preserve that caller behavior, not skip it. Keep the upstream
  short-segment collision quirk unchanged; do not invent a contour heuristic.
- Keep geometry logic platform-neutral in `ares-core::fill::base_support`;
  no filesystem, terminal output or native dependency in core.
- Retain both existing public thin-tip RED assertions until full output is
  equal. Timing changes do not excuse a support path difference.
- Diagnostics use external debugger/artifacts, not committed debug probes.
- Official and seeded comparisons use matching runtime domains. Cached
  replay shortens diagnosis but is not fresh original-Orca acceptance.
- Existing graph/adapters are compatibility shells around the cited source
  concepts, not authority for new pipeline behavior. Scope excludes concurrent
  matrix cleanup, tower/AdaptivePA and Arachne work.
