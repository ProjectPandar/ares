# Concentric sparse multiline: preserve nominal spacing and coordinate units

Status: SPEC proposed; no production changes. Base: `48ff6b184b4681e30a855b0ffed42873c36d0f27`.

## Upstream boundary

Upstream checkout: `8500fcdccaa10b5099ac20d252af3a7c560046f1`.
The owner is `FillConcentric::_fill_surface_single(..., Polylines&)`,
`src/libslic3r/Fill/FillConcentric.cpp:12-68`:

- Lines 22-23 derive scaled `min_spacing` and density-dependent integer `distance`.
  The latter is the ring pitch, not the extrusion-line spacing.
- Lines 25-27 rewrite `this->spacing` to `unscale<double>(distance)` only when
  `density > 0.9999f && !dont_adjust`. Sparse fill retains its nominal mm spacing.
- Line 31 contracts by `-float(scale_(0.5 * (multiline - 1) * this->spacing))`.
- Lines 37-38 use the ring pitch to generate nested rings; lines 43-50 order and split them.
- Line 54 passes the mm-valued member `spacing` into
  `FillBase.cpp:2712-2784 multiline_fill(..., float spacing)`.
  That function scales its offset at lines 2739-2749. Supplying an already-scaled
  pitch here both substitutes the wrong quantity and scales it a second time.
- Lines 56-68 retain loop-end clipping and validity filtering.

The Rust destination is `crates/ares-core/src/fill/concentric_sparse.rs`,
`fill_concentric_sparse`. Its temporary compatibility shell currently computes
`distance` in scaled units at lines 60-66, then uses that value for both
contraction (80-83) and `multiline_offset::apply` (136-140), including sparse fill.
`fill/multiline_offset.rs:32-44` narrows the supplied mm spacing to f32 and scales
its derived offset. The public caller is
`project_slice/fill_entities/concentric.rs:95-132`.

## Native-first survey

CPU11, outer taskset, existing main CLI from FULL12, fresh per-case roots/cache,
no builds, no retries. Both official (wrapper `1689a2ff...`) and seeded producer
`1k0p5...`, seed 0, accepted every input (exit 0).

Evidence: `/tmp/worker6-concentric-survey.kln54hvl/report.json` and
`/tmp/worker6-concentric-classic-survey.so_yd47x/report.json`.
The latter changes only `wall_generator` to `classic`; results are identical.
Actual Native affinity samples: 30 and 41 respectively, all CPU11.

| Sparse density / multiline | Native | Ares | Retained whole-output first difference, both modes |
| --- | --- | --- | --- |
| 30% / 2 | accepts | CLI -102 (exit 154) | line 18: `M73 P0 R39` vs `M73 P0 R34724` |
| 45% / 2 | accepts | CLI -102 (exit 154) | line 18: `M73 P0 R44` vs `M73 P0 R23159` |
| 30% / 3 | accepts | CLI -102 (exit 154) | line 18: `M73 P0 R39` vs `M73 P0 R104119` |

The M73 difference is a downstream observation, not a proposed timing fix.
Ares writes retained G-code and reports `run found error, return -102, exit...`.
The source discrepancy explains an offset many orders larger than nominal spacing;
public RED will also capture the earliest sparse-path geometry difference.
The survey is discovery evidence, not the separately committed RED.

## Included rewrite

Keep nominal/effective spacing in mm separately from scaled integer ring pitch.
Only the upstream full-density adjustment may rewrite effective spacing; sparse
contraction and multiline expansion consume nominal spacing. Preserve the exact
line-31 expression order and f32 conversion after scaling. Preserve existing
ring generation, union ordering, nearest split, clipping, and public emission.
No change to shared multiline offset, Clipper, motion, validation or admission.

Do not absorb the independent pitch-arithmetic differences at lines 22-23 in this
slice without a new source-cited request and attributable RED. Count1 Ares raw
must remain unchanged. Full-density callers must retain their current behavior
unless the same unit correction changes an observable stream; report such a
change before expanding acceptance.

## Deferred findings

The count1 / 30% survey already diverges at layer 4, line 1463:
Native `G1 X132.878 Y95.122 E1.51782`, Ares `... E1.51781`.
Both accept it, and Classic gives the same result. This is frozen as an independent
flow/geometry precision finding, not a PASS control or permission to change flow.
The Arachne full-density constructor and its known initialization boundary remain
outside this slice. Support, ironing (worker5), Adaptive/SupportCubic (worker3),
brim, OptionsZ replay and ZAA motion are not touched.

## RED, GREEN and acceptance

After approval, add public CLI/API runtime Native E2E cases for the three sparse
multiline requests above in both modes. Use the authored prism input and existing
OrcaRunner runtime export/fixture adapter; commit no Native products. Assert
successful validated API and CLI admission, complete ordered bytes and line counts
with the existing generated-by-only comparator. Report the independent RED hash
before production. Keep count1 raw unchanged and add passing Rectilinear count1
and TPMS-D 45-degree controls; do not relabel the known Concentric count1 failure.

