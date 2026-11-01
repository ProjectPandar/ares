# Tier1 beta Native supplier cutover

## Decision and scope

The only active source/resource authority is OrcaSlicer v2.5.0-beta commit
`345efec64c7b95a05ce5cf96df0dde66a158835c`. Tier1 checkouts, source archives,
Native recipes, source-kernel extractions and the official AppImage must move
together. No2.4.2 fallback or mixed-version supplier graph is permitted.
This is oracle provisioning for the existing source-cited Rust rewrite, not a
new slicing milestone or Ares-owned pipeline. No Rust behavior, test assertion,
fixture, comparison count or browser timeout changes here.

## Upstream boundaries

- `src/libslic3r/GCode.cpp:GCode::_do_export` and
  `Feature/FuzzySkin/FuzzySkin.cpp:random_value`: retain the declared replay seed.
  `Arachne/WallToolPaths.hpp:WallToolPathsParams` and `Print.hpp:m_id`
  now initialize the previously patched members upstream; retire those insertions.
  The initialized KSR authority retains its narrower FuzzySkin-only seed patch;
  the full replay/statistics authority also seeds the GCode placeholder RNG.
- `GCode/GCodeProcessor.hpp:OptionsZCorrector::on_moves_inserted` and
  `GCodeProcessor.cpp:calculate_time`: retain beta's own sorted-vector index
  remap. Do not stack the old map-based stable-options patch. The distinct raw
  and stable supplier names remain separate derivations; their former patch
  difference disappears because the correction is now upstream, not by aliasing
  executable paths or substituting one authority for another.
- Existing dump seams in `src/OrcaSlicer.cpp:main`, `CLI::run` and
  `GCodeProcessor::reset/apply_config` remain read-only diagnostics plus the
  explicitly declared pressure-advance initialization/travel-limit alignment.
- `src/libvgcode/src/ViewerImpl.cpp:get_estimated_time_at/load`, actual-speed,
  role-time, segment-position/width/height and color methods, and
  `src/slic3r/GUI/GCodeViewer.cpp` custom overview: re-extract the beta source.
  In particular preserve the new layer prefix-time caches, line/joint vertex W
  data, and ordinary non-belt overview cache branch. These are suppliers for
  existing `ares-vgcode`/`ares-wasm` public data comparisons, not viewer runtime.
- Upstream `CMakeLists.txt`, `src/libslic3r/CMakeLists.txt`, `deps/OCCT`,
  `deps/SLVS` and `deps/wxInspector` define the beta build dependency boundary.
  Use pinned upstream archives/hashes and repository-relative packaging patches,
  not coordinator store-path references. Keep native-only GUI/CAD dependencies
  in the oracle build; they do not enter platform-neutral Rust crates.

## Delivery invariants

All existing producer variants and retained public comparison selections remain
explicit. Seed ELF/debug/source must belong to one beta derivation. The official
release asset has an independently pinned digest. No compiler output, native
G-code or expected-output golden is committed. The mandatory exact-run cache
handoff, miss failure, content-hash validation, official-only dependency
substitution and no-local-Native-build consumer contract remain unchanged.

The prior2.4.2 capacity result is historical, not a beta result. Re-evaluate the
beta archive footprint before pushing; do not silently transfer the old bound.
A verified source archive and recipe/patch/extraction preflight are not a claim
that beta Native compilation or the integrated comparisons have already passed.

## Deferred and landing order

No new GUI/CAD/belt behavior, Rust parity repair, private tests or browser E2E
repair is included. WEB owns its existing selector/mobile repair; it lands first
on main. This local cutover commit is reviewed without pushing, then PR4 rebases
on that main and CI must be entirely green before REVIEW-REQUEST. No waiver.
