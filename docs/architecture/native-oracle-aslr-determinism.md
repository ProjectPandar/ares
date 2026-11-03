# Native Oracle ASLR Determinism (Variant A) — ARD

Status: SPEC rev.2 (pending REVIEW approval; no implementation yet)
Branch: `w5-native-variant-a` (from `c22f5a81d`)
Investigation artifacts under `${TMP_ROOT}/ares-w5-gates/nso/` (not committed).

## Problem

The official OrcaSlicer 2.4.2 oracle (`/tmp/orca-official-242/squashfs-root/bin/orca-slicer`)
intermittently emits a variant G-code body for a fixed 3MF input. The variant differs from
the normal body in exactly two localized ways (fixture: `normal-support/interface-fan/below-baseline`,
worker6 capture `case-BUCqyF`):

1. Two adjacent `;TYPE:Internal solid infill` extrusion blocks (widths 0.46426 / 0.437637)
   are emitted in swapped order; the connecting travel target flips between
   `X113.388 Y113.135` and `X113.244 Y113.297`.
2. An `M73` progress line shifts by one extrusion line.

Ares output is byte-stable and equals the normal native body (`bb96a503…`). The variant
(`b818bbc2…`) has been observed in CI (threads8) and in worker fulls across four unrelated
tests sharing the fixture family. No Ares change can or should track it: the flip is Native UB.

## Root cause (candidate; acceptance rests on empirical statistics, not on this mapping)

valgrind memcheck (single core, `LC_ALL=C`, wrapper env replicated) shows:

- Uninitialised-value origin (symbol-level): **stack allocation in
  `Slic3r::ClipperLib::Clipper::BuildResult2(Slic3r::ClipperLib::PolyTree&)`**.
  The binary carries no DWARF; source mapping is by candidate analysis, not proof:
  - `deps_src/clipper/clipper.cpp:2801-2852` (`Clipper::BuildResult2`, OrcaSlicer v2.4.2 vendored Clipper1)
  - candidate: the only stack-constructed object in that frame is the `PolyNode()` temporary at
    **`clipper.cpp:2824`** (`polytree.AllNodes.emplace_back(PolyNode());`), whose constructor
    (`deps_src/clipper/clipper.hpp:147`) leaves **`m_jointype` / `m_endtype`
    (`clipper.hpp:160-161`) uninitialized** (`Slic3r::Point()` is zero-initializing,
    `src/libslic3r/Point.hpp:192` — excluded).
  - The chain *uninit `PolyNode` member → infill block-order swap* is **not proven**;
    it is the leading candidate consistent with all observations. This slice's
    acceptance criterion is the empirical A/B statistic below, not the causal story.
- Consumer (canonical record, ×22 in the log):

      Conditional jump or move depends on uninitialised value(s)
        at scalable_free
        by Slic3r::_clipper_pl_closed<vector<Polygon, tbb::scalable_allocator<Polygon>>…>(…)
        by Slic3r::intersection_pl(Polygons const&, Polygons const&)
        by Slic3r::traverse_loops(…)                      [src/libslic3r/PerimeterGenerator.cpp:100,
                                                           intersection_pl call at :182, recursion :254]
        by Slic3r::PerimeterGenerator::process_classic()
        by Slic3r::LayerRegion::make_perimeters(…)
        by Slic3r::Layer::make_perimeters()
        by tbb::start_for<blocked_range, PrintObject::make_perimeters()::$_0>::execute   ← TBB parallel_for
        by Slic3r::PrintObject::make_perimeters()
      Uninitialised value was created by a stack allocation
        at Slic3r::ClipperLib::Clipper::BuildResult2(PolyTree&)

  Second consumer chain at 3MF import: `_BBS_3MF_Importer::_load_model_from_file →
  _generate_volumes_new → ModelObject::add_volume` (×36).

- Under valgrind the layout is fixed and the variant reproduces deterministically
  (2/2 runs emit `b818bbc2…`), confirming layout dependence of the oracle output.

## Statistical evidence (all single-core `taskset`, natural load, E2 trigger =
two concurrent Native invocations pinned to one CPU)

| Arm | Variants / samples | Notes |
|---|---|---|
| E1 idle baseline | 0/64 | load1 ≈ 4.7 |
| E2 control (ASLR on) | 1/128 | the single variant byte-identical to incident capture; load1 4–5 |
| E2p paired control arm | 0/256 | load1 5.6–7.0, interleaved with setarch arm |
| E3 / E3b setarch -R | 0/1280 | load1 4→8.7 across batches |
| E2x4 four Native on one core | 0/256 | inter-process contention does not raise the rate |
| E4 `MALLOC_PERTURB_=1` | dominant hash shifts entirely + 4/512 outliers | allocator-layout sensitivity |
| **Paired 2048/arm (closed early at W4 exclusive-full boundary, blk16/48; interrupted partial block discarded)** | **control 9/2048; setarch 0/2048** | control first hit blk3; blocks interleaved under matched load |