GREEN changes only the spacing ownership/unit handling described above. Any
residual whole-file difference, including the independent E precision signature,
requires STOP and coordinator review; no tolerance or comparator changes.
Then exact-HEAD directed official+seeded, complete option_coverage in both modes,
non-RED raw invariance, strict clippy, fmt and wasm32. Files stay <=400 LOC.
Full/workspace remain coordinator-only. No push without exact-hash approval.

## Proposed amendment: preserve density and ring-pitch expression types

Status: amendment pending approval. The spacing-only trial `b94b93053` is NOT
GREEN: both modes are 0/3, though all API/CLI calls now accept. Evidence:
`/tmp/worker6-concentric-green.Qo0bBh/residual-audit.json` and
`/tmp/worker6-concentric-STOP.md`. No assertion/comparator changes.

The coordinator authorized bounded observation after STOP. Existing binaries
were observed with gdb only; no source instrumentation, Native rebuild, memory
writes or expression substitutions. The initial absolute-address Native attempt
failed to insert a PIE breakpoint and is not evidence. Successful runs resolve
breakpoints after exec. Every observed Native inferior records affinity CPU11.

C30M2 observation: `/tmp/worker6-concentric-diagnose.rkrl8m01/report.json`,
`native-values.json`, `rust-paths.json`, `pre-multiline-diff.json`.
Native normalized output equals the unchanged trial reference; Ares raw output
is exactly unchanged. At physical layer 4 (Native `layer_id=3`, Z0.8):

| value | Native | Ares trial |
| --- | --- | --- |
| spacing mm | 0.40707963705062866 | same |
| density fraction f32 | 0.30000001192092896 | 0.29999998211860657 |
| scaled min_spacing | integer 814159 | f64 814159.27410125732 |
| integer ring pitch | 2713863 | 2713864 |
| negative contraction f32 | -203539.8125 | same |
| spacing passed into multiline, f32 | 0.40707963705062866 | same |

The first differing input is density. `Fill.cpp:1283` (and the other surface
path at 1479) computes `float(0.01 * surface_fill.params.density)`: the literal
is double, the product is double, and only the assignment narrows to float.
Rust `project_slice/fill_entities/concentric.rs:98` currently multiplies in f32.
The owning Rust boundary is this Concentric caller only; no shared density
helper or other pattern changes are proposed.

There is a second independently wrong expression boundary:
`FillConcentric.cpp:22` stores the double product
`scale_(this->spacing) * params.multiline` in `coord_t`, truncating first.
Line 23 then converts that integer to float for division by float density;
`coord_t(...)` truncates the resulting f32 quotient. The existing Native
instructions confirm `cvttsd2si`, `cvtsi2ss`, `divss`, `cvttss2si` in that order.
Rust currently retains f64 for both the product and quotient. Correcting only
the density input still leaves a modeled f64 pitch of 2713864 for C30M2;
Native's observed typed calculation gives 2713863.
Line 37 also uses integer `min_spacing/2`, including integer sum/negation before
the offset call's float conversion. Retain that integer arithmetic rather than
the current f64 half spacing. These are one ring-pitch ownership/type chain.

Before multiline, the observed emitted layer-4 outer ring is byte-for-byte
equal as a point vector. The first inner ring already differs:
Native (-28102285,-20102285), Ares (-28102284,-20102284).
The next ring differs by 2 units, and so on. Thus the geometry discrepancy
exists before multiline's shared offset/union; its input pitch differs by one
unit and accumulates exactly one unit per ring. Do not change shared Clipper,
contract, multiline offset or flow based on this finding.
The Rust capture at index 0 belongs to a separate projection call, so the
emission comparison uses capture index 1, whose outer ring matches Native.
Projection caller alignment is deferred, not relabeled as matching.

Count1 observation: `/tmp/worker6-concentric-diagnose.rrc7_fjq/report.json`.
It uses the frozen count1 input, not a replacement control. Native normalized
output remains identical to its survey; candidate Ares raw is exactly identical
to main's frozen output. The same density discrepancy is present. Native
min_spacing/pitch are 407079/1356930; Ares are 407079.63705062866/1356932.
This proves count1 shares the wrong input/type chain, not that its E residual
has already been closed. Its E1.51782/E1.51781 finding remains a FAIL.

Requested scope: add the Concentric-specific density conversion and exact
integer/f32 ring-pitch/half-spacing expressions above to the existing slice.
No implementation has been made for this amendment. Keep `b94b93053` unpushed.
The earlier count1 raw-unchanged requirement needs an explicit exception if this
amendment is approved: this shared arithmetic necessarily changes count1 ring
coordinates. Proposed replacement acceptance is an additional public runtime
Native count1 witness with strict whole bytes, not a rebound hash or tolerance.
No existing test assertion would be altered. Confirm this exception before code.

After approval, freeze the additional count1 RED before implementing this numeric
chain; retain all three original RED cases. Any remaining diff requires STOP,
including C45M2's layer-17 non-sparse path mismatch (first Native geometry line
6005 X140.571/Y133.201 versus Ares X141.05/Y133.201), which is not proven to be
fixed by pitch arithmetic. Do not touch anchor_projection or consume a flow /
solid-path fix without another ruling. All accepted cases must match both modes;
then run non-RED raw auditing and the previously specified gates.
