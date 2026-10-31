# Spec — ironing_pattern coverage for all ProcessInfillPattern variants

## Gap

`project_slice/fill_entities/ironing.rs:231-234` rejects every
`ProcessInfillPattern` outside {Concentric, HilbertCurve, ArchimedeanChords,
OctagramSpiral, Rectilinear, ZigZag, CrossZag, LockedZag} with
`UnsupportedProjectFeature("ironing_pattern")`. Native probes on a top-surface
ironing fixture (ironing_type=top, real model `tests/parity/summary/time-cost.3mf`
with `ironing_pattern=<x>` patched into `project_settings.config`) show OrcaSlicer
emits real ironing feature blocks for 16 of the remaining 19 variants and crashes
on 3.

## Upstream cite

`src/libslic3r/Fill/Fill.cpp:1573-1705` (`Layer::make_ironing`) builds one
`IroningParams` per region, sorts regions by `(extruder, just_infill, pattern,
angle, spacing)`, and drives a shared `std::unique_ptr<Fill>` created via
`Fill::new_from_type(f_pattern)` (`Fill/FillBase.cpp:40-77`). There is **no
pattern whitelist** — every `InfillPattern` reaches the corresponding
`fill_surface` implementation.

`fill_params` for ironing is set at `Fill.cpp:1617-1622`:

    fill_params.density     = 1.;
    fill_params.monotonic   = true;
    fill_params.link_max_length = scale_(3 * spacing);

and `f->overlap = 0`, `f->dont_alternate_fill_direction = zaa_dont_alternate_fill_direction`,
`f->layer_id = this->id()`, `f->z = this->print_z`,
`f->set_bounding_box(this->object()->bounding_box())` (`Fill.cpp:1623-1628,
1633-1638`). The emitted polylines are pushed into a `no_sort` collection as
`erIroning` with `mm3_per_mm = nozzle * extrusion_height`, `width =
rounded_rectangle_extrusion_width_from_spacing(nozzle, extrusion_height)`,
`height = ironing_params.height * spacing / nozzle_dmr` (`Fill.cpp:1705-1723`).

`InfillPattern` serialization (`PrintConfig.cpp:3024` `enum_keys_map`) maps the
CLI strings the probe used: `monotonic`, `monotonicline`, `alignedrectilinear`,
`line`, `grid`, `triangles`, `tri-hexagon`, `cubic`, `adaptivecubic`,
`quartercubic`, `supportcubic`, `lightning`, `honeycomb`, `3dhoneycomb`,
`lateral-honeycomb`, `lateral-lattice`, `crosshatch`, `tpmsd`, `tpmsfk`,
`gyroid`, `concentric`, `concentricinternal`, `hilbertcurve`,
`archimedeanchords`, `octagramspiral`, `rectilinear`, `zigzag`/`zig-zag`,
`crosszag`, `lockedzag`.

## Native classification (probe on time-cost.3mf, ironing_type=top)

| Pattern | Result | ironing_blocks |
|---|---|---|
| monotonic | ok | 84 |
| monotonicline | ok | 84 |
| alignedrectilinear | ok | 84 |
| line | ok | 85 |
| grid | ok | 116 |
| triangles | ok | 116 |
| tri-hexagon | ok | 116 |
| cubic | ok | 117 |
| adaptivecubic | **Native SIGSEGV** | 0 |
| quartercubic | ok | 115 |
| supportcubic | **Native SIGSEGV** | 0 |
| lightning | **Native SIGSEGV** | 0 |
| honeycomb | ok | 82 |
| 3dhoneycomb | ok | 73 |
| lateral-honeycomb | ok | 117 |
| lateral-lattice | ok | 114 |
| crosshatch | ok | 72 |
| tpmsd | ok | 74 |
| tpmsfk | ok | 70 |
| gyroid | ok | 74 |

