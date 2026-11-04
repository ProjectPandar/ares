# SPEC D: local anchor FloatOrPercent projection admission

Status: proposed SPEC, docs only; Native admission unproven.
Base commit `96eb17ccc2805ce1d4f95045e452eb070e19f183`; independent of the
A/C1/B candidate `18d59a638`, which stays frozen.

## Owning source and audited distinction

Source: `OrcaSlicer/src/libslic3r`, commit
`8500fcdccaa10b5099ac20d252af3a7c560046f1`.

| Source | Behavior |
| --- | --- |
| `PrintConfig.cpp:3137-3187` | Anchor and maximum are FloatOrPercent; max_literal=1000 is distinct from a numeric min/max. |
| `PrintConfig.cpp:10519-10562`, `Config.hpp:2478-2479` | Numeric validation of the loaded config uses definition min/max, default +/-FLT_MAX. This does not establish the range of every local model override. |
| `PrintObject.cpp:3602-3709` | Effective-region merge copies local values; post-merge normalization has no anchor range/sign/finiteness rule. |
| `Fill/Fill.cpp:978-989` | Cast each carrier to float before percent projection, multiply by 0.01 and spacing, then cast result to float; clamp anchor with std::min(anchor, maximum). |
| `Fill/Fill.cpp:1477-1482` | Sparse anchor generation passes these projected lengths to the real Fill implementation. |
| `Fill/FillBase.hpp:52` | dont_connect selects the maximum<0.05 path. |
| `Fill/FillBase.cpp:1584-1586,2251-2252` | Debug assertions on projected lengths do not establish shipped release-oracle rejection semantics. |

Ares `options/config_types/scalar.rs:200-305` accepts finite f64
FloatOrPercent carriers. Local metadata uses this carrier via
`region_options/metadata.rs`, and `region_options/normalization.rs` does not
bound anchors. `bridge_over_infill/sparse_anchoring/projected.rs:11-25`
narrows to f32 before percent projection, as upstream does.

A finite f64 input can therefore narrow/project outside finite f32 range.
The two existing `!anchor_max.is_finite()` / `!anchor.is_finite()` admission
checks are not proven unreachable by a finite-carrier argument. Native may
reject earlier or fail while slicing; only its actual public admission and
complete output can establish a usable RED.

## Rust boundary / sequential slices

Destination: existing
`project_slice/prepare_infill/bridge_over_infill/transaction/anchor_projection.rs`,
with `sparse_anchoring/projected.rs` and existing Fill connect consumers only
if their behavior is proven missing and a repair PLAN is separately accepted.
The admission shell must not introduce a new Ares numeric policy around
upstream FloatOrPercent projection.

D-M first: maximum projection check. D-A second: anchor projection check,
with a finite maximum so the earlier maximum check cannot mask it. Separate
public RED and repair commit for each; no combined deletion.

Included: bounded finite public values that Native truly accepts and completes.
Initial repair is deletion of only the selected gate if sufficient. Preserve
exact narrowing, percent projection, min ordering, dont_connect and existing
Fill algorithms. Deferred: C2 patterns, density, new schema acceptance,
layer-range/height-profile work, defaulting/clamping policy, geometry redesign.

## Native preflight inputs (not admitted cases yet)

After this SPEC, use object/model-part
3MF metadata overrides on a positive-density rectilinear solid-ceiling fixture.
Keep the loaded global anchors ordinary and valid. Check the exact inserted
local key and finite string, input byte identity, unchanged ZIP member hashes,
actual Native wrapper/ELF SHA and complete final stream.

Probe candidates independently:

- D-M: `infill_anchor_max=1e39`, ordinary anchor. The raw local carrier is finite
  f64; expected narrowing may exceed finite f32. Also inspect `1e39%` as an
  independent percent carrier, not by overriding projection to keep it finite.
- D-A: `infill_anchor=-1e39`, `infill_anchor_max=1000`. The earlier maximum
  check remains finite; min/projection may retain the negative overflow.
- Controls: ordinary absolute and percent anchor/max combinations; negative
  finite anchors and max=0 preserve the already merged dont_connect semantics.

Do not send literal NaN/Inf strings and call their parse rejection an admitted
RED. Global out-of-range rejection is a diagnostic control, not proof that
local region metadata is rejected or accepted.

Record each real Native exit, timeout, raw output and producer identity before
classifying. Require internal bridge/sparse output witness, complete footer,
finite public input and no setup failure. If Native crashes, times out,
rejects, or invokes release behavior that cannot establish repeatable complete
output, stop and report; do not define a guessed output or turn off assertions.
Natural repetition, if needed for deterministic admission, uses fresh roots
and the same single-core domain.

## Public RED / repair gate

For a Native-admitted completed case, Ares' public call must reject at the
selected exact `infill_anchor_max` or `infill_anchor` unsupported return.
The test expects existing strict whole-output Native/API/CLI parity; baseline
failure is recorded as RED on its own commit with unchanged future assertions.
Keep original raw input/output SHA and producer records outside Git.

If no admitted candidate reaches the gate, submit the actual admission finding
for a decision before implementation; do not claim either guard unreachable
merely because these few probe values fail upstream.

After accepted RED, delete only the selected rejection if that matches upstream
release output. If real Fill behavior differs after deletion, preserve the
failure and request a narrow source-cited PLAN before touching a consumer.
Do not add guards, saturate/clamp the input, fabricate anchors, silently skip
regions or substitute fallback patterns/lengths.

## Validation and scheduling

Official and seeded runs use an independent fresh Native cache
and artifact root every run.
Selected active/control and existing bridge/anchor comparisons must PASS both
modes; strict clippy/fmt/wasm32 green, each file at most 400 LOC. No new ignore,
allow/expect suppression, dbg/println, regions[0], weaker assertion, or committed
Native artifact. Land per slice. No run has started for this SPEC.
