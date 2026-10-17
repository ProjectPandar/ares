# SPEC — Retire the temporary rectangle shell (`avoid_crossing::rectangle_route`)

## Source boundary

`AvoidCrossingPerimeters::travel_to`
(`OrcaSlicer/src/libslic3r/GCode/AvoidCrossingPerimeters.cpp:1233-1312`)
versus Ares
`crates/ares-core/src/project_slice/gcode_emit/motion/path/avoid_crossing/rectangle.rs`
(194 LOC) reached from `start_travel/route.rs:61`
(`route().unwrap_or_else(rectangle_route)`).

## (1) Every `route() -> None` condition, with the upstream behavior

Upstream `travel_to` **always returns a valid polyline** — at minimum
`{start, end}` (`:1284-1288`: `if (result_pl.empty()) result_pl = {start,
end}`). There is no upstream failure/None analogue; every Ares `None`
is a Rust-internal error path. Enumerated from `avoid_crossing.rs`:

| # | `route()` None site | Condition | Upstream `travel_to` behavior |
|---|---------------------|-----------|-------------------------------|
| 1 | `if !detour_emission_ready() { return None }` (:66-68) | Never — the flag is `true`; dead code | n/a (flag existed only while waypoint emission was unported) |
| 2 | `boundary?` where `boundary` is `Option` from `build_boundary()` (:86) | `build_boundary` returns `None` when `safe_zone::build` returns `Err(ClipperError)` | `offset_ex` failure would throw upstream (uncaught); no real-input trigger — the safe zone is `offset_ex(layer.lslices, ...)` of a layer that was already sliced |
| 3 | `to_scaled(start)`/`to_scaled(end)` return `None` (:103-104) | `checked_scale_rounded` overflow — a travel endpoint outside the i64 scaled range (mm ≈ ±9.2e12 mm at SCALING_FACTOR=1e-6); impossible for a real print | n/a |
| 4 | `boundary::Boundary::build(...).ok()?` / `build_with_spacing` returns `Err` or `Unavailable` (:111-118) | `Unavailable` = `layer_slices.is_empty()` or `perimeter_spacing <= 0` or spacing `checked_scale` overflow; `Err` = Clipper failure | Upstream calls `init_boundary` unconditionally in the same situation; an empty boundary leads to `!m_internal.boundaries.empty()` false → `result_pl = {start, end}` → **straight travel** (`:1259-1264`, `:1284-1288`). `Unavailable` semantics = straight. |
| 5 | `boundary::Boundary::build_external(...).ok()?` (`use_external`, :140-146) | `Unavailable` = `checked_scale(perimeter_spacing)` overflow only; `Err` = Clipper | `use_external` with an empty external boundary → straight travel (`:1273-1288`) |
| 6 | `boundary.internal.as_ref()?` / `boundary.external.as_ref()?` | `Unavailable` stored as `None` | Same straight-travel mapping as #4/#5 |
| 7 | `router::avoid_perimeters(...).ok()?` (:159) | `Err(ClipperError)` | `avoid_perimeters` returning an error has no upstream branch — upstream computes `result_pl` unconditionally |

**Divergent consequence today:** on any `None`, `plan_route` invokes the
rectangle shell, which — for the single-axis-aligned-4-point-contour
geometry it accepts — *fabricates boundary detour waypoints* (the
`boundary_path` corner/edge path), behavior upstream never exhibits.
For all other geometry it returns `Vec::new()` = straight, which
coincidentally matches upstream's `{start, end}` for conditions #4/#5
but hides the error silently for #2/#3/#7.

## (2) Fixture reaching the shell + Native comparison

No real input drives `route()` to `None` — every site is an internal
error path (Clipper failure, spacing ≤ 0, i64 overflow), not a
geometry-dependent state. The shell was only reachable end-to-end while
`detour_emission_ready() = false`; the boundary router landed and the
flag is permanently `true` (`avoid_crossing.rs:226-228`).

