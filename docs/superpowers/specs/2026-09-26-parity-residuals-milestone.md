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
- [x] `z_offset` (sweep-reopened): upstream applies `z_offset` at EVERY
      Z-target (`GCode.cpp:5692` `z = print_z + z_offset` in
      `change_layer`, `:3382` first-layer `travel_to_z`). The emit path's
      writer-domain z is now `print_z + z_offset` (`EmitState::layer_z`),
      keeping `;Z:`/`source_layer_z` model-space. `writer_z` +
      `will_move_z`/`retract_lift_above/below` evaluate in the same
      offset domain. `max_layer_z` now tracks `float(print_z)` running
      max (`GCode.cpp:4630`) instead of an f64 height sum — fixes the
      `{z_offset+min(max_layer_z+2, printable_height)}` end-gcode
      placeholder drift (15.9351 vs 15.9352). Case `z_offset=3.93515`
      byte-identical modulo timestamp; `z_offset=0` regression-free
      (ksr 84/84).
- [x] Probe-scope fix: unbounded `coInt(s)`/`coPoint(s)` probes are not
      provably-legal for structured/per-filament options (oracle reject
      or crash on `filament_map`, `physical_extruder_map`,
      `printable_area` degenerate polygon, `master_extruder_id` +
      `filament_self_index`). Reverted to baseline (stays INCOMPLETE by
      design); `coFloat(s)`/`coFloatOrPercent` seeded probes kept.
- [ ] Newly surfaced FAIL domains (post-seed): `retraction_speed`,
      `filament_retraction_speed` — new seeded probes exercise a path
      where retraction differs; pending triage against upstream
      `GCodeWriter.cpp`/`GCode.cpp` extruder retraction slices.
- [x] `retraction_speed`/`filament_retraction_speed` sweep FAILs —
      decomposition: (a) during-wipe cap `retractionDuringWipe` used
      `retraction_feedrate/60` (integer-round() of effective
      `retraction_speed`, mm/s) — upstream `GCode.cpp:350` uses the
      effective FLOATS value unrounded (`filament_retraction_speed`
      override folds into `config.retraction_speed`). Now plumbed as
      `retraction_during_wipe_speed`; (b) residual = short `m_wipe.path`
      tail from ares emitting upstream-dropped micro paths (~0.07mm
      `E.00232` sliver extrusions — LayerRegion elimination slice).
      `retraction_speed` case 009R78 now byte-identical modulo
      timestamp; `filament_retraction_speed` probe-min case reduced
      704→75 diff-lines, residual = micro-path elimination (shared
      LayerRegion boundary) + M73-only lines.
- [x] Review-cycle-4 fixes (z_offset writer-domain, commit 61930266
      follow-up): `GCode::preamble` silent `travel_to_z(z_offset)`
      seeds `writer_z` (`GCode.cpp:5676-5679`); spiral-vase emit +
      object-entry filter Z in writer domain; timelapse template-Z
      adopt/compare in writer domain (`GCode.cpp:5171-5178`); spiral
      travel float-gate uses `state.layer_z`; end-template `layer_z`
      = `writer_z - z_offset` (`GCode.cpp:3478`), `max_layer_z` keeps
      the f32 running-max semantics.
- [ ] Post-z_offset filtered re-sweep (2026-09-26, 564/650 PASS): 4
      residual FAIL domains, each an upstream-boundary port slice:
      (a) `ironing_pattern` — resolved by the half-spacing surface
      offset slice below. Earlier slice-boundary and flow-drift
      hypotheses were disproved by direct upstream probes.
      (b) `sparse_infill_pattern` — M73 mid-layer position shifts +
      upstream-dropped micro paths (LayerRegion micro-sliver slice).
      (c) `raft_layers` — same LayerRegion + M73 family (extra
      `M73 P2 R30` position diff at skirt).
      (d) `fuzzy_skin_thickness` — nondeterministic classic
      (`fuzzy_extrusion_line_classic` port + input polygon fidelity).
      `filament_retraction_speed`/`retraction_speed`/`z_offset` now
      PASS; `filament_retraction_speed` residual was folded into the
      LayerRegion micro-sliver bucket (sweep re-ran on fixed code).

## Concentric surface-offset rewrite slice

