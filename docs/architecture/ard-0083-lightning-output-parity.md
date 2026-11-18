# ARD0083 — Lightning complete-output parity

Status: accepted. Bounded argument-order repair verified.

## Non-negotiable decisions

Use the upstream `FillLightning`/`Fill/Lightning` source boundaries and
`PrintObject.cpp` anchoring callers. The existing Rust modules are the
rewrite destinations; do not design an independent infill pipeline.
Before choosing a repair, establish which owner first differs.

Acceptance is full ordered bytes, excluding only the generator line.
Randomness is not a blanket excuse for deterministic Ares/oracle divergence.
The pre-agreed public project-input/byte-output seam remains the test seam.
Keep original and reproducible-reference results distinct. Diagnostics
must not remain in core, which cannot own terminal or filesystem behavior.

## Evidence

Revision 8a52b2d2 targeted `sparse_infill_pattern=lightning` sweep exits 100.
All three saved cases carry identical project SHA256
`c786373034253980140ef933e004620df99149341ea2f4cc06578ab98512490c`.
Full input/native/Ares/error/manifest artifacts are retained for all three
cases.
Saved comparator reference provenance is explicitly unknown, not invented.

Two direct initialized native outputs are retained separately.
They replay identically and match the saved
reference. Invocation uses the existing seeded replay wrapper, seed 0,
single-core affinity and the initialized
`orca-slicer-seed-replay-2.4.2` binary.

The first complete-stream mismatch is an M73 placement difference.
Motion-only diagnostics (not acceptance) find actual movement divergence
at movement index1166: native `G1 X112.982 Y109.495 E.12356`,
Ares `G1 X112.83 Y109.63 E.11681`. Native2381 motion lines vs Ares2386.

## Owning source and causal result

`Fill/Lightning/TreeNode.cpp:154-180::lineSegmentPolygonsIntersection`
calls `Geometry.hpp:92-120::segment_segment_intersection` with the outline
edge first and the tree branch second. Rust's destination is
`fill/lightning/tree_geom.rs::line_segment_polygons_intersection`.
Only that call's argument order changes; existing Rust arithmetic, query
traversal, RNG and tree algorithms remain unchanged. Do not assume this
upstream helper is symmetric or replace its arithmetic independently.

This is an argument-order repair, not a complete port of the call contract.
`EdgeGrid.hpp:352::segment` returns endpoints: native passes those endpoints
as the second/fourth arguments to a helper whose parameters are directions.
The existing Rust shell instead passes `s2-s1` and `b-a`. In addition,
`TreeNode.cpp:165` measures the previous `intersection_pt`, whereas Rust
measures the newly calculated `ip`. Both pre-existing differences remain
explicitly deferred. Matching the captured trees and complete output on
this fixture does not prove those differences harmless on other inputs;
no general intersection-equivalence claim is made. They are not changed
mechanically to reproduce a suspected upstream defect without a separate
public failing input and causal investigation.

Independent native return-value capture at this exact function records
145 calls and zero successful intersections; native exits 0
and its generated output matches the fresh native stream except generator.
Thus this fixture does not exercise a successful returned intersection,
and cannot validate endpoint/direction equivalence or nearest-hit distance
selection. The initial debugger launcher used bash on an ELF wrapper and
failed before execution; that zero-call log is not evidence. The corrected
direct ELF invocation is the145-call evidence above.

Native debugger capture
retains 250 boundary records;
its generated output equals the original two native runs. The Ares capture
first differs after tree
propagation42→41, introducing detached branches absent in native output.
The native LD_PRELOAD rand/node captures preserve
returned random values and native output. Actual128 random values agree,
and unordered node geometry agrees, excluding RNG sequence as the cause.

Changing only argument order yields full ordered output equality,
128 matching ordered
random/node records and 150 matching tree-stage records in the companion
log. Temporary core diagnostics are removed before final verification.
The public test at `option_coverage/lightning_output.rs` witnessed RED in
witnessed RED, with paired complete artifacts.
The sweep's historical lightning nondeterminism exemption is removed.
The regression additionally requires positive sparse-infill extrusion in
the reference's relative-E stream, on a G1 with X/Y motion words (not a
pure E unretract), parsing only before the comment delimiter;
complete-output equality then proves
that the Ares stream exercises the same behavior. Artifacts are written
before either output-behavior assertion.

Two falsified candidates remain diagnostic-only outside the repository:
the different stable-sort traversal and reversing same-grid candidate
order both leave the original G-code failure unchanged. The sort replay
shows a separate latent ordering mismatch but is not part of this repair.
Native sampled289 points vs Rust285; `ClipperUtils.hpp:19` defaults to
Miter, whereas the current distance-field shell chooses Square. This
adjacent rewrite discrepancy is also deferred, not concealed as parity.
Neither difference prevents full equality on the present fixture after
the actual call-order repair; broader inputs still need verification.

## Verification and remaining scope

After the motion-witness refinement, public regression and full
26-value `sparse_infill_pattern` enum sweep pass 2/2, with 26/26
strict comparisons and zero rejections.
Select the public regression directly. For the complete enum, set
`ARES_PARITY_OPTIONS=1 ARES_OPTION_KEYS=sparse_infill_pattern` without a value filter. Both use explicit artifact
root, initialized replay wrapper and seed 0 with single-core affinity as above.

Shared initialized consumers pass 10/10:
nine live Ares full-output cases
and native KSR self-replay. Workspace passes 7460 tests, 9 skips.
Final all-target Clippy exits 0 with existing warnings;
fmt and diff checks pass.

True Windows/macOS/Linux core builds exit 0;
release WASM exits 0
and wasm-bindgen exits 0. Actual Chromium KSR tests pass 2/2.
An additional real Chromium run slices the original lightning input and
strictly matches the fresh native stream except generator, retaining full
input and paired outputs (92650 browser bytes).
The first Chromium launch failed on missing libglib before slicing; the
successful runs supply the actual shared-library closure for this process.
No debugger or browser diagnostic code is added to the production patch.

Source-contract limits and an active motion witness are both applied.
The 8a52b2d2
option scans are not completion proof: the printer scan ended with 971/1001 passing
its modified classic baseline and 30 incomplete/failed.
