# Periodic infill post-clipping endpoint joining

## Source boundary

Authority: OrcaSlicer v2.5.0, `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`.
The rewrite slice is upstream commit
`803dddb0f5c194a0f502ef0f2cc9e6c0e9360d3c`:

| Upstream method and call site | Rust destination |
| --- | --- |
| `FillGyroid::_fill_surface_single`, `Fill/FillGyroid.cpp:379–381` | `crates/ares-core/src/fill/gyroid.rs::fill_component` |
| `FillTpmsD::_fill_surface_single`, `Fill/FillTpmsD.cpp:166–168` | `crates/ares-core/src/fill/tpms_d.rs::fill_tpmsd_surface` |
| `FillTpmsFK::_fill_surface_single`, `Fill/FillTpmsFK.cpp:168–170` | `crates/ares-core/src/fill/tpms_fk.rs::fill_tpmsfk_surface` |

The commit adds `join_close_ends(polylines, SCALED_EPSILON, true)`
immediately after `intersection_pl`. Clipping may split a closed ring at
its stored first point, leaving two fragments that must be rejoined before
short-fragment removal and infill connection.

## Required behavior

- Invoke the existing Rust `fill::join_close_ends::join_close_ends` after
  open-polyline intersection in each of the three destinations.
- Use `1e-4 / scale.factor()` for both the maximum gap and the helper's
  existing scaled epsilon argument, and allow reversal regardless of the
  later path-ordering setting. This preserves the upstream scaled-double
  boundary rather than rounding it to an integer coordinate.
- Join before the existing `0.8 * spacing` length filter. Preserve subsequent
  connection, sorting, anchoring, and rotation behavior.
- Keep the earlier multiline expansion and base-fill endpoint joining in
  their current positions. Neither substitutes for the new post-clip call.
- Reuse the existing endpoint algorithm corresponding to
  `Fill/FillBase.cpp::join_close_ends` at line3036. Do not change its gap
  ordering, collision handling, or parent/path assembly.

The current fill modules already express these upstream methods; their
missing calls are completed directly. No Ares-specific pipeline stage,
compatibility shell, fallback, or new public API is introduced. The logic
remains platform-neutral core geometry for browser WASM and native targets.

## Excluded behavior

Sampling equations, floating-point rotation boundaries, clipping,
multiline geometry, extrusion roles, speed estimation, G-code formatting,
profiles, fixture parameters, STL bytes, and comparison rules are unchanged.
No adaptive-infill algorithm or endpoint-joining redesign is included.

Bridge speed selection is a distinct source boundary:
`44f792dbc3cd94b927ad770ae482a99e17db4d25`,
`GCode.cpp::_extrude`, v2.5.0 lines9719–9739. In particular, fully unsupported
`erBridgeInfill` uses bridge speed when curled-perimeter slowdown is enabled.
That behavior is not part of this slice.
