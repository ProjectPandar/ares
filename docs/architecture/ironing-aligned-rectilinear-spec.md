# Ironing `alignedrectilinear`: `FillAlignedRectilinear` layer angle + monotonic traversal

ARD: none required (no non-negotiable architectural decision; this is a direct
port slice of two upstream call-site behaviours already scaffolded in Ares).

## Problem

`ironing_pattern=alignedrectilinear` produces byte-different G-code from Native
OrcaSlicer 2.4.2 on the same project, while `ironing_pattern=rectilinear`
(and every other committed ironing pattern) is byte-equal.

Measured on `tests/parity/summary/time-cost.3mf` patched with
`ironing_type=top`, `ironing_pattern=alignedrectilinear`,
`gcode_label_objects=0` (seeded oracle, `ORCA_REPLAY_SEED=0`):

- Native: first diff at gcode line 12797 of `plate_1.gcode`; model time
  `6h 44m 55s`.
- Ares: `6h 45m 1s`; the same line and the whole ironing block diverge.
- Control `ironing_pattern=rectilinear` on the same fixture: byte-equal.

Two independent causes, both at the `Layer::make_ironing` fill construction:

1. **Layer angle.** Ironing builds its filler with
   `Fill::new_from_type(ironing_params.pattern)`
   (`Fill.cpp:1626-1641`). For `ironing_pattern=alignedrectilinear` that is
   `FillAlignedRectilinear`, whose `_layer_angle` override returns `0.f`
   (`FillRectilinear.hpp:38-47`). The base implementation alternates
   `(idx & 1) ? PI/2 : 0` when `fixed_angle` is off (`FillBase.hpp:211`,
   applied at `FillBase.cpp:300-316`). The fixture has
   `ironing_angle_fixed=0` and an empty `solid_infill_rotate_template`, so
   upstream never rotates the aligned-ironing direction, while Ares rotates on
   every other layer. Verified from the emitted G-code: Native ironing layers
   are all `45°` (`z=4.00, 9.20, 10.00, 14.00, 16.00, 18.00`), Ares alternates
   `135°/45°` exactly like `rectilinear`.
   The same override list also fixes `Grid`, `LateralLattice`,
   `LateralHoneycomb`, `Triangles`, `Cubic`, `QuarterCubic`
   (`FillRectilinear.hpp:75-77,87-89,100-102,113-115,126-128,139-141`;
   `FillHoneycomb.hpp:53` is the honeycomb 1/3-turn exception and is *not*
   angle-fixed).

2. **Monotonic traversal.** `Layer::make_ironing` sets
   `fill_params.monotonic = true` (`Fill.cpp:1619-1621`) for *every* ironing
   pattern. `FillRectilinear::fill_surface_by_lines` consumes it as
   `monotonic_infill` and switches to the monotonic-region traversal
   (`FillRectilinear.cpp:2853-2867`). Ares routes `Rectilinear`/`ZigZag`/
   `CrossZag`/`LockedZag` through `fill_monotonic_surface`, but
   `AlignedRectilinear` fell through to the generic surface dispatcher
   (`ironing::surface::polylines` → `fill_entities::monotonic::
   expolygon_polylines` → `fill_rectilinear_surface`), i.e. the
   *non*-monotonic traversal. That produced the observed link/ordering
   difference: Native links the end of one ironing line to the start of the
   next and re-enters it from the near end, Ares retracts and travels to the
   far end.

## Upstream boundary (source-cited)

- `OrcaSlicer/src/libslic3r/Fill/Fill.cpp:1507-1730` — `Layer::make_ironing`:
  `.1626-1641` filler construction from `ironing_pattern`, `.1619-1621`
  `fill_params.density = 1.; fill_params.monotonic = true;`, `.1700-1712`
  `f->fill_surface(&surface_fill, fill_params)` per ironing area.
- `OrcaSlicer/src/libslic3r/Fill/FillBase.cpp:40-77` — `Fill::new_from_type`,
  `ipAlignedRectilinear → new FillAlignedRectilinear()` (`:51`).
- `OrcaSlicer/src/libslic3r/Fill/FillRectilinear.hpp:38-47` —
  `class FillAlignedRectilinear : public FillRectilinear` with
  `_layer_angle(idx) = 0.f`; `.75-161` the other `0.f` overrides.
