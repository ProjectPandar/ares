# Parity Residuals Milestone — 4 FAIL domains + 146 INCOMPLETE harness gaps

## Status: PLANNED (spec authored 2026-09-26; review cycle 3 in flight)

Current sweep state (`/tmp/parity-artifacts/option-coverage-summary.md`,
post `ec96c3f8`): **500 PASS / 4 FAIL / 146 INCOMPLETE** across option
domains; `ksr_fdmtest_v4` 84/84; all three completed review cycles
fixed and verified.

## Objective

Close the remaining 4 FAIL option domains (each a source-cited upstream
port slice, not a patch) and convert the 146 INCOMPLETE domains —
schema-unbounded options currently compared only at their default — into
real tested coverage per the goal's "all options" contract.

## Upstream boundaries (source-cited)

### FAIL domains

| Domain | Upstream boundary | Ares destination | Included / deferred |
|---|---|---|---|
| `ironing_pattern` + `sparse_infill_pattern` (concentric values) | `src/libslic3r/Fill/FillConcentricInternal.cpp` (Arachne variable-width rings from the medial-axis skeleton; `_fill_surface_single`, `offset2_ex` ring loop, first-offset quantization) + `FillConcentric.cpp:60-80` first-layer sliver suppression | `crates/ares-core/src/project_slice/fill_entities/concentric.rs` (currently rectangle-only `infills/concentric.rs`) | Included: full Arachne-ring concentric generator for arbitrary surfaces. Deferred: concentric-as-perimeter-loop ordering nuances inside narrow regions. |
| `fuzzy_skin_thickness` | `src/libslic3r/PerimeterGenerator.cpp` fuzzy-skin path fuzzification (`fuzzy_skin_thickness` option: point jitter generation, deterministic noise lattice) | `crates/ares-core/src/perimeters/fuzzy_skin.rs`, `project_slice/perimeters/classic/fuzzy_skin.rs` | Included: exact point-jitter algorithm. |
| `raft_layers` (max) | Object bottom-over-raft surface generation — `src/libslic3r/LayerRegion.cpp` `make_fill_surfaces`/`surface_type_detection` chain: `stBottom` classification + `surface.thickness` feeding `layerm.flow(frSolidInfill, thickness)` (`Fill.cpp:951-955`) + `erBottomSurface` (`Fill.cpp:904-975`) | `crates/ares-core/src/project_slice/prepare_infill/surface_type_detection/*`, `project_slice/raft/*` | Included: `fill_surfaces` region geometry/classification for bottom-over-raft so the emitted hatch, `path.width`, and downstream speed/slowdown all match. Verified deltas: 90° hatch-angle flip, `;WIDTH` 0.454819 vs 0.420844 (post-`with_spacing` `path.width`, `GCode.cpp:6794-6796`), F1913 vs F2100 (cooling slowdown on a different line population). |

### INCOMPLETE domains (146)

Harness gap, not implementation gap: `OptionPlan` marks domains
INCOMPLETE when `compared != cases.len()` or `!provably_legal`. 140/146
are schema-unbounded (`coFloat`/`coInt`/`coString`/`coPoints`/… with no
`min`/`max`) → only `baseline_case` (value=None default) runs and the
domain can never prove coverage.

Plan: add a `seeded` case for unbounded primitive domains —
`coFloat`/`coInt` → deterministic interior value over a nominal range
(`0` / `min(1.0, default*2+1)` / documented bound); `coString`/
`coEnum`-adjacent → legal-value probes where an implicit domain exists
(e.g. path-like strings, empty vs non-empty). For `coFloatOrPercent`
unbounded, also add the absolute-literal branch. `provably_legal`
becomes true for seeded+default coverage *per the goal's contract*
(the goal requires min/max+interior for ranges and full enumeration
for bools/enums; schema-unbounded options have no declared range —
seeded probing is the maximum achievable coverage).

## Constraints

- Every slice keeps `cargo clippy`, `cargo fmt`, `cargo nextest
  run -r -p ares-cli --test ksr_fdmtest_v4` (84/84) green.
- No `include!`/`include_bytes!`; files < 400 LOC; tests in `tests.rs`.
- Commit + push per slice, docs riding with code.
- Each FAIL-domain fix must verify its own option domain goes
  byte-identical, not merely "fewer diffs" — oracle binaries at
  `/tmp/medial-probe/*/bin/orca-slicer` and `scripts/orca-parity.sh`
  remain the authority.

## Exit criteria

1. `ironing_pattern` concentric, `sparse_infill_pattern` concentric:
   ring grid matches oracle (0.25-mm grid, correct ring count).
2. `fuzzy_skin_thickness` max: all case values byte-identical.
3. `raft_layers` max: byte-identical G-code incl. bottom-surface hatch
   family, `;WIDTH`, cooling-slowdown feedrates, and M73 alignment.
4. INCOMPLETE count → 0: every domain either PASS or FAIL with real
   comparisons (no `value.is_none()`/empty-case plans).
5. Independent review (gpt-reviewer or equivalent) re-verifies all
   slices with a clean six-axis report.

## Execution log

- [x] Seeded probes for unbounded coFloat/coInt/coFloatOrPercent/coPoint/
      coPoints (`b7753c4f`) — ~35 domains now run real comparisons.
      Remaining INCOMPLETE: coString/coStrings/coPointsGroups
      (no mechanically-legal probe) + 2 coBool/1 coEnum anomalies.
- [x] `concentric_sparse` coord_t truncation (`16ee106c`) — ring pitch
      integer-truncated like upstream.
- [x] `_adjust_solid_spacing` ported into concentric rings (`662b284b`) —
      concentric ring geometry byte-identical on stored cases.
- [ ] `ironing_pattern` residual = shared M73/estimator time bucket
      (R9 vs R10, ~+60s) — CoolingBuffer/time-model slice.
- [ ] `sparse_infill_pattern` residual = stInternalSolid sliver region +
      ring-lattice spacing (frInfill flow) + F971/F996 cooling slowdown —
      LayerRegion `expand_merge_surfaces`/`minimum_sparse_infill_area`
      + CoolingBuffer slices.
- [ ] `raft_layers` = same LayerRegion region-classification boundary
      (bottom-over-raft surface + hatch angle/width cascade).
- [ ] `fuzzy_skin_thickness` = `Feature/FuzzySkin/FuzzySkin.cpp` slice —
      classic noise is upstream `random_device`-seeded (non-deterministic;
      byte parity impossible — comparator envelope handles it);
      `ripple` type is deterministic but `fuzzy_extrusion_line_ripple`
      (Arachne junction/width-preserving, :221-283) and `displacement`
      mode dispatch have port gaps.