Source: `FillBase.cpp::Fill::fill_surface` (105–118) offsets each input
surface by `overlap - 0.5 * spacing` before calling the classic
`FillConcentric::_fill_surface_single`. `Fill.cpp::Layer::make_ironing`
sets overlap to zero. Rust destinations: `fill_entities/ironing.rs` and
`fill_entities/concentric.rs::append_configured` sparse branch. Existing
ring generation remains the `_fill_surface_single` implementation; its
callers now supply the correctly offset domain instead of bypassing
`Fill::fill_surface`. No new Ares-owned pipeline is introduced.

Included: split/empty offset domains, half-spacing ironing inset and
sparse-concentric overlap. Deferred: other sparse pattern residuals,
raft geometry, fuzzy-skin nondeterminism and complete printer coverage.

Evidence from case-0F6Hrc, using `tools/orca-probes/ironext.nix` and
`concrings.nix`: upstream polys and lslices both span ±5mm, ironing areas
span ±4.8mm, but the classic filler receives ±4.75mm with spacing 0.1mm,
multiline 1 and distance 100000 scaled units. This disproves the previous
classified-surface/closing-radius hypothesis. The extrusion delta follows
path length, not a flow-rate mismatch.

Plan and checks:
1. Apply the missing caller-side offset; keep ring generation unchanged.
2. Test the emitted first ring at two spacings; verify failure when the
   new offset is removed, then restore and rerun. Preserve ironing's
   `FillParams::dont_adjust=true` (`FillBase.hpp:67`), including domains
   split by the offset. Regression: 0.95mm-wide split lobes must retain
   100000-unit ring pitch, not the adjusted 105555-unit pitch. Also test
   sparse overlap contraction/expansion and an erased domain.
3. Keep source files below 400 lines using a `concentric/finalize.rs`
   module, not inclusion macros.
4. Run fmt, clippy, nextest and live Orca comparisons; independent review
   must pass before commit/push with this documentation.

Pre-split live evidence: `/tmp/parity-fix/option-coverage-summary.md`
reports ironing_pattern 2/2 PASS; sparse_infill_pattern remains FAIL
(first residual crosszag, M73 placement). KSR: 84 passed, 2 skipped.
These scoped results do not establish all-option or all-printer parity.
The sparse sweep also reports a concentric divergence at
`/tmp/parity-fix/case-9nONGI/manifest.json` (`;WIDTH:0.449999`); crosszag
is merely its summary's first failure, not its only failing value.

Independent review found the missing `dont_adjust` propagation and a
newly unused `Point` import. The split-neck test reproduced the pitch bug
(`/tmp/concentric-split-red.log`); the implementation now passes the
upstream flag explicitly: true for ironing, false for configured fill and
sparse anchor projection. No global disabling of spacing adjustment.

Post-review verification: 16 focused tests pass; release KSR 84 pass,
2 skipped; live ironing enum 2/2 strict comparisons pass. `cargo fmt
--check` and `cargo clippy` succeed (existing warnings remain). Full
`cargo nextest run --workspace --no-fail-fast`: 7302 passed, 2 failed,
10 skipped. Both failures are existing pipeline wipe tests: the same
failures reproduce in clean detached baseline 08dda8d1 at
`${TMP_ROOT}/ares-concentric-baseline`, log `/tmp/concentric-baseline-wipe.log`.
They are not waived for the overall parity goal; this slice does not
claim a green full workspace. Final logs: `/tmp/concentric-review-green.log`,
`/tmp/concentric-review-ksr.log`, `/tmp/concentric-review-oracle.log`,
`/tmp/concentric-workspace-full.log`.

## Wipe assertion follow-up

Boundary: `GCodeWriter.cpp::travel_to_xyz` delegates unchanged-Z travel
to `travel_to_xy` without forwarding its comment (771–780), already
ported in `gcode_move_emit.rs`. The two pipeline wipe tests still expected
`; travel`, unlike this upstream behavior. Plan: remove only that stale
suffix from their exact ordered-line assertions; keep all coordinates,
feedrates, extrusion amounts, wipe commands, and ordering checks intact.
No production movement change or source-text pinning test is added.
Verify the eight wipe tests and full workspace, then independent review
before committing these test corrections with this documentation.
Result: `/tmp/wipe-assertion-workspace.log` records 7304 passed and
10 skipped with no failures; fmt check and clippy also succeed. Skipped
opt-in parity jobs remain explicit coverage gaps, not inferred passes.

## Sparse concentric fixed-width materialization slice

