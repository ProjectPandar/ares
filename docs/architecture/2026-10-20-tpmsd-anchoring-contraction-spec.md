# TPMS-D sparse anchoring: base surface contraction

Status: SPEC-REVIEW-REQUEST; independent discovery only, no public RED or
production changes. Base/current main:
`8acb19cf79998fbff7fe873d803690bd5c5d8ec7`.

## Upstream owner and exact Rust boundary

This is a libslic3r filler-lifecycle rewrite slice, not a new anchoring
pipeline or another TPMS-D numeric/rotation algorithm.
OrcaSlicer `8500fcdccaa10b5099ac20d252af3a7c560046f1` owns:

- `Fill.cpp:1394-1504`,
  `Layer::generate_sparse_infill_polylines_for_anchoring` creates the filler
  for the surface and uses zero overlap. `:1477-1497` explicitly generates
  through `f->fill_surface`, resetting the fill spacing per expolygon.
- `FillBase.cpp:105-118`, `Fill::fill_surface`: offset the surface by
  `float(scale_(overlap - 0.5 * spacing))`; independently call the pattern
  kernel on each resulting component, concatenating output in that order.
- `FillTpmsD.hpp:14-32`: inherits Fill and overrides only the kernel, not
  `fill_surface`. `FillTpmsD.cpp:101-160` consumes the contracted domain for
  corrected-angle rotation, bounds/grid, waves, multiline, clipping and
  connection. Those algorithms are already ported and are not this slice.
- `PrintObject.cpp:2751,2848-2937,3242-3251` projects sparse anchors and uses
  their directions to construct internal bridge geometry. It is a consumer,
  not authority to change angle selection or sampling.

Rust destination:
`project_slice/prepare_infill/bridge_over_infill/sparse_anchoring/
arms_tail.rs`, only its `ProcessInfillPattern::TpmsD` arm (`:133-151`).
It passes the uncontracted expolygon directly to `fill_tpmsd_surface`.
The kernel `fill/tpms_d.rs:34-110` intentionally implements the pattern
operation, not base `fill_surface`; ordinary emission already applies the
correct contraction in `fill_entities/tpms_d.rs:36-45`. The neighboring
TPMS-FK anchoring arm also retains that base lifecycle.
The adapter is a temporary compatibility shell around upstream Layer/Fill,
not an Ares-owned lifecycle. Restore the missing base contraction here;
do not redesign the shared kernel or duplicate another pipeline adapter.

## Native-first bounded witness

Use the previously approved authored Honeycomb anchoring mesh/profile
bundle as input provenance (10mm model, direction45, sparse20%, layer0.2).
Change only pattern to `tpmsd`, multiline, top-shell count, and the two ZAA
bits to0. Keep all other metadata/ZIP members. ZAA is disabled because
TPMS-D reads its raw angle; this is not another ZAA-direction slice.

A bounded initial official check with multiline1 and top5/6/7/8 was exact.
Multiline2 at top6 exposed a repeatable delta; adjacent top5 and top7 were
then measured without any mesh search. Do not discard the second observed
failure at top7. The four proposed public inputs were independently run in
canonical official/seeded modes, with two fresh Native references per input:

| top | multiline | input SHA256 | outcome, both modes |
|---|---|---|---|
| 6 | 2 | `1e250882f206fd72c46c8a89349cd0e6ce79c3cfe392b819c188c0b3a87bc46f` | DIVERGENT |
| 7 | 2 | `cb6346529e9aa6455c46b90d33a95b2fa25d05bd09f648b9898615027e29c9c4` | DIVERGENT |
| 5 | 2 | `68062dd99ad802baab49e21b2ab2c0cd8746f576c3e7ae1193671068cf5aa63f` | PASS |
| 6 | 1 | `874443522220d36b71b0c717cb8a93498540f0d018da789d1fe7be240f052989` | PASS |

All16 Native runs admitted pattern/multiline/top-shell/ZAA footer values.
Repeats and normalized cross-mode complete bytes agree. Both controls match
Ares CLI with the unchanged generator-only whole-file comparator. Public
API admission/equality for these new inputs remains future RED work.

Top6/multiline2 first strict difference: byte99016, line4192,
Native `G1 X114.003 Y114.003 E.05321`, Ares same XY with `E.05322`;
normalized lengths134835/134830. This occurs in Internal Bridge at physical45.
All1756 ordinary Sparse motion rows match; both sides have44 positive bridge
extrusion rows there.

Top7/multiline2 first strict difference: byte98251, line4155,
Native `G1 X113.962 Y105.997 E-.3025`, Ares same XY with `E-.30251`;
lengths135416/135418. It is a following-layer wipe/retraction command at
physical45 after a positive Internal Bridge on physical44 (48 rows each).
All1709 ordinary Sparse rows match. These are complete-command differences,
not a claim that the first formatted XY tokens differ. No E tolerance,
rounding patch, M73 filtering or semantic substitute is an acceptance gate.

