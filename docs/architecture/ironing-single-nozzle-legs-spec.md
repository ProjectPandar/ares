# Object ironing patterns on a single-nozzle base

## Source boundary

The comparisons cover OrcaSlicer `345efec64c7b95a05ce5cf96df0dde66a158835c`:

- `src/libslic3r/Fill/Fill.cpp:1707-1932`, `Layer::make_ironing`: one `IroningParams` per
  region, `ironing_pattern` read at `:1807`, and the shared filler created through
  `Fill::new_from_type` at `:1818` and `:1831`.
- `src/libslic3r/Fill/FillBase.cpp:65-104`, `Fill::new_from_type`: every `InfillPattern`
  reaches its own `fill_surface`.

The Rust destination is unchanged: `fill_entities/ironing.rs` dispatches every pattern
listed in `ironing-pattern-coverage-spec.md`. This slice changes only the inputs the
comparisons run on, so that each pattern leg tests ironing and not the dual-nozzle path.

## Why the legs move

The pattern legs in `crates/ares-cli/tests/orca_parity/option_coverage/ironing_pattern.rs`
run on `tests/parity/summary/time-cost.3mf`, a Bambu Lab X2D project with two nozzles. On
that fixture, the ironing extrusion moves of all 16 legs that Native completes match Native
in order on every ironed layer, in the official and seeded modes. The whole-output
comparison still fails, because of differences on the dual-nozzle path that do not come
from ironing: the same differences remain with ironing off. A failing leg therefore says
nothing about its pattern.

## Derived base

Each moved leg builds its project at runtime from two committed fixtures; no fixture file is
added:

- the mesh, model settings and plate from
  `tests/parity/wall-generator-arachne/x1c-wall-generator-arachne.3mf`, the same
  12234-triangle model as `time-cost.3mf`;
- `Metadata/project_settings.config` replaced by the one from
  `tests/parity/rear-seam-visibility/aligned-control.3mf` (Prusa MK4, one 0.4 nozzle,
  Arachne walls), whose whole-output comparison already matches Native in both modes;
- in that configuration, `"ironing_type": "no ironing"` becomes `"top"` and
  `"ironing_pattern": "rectilinear"` becomes the leg's pattern.

The model fits the MK4 bed at its stored position, so no transform changes.

Premises, checked at runtime, never against stored Native values:

- both replaced keys are present in the MK4 configuration;
- the derived configuration has exactly one `nozzle_diameter` value;
- the Native output irons more than one layer: it counts the layers (`;LAYER_CHANGE`) that
  contain a `;TYPE:Ironing` block.

The comparison is the existing one: Native `--slice 0`, then the validated API and the
CLI, each compared whole-output against Native with
`golden::compare_ordered_bytes_generator_only`, and the API output must equal the CLI
output. Nothing is masked, `M73` lines included.

## Rust destination of the tests

- `option_coverage/ironing_pattern/single_nozzle.rs`, a new child module, builds the derived
  project, checks the premises and holds the 15 moved legs.
- `option_coverage/ironing_pattern.rs` keeps the X2D legs, `control-rectilinear`,
  `adaptivecubic`, `supportcubic` and `lightning`, and shares its Native, API and CLI
  comparison with the child module. `three_d_honeycomb_guard.rs` is unchanged.
- The module documentation names v2.5.0-beta `345efec64` instead of 2.4.2, and its
  `Fill.cpp`/`FillBase.cpp` line citations move to the ranges above.

Test names state the behavior: `<pattern>_ironing_matches_native`, for example
`tri_hexagon_ironing_matches_native`.

## Effect on the full suite

The full suite identifiers are `option_coverage::ironing_pattern::…`. For each of the 15
moved patterns, `orca_parity_ironing_pattern_<pattern>` becomes
`single_nozzle::<pattern>_ironing_matches_native`, which is a rename for the inventory. The
expected status after the move is the same in both modes:

| Pattern | Before (X2D) | After (single nozzle) |
|---|---|---|
| monotonic | F | P |
| monotonicline | F | P |
| alignedrectilinear | F | P |
| line | F | P |
| grid | F | P |
| triangles | F | P |
| tri-hexagon | F | P |
| quartercubic | F | P |
| honeycomb | F | P |
| lateral-honeycomb | F | P |
| crosshatch | F | P |
| tpmsd | F | P |
| gyroid | F | P |
| cubic | F | F |
| tpmsfk | F | F |

That is 13 F to P and 2 F to F per mode, with no other identifier touched.
`cubic` and `tpmsfk` fail on one `M73` line each, deferred below.

Unchanged: `orca_parity_ironing_pattern_control_rectilinear` (F) stays on the X2D fixture
as the whole-output witness for the dual-nozzle differences; the three crash guards (P) and
the 3D honeycomb guard (P) stay as they are.

## Failure modes

- The derived base is no longer a single-nozzle project, or the MK4 configuration loses an
  anchor: a premise fails before any comparison.
- A pattern stops ironing more than one layer in Native: the premise fails instead of a weak
  comparison passing.
- Masking or dropping `M73` lines would hide the processor gap below: the comparison stays
  whole-output.
- Moving the control leg would drop the only whole-output dual-nozzle witness for ironing:
  it stays on X2D.

## Deferred

- `M73` progress placement, an open GCodeProcessor gap: `GCodeProcessor::run_post_process`
  (`src/libslic3r/GCode/GCodeProcessor.cpp:1167`) inserts the `M73 P… R…` lines through
  `process_line_move` (`:1445-1512`, called at `:1846` and `:1852`) from the
  `g1_times_cache` filled at `:748`. On the derived base, Native places one `M73` line one
  G1 earlier than Ares. With ironing off it is `M73 P4 R137` inside an inner-wall block;
  that control (the derived base with `ironing_type` left at `no ironing`) is the witness
  for the later slice. With ironing on, the shift lands on `cubic` (`M73 P6 R168`) and
  `tpmsfk` (`M73 P4 R163`), both inside outer-wall blocks, not in ironing. Which pattern
  hits it depends on where the time boundaries fall.
- The dual-nozzle differences of the X2D fixture, which the control leg keeps visible.
- `rectilinear` on the derived base: the control leg keeps that pattern on X2D.