Upstream boundary: `FillConcentric.cpp::_fill_surface_single` thick-line
fallback (141–145) converts classic polylines with integer `min_spacing`;
`FillBase.cpp::fill_surface_extrusion` then invokes `variable_width`, whose
`VariableWidth.cpp` constant-width run reconstructs extrusion width and
flow from that quantized spacing. Rust destination:
`fill_entities/concentric.rs::append_configured` sparse branch. The
existing fixed-path representation remains appropriate for uniform-width
rings, but reusing `params.flow` bypasses upstream quantization.

Plan: lock the 0.45mm nominal-width case to emitted 0.449999mm width,
then reproduce the constant-width conversion with f32 scaled spacing,
rounded-rectangle width reconstruction and `Flow::with_width`. Keep role,
ring geometry, overlap and clipping unchanged. Validate focused tests,
live sparse enum comparisons and KSR, then independent review and commit
with these docs. Deferred: other sparse patterns, nonuniform variable
width paths, raft/fuzzy residuals and broader option/printer coverage.
Reference case `/tmp/parity-fix/case-9nONGI` contains missing WIDTH tags;
it must be rerun rather than treated as current binary evidence.

Verification: `/tmp/sparse-flow-red.log` records the width regression;
`/tmp/sparse-flow-green.log` records 17 passes. Current live sweep
`/tmp/sparse-flow-oracle.log` now reports concentric PASS, with 20/26
sparse pattern values passing and six divergent (crosszag, honeycomb,
lightning, line, lockedzag, quartercubic). Thus the enum domain remains
FAIL, not complete. `/tmp/sparse-flow-workspace.log`: 7305 passed,
10 skipped; `/tmp/sparse-flow-ksr.log`: 84 passed, 2 skipped. Fmt and
clippy succeed with existing warnings. Independent source review found
no code issues; these completed run logs resolve its pending evidence
note for the concentric slice only.

## Quarter-cubic shift arithmetic slice

Source: `FillRectilinear.cpp::FillQuarterCubic::fill_surface` (3518–3548).
Rust destination: `fill/multiline/sweeps.rs::quartercubic_sweeps`.
Included: promote Boost's **float** `one_div_root_two` constant to double;
preserve fractional `scale_(z)` and `scale_(spacing)` until the upstream
integer casts. Existing Rust sweep dispatch stays in place; replace only
its prematurely truncated intermediates and double-precision constant.
Deferred: multiline chaining, other pattern residuals, timing estimation
and printer/option coverage expansion.

Plan: compare a small C++ arithmetic probe using Boost float constants
against a separate Rust regression at z=1.2, 1.2000009 and 9.8; establish
red, apply the source-equivalent computation, then run focused tests,
KSR and live quarter-cubic output comparisons. Review independently
before code-and-doc commit. Probe `/tmp/quarter-shift.cpp` yields period
10855456 and dx2 scaled units 2330962, 2330961, 3750156 respectively
(spacing 0.40707963705062866, density 0.15f, multiline 1).

Results: red dx2 was 2.330956 versus expected 2.330962; focused nextest
28 passed. `/tmp/quarter-shift-workspace.log`: 7306 passed, 10 skipped;
`/tmp/quarter-shift-ksr.log`: 84 passed, 2 skipped. Fmt/clippy succeed.
Current live sweep `/tmp/quarter-shift-oracle.log` confirms quartercubic
PASS. The sparse enum still fails: crosszag, honeycomb, lightning, line,
lockedzag and (newly observed in this run) tpmsfk. The latter's first
mismatch is an Internal Bridge travel Y112.867 vs Y112.9 in
`/tmp/parity-quarter-shift/case-kqy7Pi/manifest.json`; its cause remains
unverified. Do not infer whole-enum success or nondeterminism from the
previous run's tpmsfk PASS.

## Strict reference acceptance repair

This is a test-harness correction, not a new slicing pipeline. Preserve
ARD-0023's ordered-byte requirement: neither equal line multisets nor an
inter-oracle distance envelope proves output equality. Remove the envelope
PASS override and its extra `8 * budget` attempts from
`tests/orca_parity.rs::compare_exported_for`; remove the now-unused multiset
helpers from `ksr_fdmtest_v4/golden.rs`. The existing generator-only
normalization and bounded exact-reference retries remain unchanged.
Each PASS establishes an actual matching reference, not stable oracle output
or completion of any printer/option inventory.

Plan: exercise the actual comparison/retry entrypoint with a subprocess
oracle fixture containing two swapped G1 lines from a real Ares slice.
The line multiset is identical but motion order is not. Require DIVERGENT,
exactly the configured two oracle calls, and complete paired byte artifacts.
Also verify immediate exact equality and equality on the second reference.
Run focused tests, workspace/KSR and a live lockedzag comparison, then obtain
independent review before committing code and documentation together.