Closing statistic (recorded correctly, both readings):

- Two-arm comparison, conditional binomial / Fisher exact on the paired result
  (9 vs 0 of 2048+2048 trials): p ≈ 0.5⁹ ≈ **0.002** — this is the
  headline figure; it conditions on the observed totals and does not assume
  the control rate is known.
- Naive one-arm reading (0/2048 against a control rate treated as the fixed
  constant 9/2048): p ≈ e⁻⁹ ≈ 1.2×10⁻⁴ — **overstates** significance
  because it ignores the control-rate uncertainty; reported for completeness
  only, not the acceptance basis.
- Across all ASLR-off experiments (E3/E3b 0/1280 + paired 0/2048), the
  setarch -R arm totals **0/3328**.

Conclusion: `setarch -R` removes Variant A — strong but statistical evidence
(no deterministic proof; per ruling the causal mapping stays candidate-level).

Mechanism (candidate wording): the undefined bits plausibly come from the executing
thread's stack; stack contents vary with ASLR layout and scheduling. Fixing the address
layout (`setarch -R`, `ADDR_NO_RANDOMIZE`) makes the draw constant; across all ASLR-off
samples the output equals the normal body `bb96a503…`, i.e. the determinized
oracle equals what Ares already produces.

## Decision (proposed, awaiting approval)

Determinize the Native oracle invocation by disabling ASLR **at the Native exec site
only** — the Ares test binary and all other processes keep default ASLR.

Committed side (this repo):

1. `scripts/orca-parity.sh` — final `exec "$APPDIR/bin/orca-slicer" "$@"` (currently :48)
   becomes `exec setarch -R "$APPDIR/bin/orca-slicer" "$@"`.
2. `scripts/orca-seeded-parity.sh` — the Native exec `exec taskset -c "$ORCA_REPLAY_CPU"
   "$ORCA_REPLAY_BIN" "$@"` (currently :17) wraps the binary with `setarch -R`.

Local side (not committed; changes the canonical full's oracle, sha256 recorded for
coordination broadcast):

3. `/tmp/orca-wrapper-inherit-cpu.sh` — its exec becomes `setarch -R`-wrapped the same way.
   Before/after sha256 of this file will be recorded in the implementation report and
   broadcast by the coordinator.

Explicitly NOT modified:

- `scripts/ci/pin-runner.sh` (W4-owned repo file; CI pin runner). `setarch -R` is not
  applied to the test binary — only to the Native oracle exec sites above, so no Ares
  process or unrelated test is affected.
- Canonical full script `${TMP_ROOT}/ares-parity-control-set-v2.sh`: unchanged; it invokes the
  same wrappers, so its oracle becomes deterministic through them. Not touched before
  W4's `96eb17ccc` merges and this SPEC is approved.
- **No Ares production or test code changes. No assertion, comparator, or golden-file
  changes.** Ares already matches the determinized oracle output.
- No upstream OrcaSlicer/Clipper1 fix in this slice (report-only candidate:
  `PolyNode` ctor leaves `m_jointype`/`m_endtype` uninitialized,
  `deps_src/clipper/clipper.hpp:147,160-161`).

Failure semantics: no toggle, no fallback, no environment-variable off-switch. If
`setarch` is unavailable on a machine, the wrappers exit non-zero with a clear message
naming the missing dependency.

## RED / GREEN validation plan (TDD, after SPEC approval and W4 merge)

- RED (no determinization yet): the paired statistical run above plus a directed
  reproduction leg: run the `option_coverage` group for the four affected tests
  repeatedly under the E2 trigger shape; at least one native-side mismatch is captured.
  If the control arm of the paired run finishes at 0 hits, the claim is downgraded
  to inconclusive per ruling and reported as such.
- GREEN: enable `setarch -R` at the three exec sites, then:
  1. rerun the same directed legs (official + seeded, fresh cache/root per run,
     threads2) expecting 0 mismatches;
  2. **a two-mode canonical full must pass** — with ASLR off every fixture's Native
     output is fixed; any test that now fails deterministically is a real discrepancy
     between Ares and the determinized oracle, to be reported and triaged, never
     hidden, skipped, or worked around in this slice.
- Gates: strict clippy/fmt/wasm32 unaffected (no Rust changes); directed two-mode and
  the canonical two-mode full remain the merge evidence, all threads2 canonical.

## Included / deferred

- Included: the three Native exec sites above; validation legs; wrapper sha256
  recording; this ARD.
- Deferred: pin-runner/CI runner changes (none), canonical script edits (none),
  upstream Clipper1 fix, seeded-replay binary re-pin, threads16 policy (ruled
  separately).
