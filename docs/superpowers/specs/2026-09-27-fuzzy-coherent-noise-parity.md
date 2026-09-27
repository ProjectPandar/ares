# Fuzzy skin coherent noise types + seam candidates on fuzzified loops

Date: 2026-09-27. Boundaries:

- `OrcaSlicer/src/libslic3r/Feature/FuzzySkin/FuzzySkin.cpp` `get_noise_module`
  (:41-64) and the libnoise modules (`noise::module::Perlin`, `Billow`,
  `RidgedMulti`, `Voronoi`) vs `crates/ares-core/src/perimeters/fuzzy_skin_coherent_noise.rs`.
- `OrcaSlicer/src/libslic3r/GCode/SeamPlacer.cpp` `place_seam` (:1500-1624)
  operating on the post-`apply_fuzzy_skin` `ExtrusionLoop` points vs
  `crates/ares-core/src/project_slice/seam_placement/` +
  `crates/ares-core/src/project_slice/seam_candidates.rs`.

## Problem (r10 evidence)

`option/fuzzy_skin_noise_type/{classic,perlin,billow,ridgedmulti,voronoi}` —
now actually exercising fuzzy geometry after the activator fix — diverge by
~9–13k lines each. First difference is a seam/entry-vertex pick on the inner
wall (orca enters at `(105.73,105.73)`, ares at `(114.27,105.73)`); once the
split vertex differs, every emitted wall coordinate shifts and the diff
cascades. `fuzzy_skin_{thickness,scale,ripple_*}` cases remain byte-exact, so
the ripple/classic-noise paths and the plain (unfuzzified) seam picks are
correct.

## Work items

1. **Confirm the seam-candidate source is upstream-faithful.** Upstream seam
   candidates are the *fuzzified* `ExtrusionLoop` polyline points
   (`layer->perimeters` entities built from post-`apply_fuzzy_skin`
   polygons, `PerimeterGenerator.cpp:151`); verify our
   `seam_candidates::generate_regions` input is the post-fuzzy materialized
   loop (it is — entities reach `seam_placement::apply` after
   `fill_entities::prepare`), and that the split is applied via
   `split_at_vertex(seam_point, 1.5 µm)` fallback `split_at(seam_point,
   prefer_non_overhang=true)` (`SeamPlacer.cpp:1620-1624`).
2. **Inner-loop seam projection parity.** Upstream's inner-perimeter branch
   (`SeamPlacer.cpp:1564-1618`) uses `local_ccw_angle`-driven
   concave/convex depth math plus `staggered_inner_seams` walk. Diff our
   `runtime.rs:303-347` branch against it line by line — the missing piece is
   likely the concave corner overshoot (`depth = 1.4142*depth/beta`,
   `dir_to_middle`) and the `(seam_position - candidate.position)² < depth`
   gate.
3. **Coherent noise modules.** `fuzzy_skin_coherent_noise.rs` implements a
   hand-rolled hash noise that is *not* libnoise. Byte parity for
   perlin/billow/ridgedmulti/voronoi requires porting the upstream libnoise
   modules — including `SetSeed(0)` state, perlin gradient table, voronoi
   feature-point jitter (`displacement=1.0`), octave normalization, and the
   freq/persistence/octave knobs — from `OrcaSlicer/deps_src`/`OrcaSlicer/deps`
   libnoise sources, source-cited per module. `classic`
   (`UniformNoise::GetValue` → `random_value`, `FuzzySkin.cpp:25-38`) is a
   thread-local `mt19937` seeded from `std::random_device`; empirically the
   oracle binary reproduces identical output across runs on this platform
   (the `rd.entropy()==0` → `hash(thread::id)` fallback path is presumably
   active, and all classic-mode sweep cases passed byte-exact in r7/r8), so
   classic is deterministic in practice here. The divergent `noise_type/classic`
   case is therefore the seam-vertex pick in work item 2, not the noise.
4. **Re-run** `ARES_OPTION_KEYS=fuzzy_skin_noise_type` sweep and confirm
   byte-exact (or justified exemption for classic); then the full option
   sweep + printer sweep.

## Constraints

- `<400 LOC` per source file; tests in `tests.rs` mods; no `include!`/
  `include_bytes!` for source splitting.
- libnoise port must be byte-faithful (f64 gradients, table sizes, seed
  permutation) — the emitted G-code is compared byte-for-byte.
- No fs/env diagnostics inside `ares-core` (per review finding F7); keep
  probes in `ares-cli` tests or `xtask`-style helpers.
