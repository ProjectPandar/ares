# ARD: preserve independent spiral orientation and split conditions

Status: accepted for this bounded slice.
Upstream: OrcaSlicer `8500fcdccaa10b5099ac20d252af3a7c560046f1`.

## Decision

Port `libslic3r/GCode.cpp::GCode::extrude_loop:5754–5775` without conflating
its two gates. Orientation forcing excludes hole loops; configured spiral
nearest projected splitting does not. Rust `emit_perimeter` must use the
existing nearest-projection helper for holes as well as contours. Keep all
orientation behavior untouched. The earlier `placement_modes` all-spiral
SeamPlacer bypass remains correct and unchanged.

The existing emitter is a compatibility shell around this upstream caller;
its extra split role exclusion has no upstream authority. Do not invent an
Ares-owned pipeline or add a second split implementation.

## Non-negotiable qualification

- Commit SPEC, tests-only RED, minimal GREEN in order, atop frozen83e2e4f09.
- Agreed seam: unchanged `native_stream_case` over real Native replay and
  public validated API/CLI. No private oracle or newly added helper.
- All three Native cases double-replayed in both official/seeded modes before
  public calls. Exact whole outputs with existing generator-line allowance;
  additionally raw API==CLI. Never mask coordinates, times, IDs or wipe moves.
- Failure inventory: unstable Native; unsupported/rejected exported input;
  tests selecting no cases; false RED caused by admission failure; API/CLI
  mismatch; hole projection still wrong; normal seam/orientation regression;
  residual full-stream difference; stale/different binary.
  Retain attempts and stop on any unexpected category; no fallback or patching.
- Fresh actual Native exports are frozen as inputs with their normalization
  provenance; no generated Native stream is committed. Source GUI reachability
  is not a live GUI-save claim.
- Core stays platform-neutral; no filesystem/UI/runtime logic added. Validate
  formatting, clippy/workspace checks and wasm32 build.
- No rebase or merge in this slice; it lands jointly with the
  unchanged reset-comment prefix.

## Deferred

All SpiralVase arithmetic, nonspiral seam ranking, scarf, random placement,
perimeter geometry, travel/wipe changes, brim and processor changes remain out
of scope. Any remaining whole-file gap requires reporting before expansion.
See [SPEC](spiral-hole-projected-split-spec.md) for fixtures and exit criteria.