- `OrcaSlicer/src/libslic3r/Fill/FillBase.hpp:211` — default
  `_layer_angle = fixed_angle ? 0 : (idx & 1) ? PI/2 : 0`;
  `FillBase.cpp:300-316` — `_infill_direction` applies it.
- `OrcaSlicer/src/libslic3r/Fill/FillRectilinear.cpp:2853-2867` —
  `monotonic_infill = params.monotonic` → `generate_monotonic_regions` /
  `connect_monotonic_regions` / `chain_monotonic_regions`.

## Ares destination boundary

- `crates/ares-core/src/project_slice/fill_entities/ironing.rs` — the
  `Layer::make_ironing` port (`Fill.cpp:1507-1730`). Its rectilinear-family arm
  (`Rectilinear | ZigZag | CrossZag | LockedZag`) gains `AlignedRectilinear`
  and passes `layer_angle_fixed: pattern == AlignedRectilinear` (the upstream
  `_layer_angle() == 0` disposition for that one class).
- `crates/ares-core/src/fill/rectilinear/surface.rs` —
  `MonotonicFillParams`/`infill_direction` is the `FillBase::_infill_direction`
  port; it gains a `layer_angle_fixed` flag that suppresses the parity
  alternation exactly where upstream's `_layer_angle` override returns `0.f`.

Existing scaffolding is used as-is: `fill_monotonic_surface`
(`fill/rectilinear/surface.rs:54-57`) already implements the monotonic
traversal; the generic `ironing::surface` dispatcher keeps serving the
non-rectilinear patterns. No new pipeline, no new module.

## Included

- `ironing_pattern=alignedrectilinear` (ProcessInfillPattern variant already in
  the ironing vocabulary) becomes byte-equal to Native on the parity fixture.
- `_layer_angle`-fixed patterns that reach `fill_rectilinear_surface`
  (`AlignedRectilinear`; `Grid`/`LateralLattice`/`LateralHoneycomb`/
  `Triangles`/`Cubic`/`QuarterCubic` are recorded but use their own fill
  modules) stop alternating across layers, matching `FillRectilinear.hpp`.
- The rectilinear control and the other committed ironing patterns are
  unchanged (verified byte-equal before/after).

## Deferred

- `ironing_pattern=lateral-lattice` — first structural diff is a ~2 MB missing
  block (`FillLateralLattice`), a separate fill-structure slice.
- Support ironing (`support_ironing_pattern`) — separate slice.
- Brim (`brim_type`, `brim_ears`, `brim_separation`) — separate slice.
- `FillHoneycomb.hpp:53`'s `PI/3 * (idx % 3)` honeycomb angle progression is
  out of scope: the honeycomb ironing patterns are already byte-equal.
- Non-ironing `FillAlignedRectilinear` surfaces (solid/top-surface infill filled
  through the generic `fill_entities::surface` dispatcher) keep the dispatch's
  existing layer alternation: this slice carries the `_layer_angle() == 0`
  disposition only on the `Layer::make_ironing` call site. Extending it to the
  non-ironing dispatcher needs its own parity evidence and is deferred.

## Test plan (red)

Extend the existing spec-driven coverage test
`crates/ares-cli/tests/orca_parity/option_coverage/ironing_pattern.rs` with the
`alignedrectilinear` leg that was previously deferred:

- Fixture: `tests/parity/summary/time-cost.3mf` patched with
  `ironing_type=top`, `ironing_pattern=alignedrectilinear`,
  `gcode_label_objects=0`.
- Native reference via the configured oracle (`--slice 0 --outputdir`), then
  byte-compare Ares' validated-API output and CLI output against it with
  `golden::compare_ordered_bytes_generator_only`.
- RED expectation: the leg fails on the pre-slice code (first diff at line
  12797; Native `6h 44m 55s` vs Ares `6h 45m 1s`).
- The existing `rectilinear` control leg keeps passing in the same run.

## Acceptance

- `alignedrectilinear` ironing leg byte-equal, official and seeded oracles.
- The `ironing_pattern` coverage suite (20 legs) green in both oracles.
- `cargo fmt --check`, `cargo clippy -D warnings`, and
  `cargo check --target wasm32-unknown-unknown` clean.
- Full `orca_parity` in both oracles with no new failures.