`concentricinternal` is gated to internal-solid surfaces upstream
(`FillBase.cpp:72` "BBS: for internal solid infill only") and is **not reachable
through `ironing_pattern`**; `zigzag`/`crosszag`/`lockedzag`/`crosshatch`/`concentric`
already implemented (W1's chain) — `crosshatch` returns a real block count.

## Spec — mapping `ProcessInfillPattern` → Rust fill entry point

Every variant must dispatch from `fill_entities/ironing.rs` to the corresponding
Ares fill entry point, called with the ironing `FillParams` (density=1,
monotonic, overlap=0, dont_alternate_fill_direction propagated, link_max_length
= 3·spacing, `f->layer_id = layer.id`, `f->z = print_z`,
`bounding_box = object bbox`). All emitted polylines go into a `no_sort`
`FillExtrusionCollection` with `ExtrusionRole::Ironing`, the `mm3_per_mm` /
`width` / `height` already computed in `ironing.rs`.

| `ProcessInfillPattern` | Rust entry point | Upstream class | Native |
|---|---|---|---|
| Monotonic | `fill::rectilinear::surface::fill_monotonic_surface` | `FillMonotonic` | real |
| MonotonicLine | `fill::line::fill_line` (monotonic lines) | `FillMonotonicLines` | real |
| AlignedRectilinear | `fill::rectilinear_multiline::fill_surface` (multiline=1 path) or `fill_rectilinear_surface` | `FillAlignedRectilinear` | real |
| Line | `fill::line::fill_line` | `FillLine` | real |
| Grid | `fill::grid_trapezoidal::fill_surface` | `FillGrid` (trapezoidal sweep) | real |
| Triangles | `fill::triangles_trapezoidal::fill_surface` | `FillTriangles` | real |
| TriHexagon | `fill::triangles_trapezoidal::fill_surface` (tri-hexagonal mode) | `FillTriangles`-family | real |
| Cubic | `fill::multiline::sweeps` cubic-family sweep (`FillRectilinear.cpp` cubic sweeps; already used by `sparse_infill_pattern=cubic`) | `FillCubic` | real |
| QuarterCubic | `fill::multiline::sweeps::quartercubic_sweeps` | `FillQuarterCubic` | real |
| Honeycomb | `fill::honeycomb::fill_honeycomb` | `FillHoneycomb` | real |
| ThreeDHoneycomb | `fill::three_d_honeycomb::fill_surface` | `Fill3DHoneycomb` | real |
| LateralHoneycomb | `fill::honeycomb::fill_honeycomb` (lateral variant) | `FillLateralHoneycomb` | real |
| LateralLattice | `fill::honeycomb::fill_honeycomb` (lattice variant) | `FillLateralLattice` | real |
| CrossHatch | `fill::cross_hatch::fill_surface` | `FillCrossHatch` | real |
| TpmsD | `fill::tpms_d::fill_tpmsd_surface` | `FillTpmsD` | real |
| TpmsFk | `fill::tpms_fk::fill_tpmsfk_surface` | `FillTpmsFK` | real |
| Gyroid | `fill::gyroid::fill_surface` | `FillGyroid` | real |
| AdaptiveCubic | — return `UnsupportedProjectFeature` | `FillAdaptive::Filler` | **SIGSEGV** |
| SupportCubic | — return `UnsupportedProjectFeature` | `FillAdaptive::Filler` | **SIGSEGV** |
| Lightning | — return `UnsupportedProjectFeature` | `FillLightning::Filler` | **SIGSEGV** |

The three Native-SIGSEGV patterns stay `UnsupportedProjectFeature` — upstream
crashes in `FillAdaptive`/`FillLightning` `_fill_surface_single` when called
from `make_ironing` (the fills need a precomputed global `Octree`/`Generator`
context that `make_ironing` doesn't supply), so matching them would have to
either reproduce the crash or produce a bounded error; a clean
`UnsupportedProjectFeature` is the deliberate spec.

`Cubic`/`QuarterCubic` are dispatched through the `fill::multiline` cubic-
sweep family (`fill::multiline::sweeps::quartercubic_sweeps`, cited at
`FillRectilinear.cpp:3518`); the same entry should serve ironing once
`density=1, monotonic, overlap=0` is applied.

`ConcentricInternal` is not in the ironing vocabulary upstream (`internal_solid`
is a different key path); leave it out — a region with
`internal_solid_infill_pattern=concentricinternal` + `ironing_pattern=<...>` reads
from `ironing_pattern` only, not `internal_solid_infill_pattern`.

## Test plan (red)

New spec-driven test `filament_override_range_ironing_patterns` (or
rename the existing one to a generic coverage module):

- Fixture: `tests/parity/summary/time-cost.3mf` (Arachne, real top surface).
- For each of the 16 Native-emits patterns, build `time-cost.3mf` with
  `ironing_type=top` + `ironing_pattern=<pattern>` patched in, run Native
  `--slice 0` once, freeze `plate_1.gcode`; then run Ares on the same fixture and
  assert byte-equality under the official + seeded oracles.
- For `adaptivecubic`, `supportcubic`, `lightning`: assert Ares exits with
  `UnsupportedProjectFeature` (matching its current contract; Native SIGSEGV is
  upstream breakage, not a parity target).
- Existing patterns (rectilinear, concentric, hilbertcurve, archimedeanchords,
  octagramspiral, zigzag, crosszag, lockedzag) must stay byte-equal — covered by
  W1's chain.

## Deferred

- `alignedrectilinear` and `lateral-lattice` remain byte-unequal after the
  `inside_internal_surfaces` coord_t fix and the coord_t routed-length
  retraction gate — their first structural diffs are fill-structure/
  fill-ordering, not retraction/wipe:
  - `alignedrectilinear` seeded first diff at line 12799
    (`G1 X100.547 Y138.336 F60000` ordering divergence; 6,952,636 vs
    6,949,849 bytes) — evidence:
    `/tmp/w4-green-fk-seeded/ironing-pattern-alignedrectilinear-lP9Twb/`
  - `lateral-lattice` seeded first diff at line 12705 with a ~2MB missing
    block (9,899,728 vs 7,905,407 bytes) — a separate
    `FillLateralLattice`-family slice, not an ironing-delegation issue —
    evidence: `/tmp/w4-green-fk-seeded/ironing-pattern-lateral-lattice-xmdHui/`
  Both legs are removed from the coverage test and deferred to separate
  FillRectilinear-family rewrite slices.
- `fill_entities/ironing.rs:121-230` flow (area/inset/angle/nozzle selection,
  `iron_everything`/`iron_completely` paths) is already byte-equal per probing.
- Support ironing (`support_ironing_pattern`) is W3's territory and stays out of
  scope.
- Brim (`brim_type`, `brim_ears`, `brim_separation`) is W2's territory.
- The `f->overlap`, `f->z`, `f->layer_id`, `f->bounding_box` and
  `dont_alternate_fill_direction` channeling already exist in the current
  `ironing.rs` call sites — reuse them unchanged.

## Acceptance

- All 16 real-ironing patterns byte-equal `ares` ↔ Native under the official
  and seeded oracles on `tests/parity/summary/time-cost.3mf` + `ironing_type=top`.
- `adaptivecubic`, `supportcubic`, `lightning` return
  `UnsupportedProjectFeature("ironing_pattern")` — Ares-only upstream crash is
  bounded, not reproduced.
- `cargo clippy -D warnings` clean on `ares-core` + `ares-cli` + tests.
- `cargo fmt --check` clean.
- `cargo check --target wasm32-unknown-unknown -p ares-core -p ares-cli` clean.