Instrumented experiment (evidence
`~/.ares-evidence/worker4/rect-shell-probe/`): `detour_emission_ready`
forced to `false` in a scratch build, `detour-3parts.3mf`
(`tests/parity/max-travel-detour`, `reduce_crossing_wall=1`, cap=0)
sliced with the shell handling every routed travel.

- Shell output: **527 stripped diff lines vs official Native** —
  every upstream boundary-detour waypoint (e.g. `G1 X74.389 Y84.389`)
  is missing; a Z-hop target differs (`Z.483` vs `Z.795`); feedrate
  drift follows (`F1625` vs `F1626`).
- `rectangle()` itself returns `None` here (multi-part `internal_surfaces`),
  so the shell degraded to straight lines; where it does engage
  (single axis-aligned block) it would instead emit *wrong* corner
  waypoints. Either way the path is wrong whenever it changes output.

## (3) Ares destination + included/deferred behavior

**Destination:** delete `avoid_crossing/rectangle.rs` entirely
(module declaration, `pub use rectangle::route as rectangle_route`,
tests exercising it), remove the
`!detour_emission_ready() { return None }` dead gate, and change
`start_travel/route.rs` to not fall back — `route()`'s `None` becomes
a straight travel.

Concretely: `plan_route` replaces
`route(...).unwrap_or_else(|| rectangle_route(...))` with
`route(...).unwrap_or_default()` — the `None`-to-empty mapping keeps
upstream's `{start, end}` result for conditions #4/#5 exactly, and
makes the unreachable internal errors (#2/#3/#7) degrade to the same
straight travel instead of fabricating waypoints.

`detour_emission_ready`/`routing_active`: the dead gate at :66-68 is
removed. `routing_active()` has exactly one caller —
`start_travel/route.rs:17` — where it selects between
`route_gate = state.positioned` (routing active) and
`route_gate = state.layer_index > 0` (the dormant shell-era default,
commented `route.rs:15-16` as the reason it exists). With routing
permanently active both collapse to `state.positioned`: delete
`routing_active()` and the `route_gate` conditional.

**Included:** straight-travel mapping for `Unavailable` boundary
(`layer_slices.is_empty()`, `perimeter_spacing <= 0`) — semantic
match to upstream `:1259-1264`/`:1284-1288`, replacing the rectangle
shell. Removal of all rectangle-shell code paths and its unit tests.

**Deferred:** making the ClipperError/overflow sites observable
(e.g. structured error propagation) — upstream aborts the process;
today they're unreachable; the straight fallback is the bounded
behavior-preserving choice. `use_external`/`use_external_mp_once`
semantics are unchanged (outside this shell's boundary).
`could_be_wipe_disabled` remains unmodeled (dead upstream consumer,
per prior GREEN review).

## RED plan

The shell is unreachable on real inputs — no real-input leg can fail
on the old code. This is a **behavior-preserving dead-code retirement
with a regression-control gate**, not a divergence RED:

1. **Control legs (existing, must stay green):** all 5
   `max_travel_detour` legs + all 6 `hole_to_polyhole` legs + the
   avoid/crossing/travel family (35 tests) — rerun both modes on the
   retirement hash. Whole-output strict comparison already proves the
   `None` sites are never hit by these fixtures.
2. **Unit witness:** extend the existing `avoid_crossing` unit tests —
   a `Request` on an empty-boundary `LayerGeometry` (already built by
   `tests/empty_boundary.rs`) must yield `Some(empty)` straight
   travel through `route()`; previously `plan_route` would have
   invoked `rectangle_route` on it — assert via the
   `empty-boundary` fixture that `route()` returns `Some(vec![])` and
   that `rectangle_route` no longer exists (compile-fails if called).
3. GREEN = the deletion + `unwrap_or_default()` + `routing_active`
   cleanup, gated by fmt/strict clippy/wasm + the 35-test family
   and queued-chain legs, both modes, exact hash.