## First differing intermediate, not an E-formatting fix

Read-only debugger observations on top6/multiline2:

- At Native TPMS-D anchoring entry, layer43 / Z8.8, spacing
  `0.40707963705062866`, density0.2, multiline2; the domain is
  `[-4019688,4019693]` on each axis.
- Ares enters the same TPMS-D kernel from the anchoring tail with identical
  spacing/Z/density/multiline/angle settings, but domain
  `[-4223228,4223233]`. The backtrace identifies the raw-expolygon call.
- The difference is203540 scaled units per side: exactly the rounded
  zero-overlap half-spacing offset required by FillBase.cpp:108.
- Later ordinary emitted Ares TPMS-D enters from `fill_entities/tpms_d`
  with contracted `[-3612599,3612604]`; its actual Sparse output already
  matches Native. Do not add contraction inside the shared kernel, which
  would contract that caller twice.

This identifies the first inconsistent filler input and rejects an ordinary
Sparse/multiline kernel mismatch as the observed output owner. Downstream
unformatted bridge/wipe precision can expose last-decimal E differences;
it does not authorize touching E arithmetic or formatting. Complete closure
of both output deltas by the proposed domain port is **not yet claimed**.
Any residual after that bounded port means STOP and report, not broaden.

Probes use the actual seeded reference + existing matching DWARF and the
already verified main8acb19cf7 Ares CLI. No nix/debug build, gcroot, debugger
state write or production instrumentation. Some unused optimized debugger
fields were unavailable; a bounded entry-only probe captured the live domain
and settings. Failed exploratory reads are retained externally, not used as
acceptance evidence. Canonical fresh references alone supply the verdict.

## Separate RED / GREEN contract

Only after SPEC approval, freeze four SHA-pinned input-only3MFs from the
matrix: retain both observed multiline2 targets plus phase/count controls.
Reuse the existing shared `runner/anchoring_zaa` frozen/verify/run_once/
observer harness with dataset-specific case/expectation data; do not copy
whole harness files or weaken Honeycomb/Line assertions or inputs.

Require public loaded effective settings and Native footer, positive
Internal Bridge at physical `51 - top`, unchanged ordinary Sparse motion,
two fresh Native references, stable repeats, and raw public API/CLI equality
before unchanged strict whole-output equality. Expected RED in each mode:
two target FAIL and two controls PASS. All existing Honeycomb/Line cases remain
controls. Submit exact clean RED hash for separate review before production.

GREEN only applies `(-0.5 * spacing / scale.factor()) as f32` and the existing
Miter3.0 offset/error mapping before the TPMS-D anchoring kernel, iterating
each component independently in original order. Preserve every TpmsDFillParams
value, float boundary, phase/Z, density, multiline expansion, wave equation,
clipping/connection, index/reference, flow and all other pattern arms.
`arms_tail.rs` has room below400LOC; no module split/new helper/API is needed.

Deferred: corrected-angle rotation already ported by
`2026-10-14-tpms-d-rotation-precision-spec.md`, ordinary emission/kernel
changes, bridge-angle/sampling algorithms, E/flow/rounding, wipe/retraction,
M73/time/cooling, support/ironing/raft/tree, other patterns, geometry search,
capability/guard/fallback changes and any independent pipeline design.
Do not add allow/expect, skip paths or UnsupportedProjectFeature guards.

Exact GREEN gates: fmt, workspace all-target clippy `-D warnings`, wasm32,
new cases + existing TPMS-D rotation/multiline/anchoring/Honeycomb/Line controls
in both modes, then canonical dual-mode FULL with fresh cache/artifacts,
zero FAIL/skipped, `ARES_PIN_OFFSET=20 ARES_PARITY_THREADS=4` (CPU20-23).
Compilation/probes CPU12. Rebase on current main and regenerate exact-hash
FULL if it advances. Coordinator alone reviews/merges; no production before
independent SPEC and RED approvals.

## External receipts

Under `${EVIDENCE_ROOT}/w3-tpmsd-anchoring-inset-`:
`top6-m2/`, `top7-m2/`, `top5-m2/`, `top6/` and matching `-seeded/` roots hold
inputs, producer identities, commands, repeat streams and complete results.
The target `role-observation.json` files localize activity without changing
the strict verdict. `localize/native-first-gdb.log` and
`localize/ares-params-gdb.log` show the first domain mismatch and live settings.
`spec-review/matrix-audit.json` plus external `...-audit.py` verify matrix,
repeat/cross-mode and ZIP/settings boundaries. The extra official multiline1
phase checks remain diagnostic receipts. No public tests/inputs or production
code are committed in this SPEC.