Diagnosis evidence: `/tmp/locked-repeat-counts.txt` records 44 fresh slices
of the same `case-LOJH3N/input.3mf`; occurrences of `WIDTH:0.405676` vary
between 2, 3 and 4. Sequential samples were stable, concurrent samples were
not. Existing domain probes reproduce identical triangular inputs and the
short paths. `FillConcentricInternal.cpp:29-36` leaves members of
`Arachne::WallToolPathsParams` uninitialized, while
`WallToolPaths.cpp:693` reads `is_top_or_bottom_layer` and
`min_length_factor`. This is a candidate explanation, not a proven causal
attribution; no speculative production short-line filter is added.
The lockedzag production residual remains open.

Red evidence: `/tmp/strict-reference-red.log` reports PASS instead of
DIVERGENT for reordered G1 lines, with envelope distance 0 and radius 0.
Earlier summary PASS totals using that override are not proof of strict
parity; they require re-verification with the corrected acceptance path.

Verification: `/tmp/strict-reference-green.log` records four passing tests
(three parent scenarios plus the subprocess entrypoint); the shell-driven
mock scenarios are Unix-only. Platform-neutral strict comparator regressions
remain shared. Workspace: 7310 passed, 10 skipped
(`/tmp/strict-reference-workspace.log`); release KSR: 84 passed, 2 skipped
(`/tmp/strict-reference-ksr.log`). Fmt and clippy complete successfully, with
existing clippy warnings. Live lockedzag remains DIVERGENT after exactly
three reference attempts (`/tmp/strict-reference-oracle.log` and
`/tmp/parity-strict-reference/case-iXFOTg`); this expected residual is not
reported as a successful option domain. All touched Rust files are below
400 lines; removal reduces `orca_parity.rs` from 484 to 352 lines.

## Adaptive cubic scaled-endpoint conversion slice

The strict sparse sweep (`/tmp/strict-sparse-oracle.log`) finds 18 of 26
values matching at least one reference within three attempts; adaptivecubic
and supportcubic join the eight divergent values. Closest adaptive reference
`/tmp/parity-strict-sparse/case-ifuYpy` differs only in one wipe E word:
`-.38885` versus `-.38886`. Independent upstream pre-wipe probes expose
integer endpoint differences before the wipe, so do not patch E formatting.

Source boundary: `FillAdaptive.cpp::generate_infill_lines_recursive:484`
calls `Point::new_scale(Vec2d)`; `Point.hpp:206` truncates scaled doubles
with `coord_t(...)`. Rust destination:
`fill/adaptive/filler.rs::generate_infill_lines_recursive`. Replace its
four rounded conversions with the existing checked truncating conversion.
Included: positive/negative generated endpoints and both coordinate scales.
Deferred: octree construction, adaptive hooks, other pattern discrepancies,
and oracle stability. The existing Rust generator remains the upstream
compatibility boundary, not a newly designed fill pipeline.

Plan: test raw generated endpoints at the recursive generator seam before
clipping, observe rounded Y4363021 versus truncated Y4363020 (red log
`/tmp/adaptive-trunc-red.log`), then apply conversion, run adaptive tests,
strict adaptive/support-cubic live comparisons, workspace/KSR, fmt/clippy,
and independent read-only review before code-plus-doc commit.

Results: focused adaptive tests 12 passed (`/tmp/adaptive-trunc-green.log`);
workspace 7311 passed, 10 skipped (`/tmp/adaptive-trunc-workspace.log`);
release KSR 84 passed, 2 skipped (`/tmp/adaptive-trunc-ksr.log`). Fmt and
clippy pass with existing warnings. Both adaptivecubic and supportcubic now
PASS strict ordered-byte comparisons (`/tmp/adaptive-trunc-oracle.log`,
artifacts `/tmp/parity-adaptive-trunc`). The value-filtered option domain
correctly stays INCOMPLETE because only 2/26 values were requested; this
is not a complete sparse enum or all-options acceptance.
A subsequent full strict enum run (`/tmp/adaptive-trunc-full-oracle.log`)
again passes both affected values but reports 16 PASS / 10 DIVERGENT overall.
Concentric, cubic, lateral-honeycomb and triangles differ in this run despite
previous exact matches. No causal attribution to this isolated adaptive
change is established; preserve the run-specific results, not a cumulative
claim that all previously observed matches are stable.
