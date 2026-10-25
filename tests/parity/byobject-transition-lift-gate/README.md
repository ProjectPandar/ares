# byobject-transition-lift-gate

Derived from `tests/parity/byobject-transition-comments/comments-on.3mf` by
patching only `retract_lift_above` / `retract_lift_below` in
`Metadata/project_settings.config`:

- `gate-open.3mf` — `["0"]` / `["0"]` (band disabled): the by-object
  transition emits the deferred hop (`G1 Z4.8 F9000 ; normal lift Z`) and
  travels at `hop_z` (`GCodeWriter.cpp:626-648` defers, `travel_to_xyz`
  branch 1 raises the destination).
- `gate-band.3mf` — `["1"]` / `["3"]`: object 1's top layer (Z 4.0) and the
  post-safety-lift writer z (4.4) sit above the upper boundary, so no lift is
  ever deferred: no `normal lift Z` line, and the origin travel targets the
  finished object's nominal Z (`G1 X140 Y110 Z4 F9000`,
  `GCode.cpp:7483` `z == DBL_MAX ? m_nominal_z`).

Upstream boundary: `GCode.cpp:3326-3327` (transition safety lift + `travel_to`),
`GCode.cpp:7432-7462` (`travel_to` retraction handling / `lazy_lift`),
`GCodeWriter.cpp:626-648` (`lazy_lift` defer conditions with the
`retract_lift_above/below` band at `m_pos.z()`).
