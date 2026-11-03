# Rear seam visibility initialization — source rewrite SPEC

Status: bounded read-only causality confirmed; docs then tests-only public RED.
Base a0c2c5724e798ca21cb2e2d5ef117055f7beef02 remains frozen. W6 owner approved;
W2 seam config work is merged. NO production edit or GREEN authorized here.

## Upstream boundary and mandatory semantics

Upstream pin8500fcdccaa10b5099ac20d252af3a7c560046f1:
- GCode/SeamPlacer.hpp58-68:SeamCandidate initializes visibility0.0f.
- GCode/SeamPlacer.cpp1427-1457:SeamPlacer::init computes global occlusion and
  candidate visibility ONLY for spAligned/spNearest/spAlignedBack. spRear is
  omitted and its candidate visibility remains0.
- cpp777-799:SeamComparator::is_first_better prefers rearY,then the existing
  visibility/angle score resolves tied rearY. No new tie-breaker is allowed.
- cpp1465-1489:pick_seam_point and align_seam_points still run for rear.
- cpp1272:stable seam ordering;1364-1391:rear spline/final position;
  cpp1500-1564:place_seam consumes chosen/final position for actual loop split.

Exact Rust destination boundary is
`crates/ares-core/src/project_slice/seam_placement.rs::apply/candidate_penalty`
and `seam_placement/alignment/preparation.rs::prepare_layer`.
Current apply builds GlobalVisibility for every active placement including Back;
prepare_layer scores Back candidates through the unconditional visibility term.
This compatibility shell incorrectly broadens upstream init's mode scope.
The port slice must restore rear's zero visibility/default and mode-scoped
sampling/use,not design an Ares-owned rear placement pipeline.

Included:rear mode visibility initialization and its contribution to existing
candidate scores. Preserve all candidate geometry/order,angle penalties,
context/overhang/embedding,comparators,stable sorting,alignment/spline math,
loop splitting and final-position consumption. Preserve existing aligned,
nearest,alignedback sampling,mesh frames,negative-volume all-hits BVH behavior,
provenance and front bias. No per-ray linear scans or changed sampling scheme.
Random is adjacent upstream behavior but DEFERRED:do not infer authorization
for a separate random-seam change from the rear witness.

No writer/time/M73,travel_to/origin/lift,processor/fan/tokenizer,brim/Type1,
seam-gap state,formatter/evaluator,geometry or model/input repair. No fallback,
feature flag,independently designed tie preference or hardcoded Native index.

## Frozen real Native witness

New Prusa MK4 stock flattened machine/process/filament loaded with raw upstream
`tests/data/overhang.obj`,not printer IDs swapped into BBL3mf. Official export0
and Native baseline slice0;effective MK4/marlin2/machine/process/filament IDs
verified. No compatibility -17 or bypass occurred.

Control baseline SHA256
`dc37585d4483eae1f5eec7331d395c99132b3bd5959ec96d486e1c901c1f0d89`.
Target SHA256
`747a5ed4ac32c8d9d070ffd126f8da559a5d0c72554db84de203300a6dba2a47`.
Effective config differs in SOLE key seam_position aligned→back (spRear).
Freeze both original Native-exported archives unchanged;no dynamic regeneration.

Discovery official Native2 stable FIRST;baseline full ordered API/realCLI PASS.
Target API success/realCLI0/rawAPI-CLI equal,but full ordered comparison FAIL
at byte1996,line98:Native purge G0X45 before M73,Core has M73 before purge.
Do not repair this early propagation or ignore M73.

Diagnostic only first non-M73 physical delta appears at printZ3.4,sliceZ3.3:
Native starts inner wall at left rear(104.433,129.004),Core at right rear
(145.567,129.004). Actual first extrusion confirms different loop seam/start,
not just travel route. Complete strict outputs/differences retained unmasked.

## Read-only causal evidence, separated from host counterfactual

Evidence root
`${EVIDENCE_ROOT}/w6-nonbbl-stock-baseline-ol75m7h7/trace/`.
Final native-v2 and rust-v4 observers CPU15 serial,may-call-functions off,
no inferior calls or writes,normal exit0,invalid probes0.
Native trace uses seeded DWARF binary,NOT official binary;its full output equals
original official Native after generator-line-only normalization. Rust output
is raw byte-identical to original exacttipCLI. No perturbed execution accepted.

LIVE target layer16/sliceZ3.3 five candidates have EXACT ordered coordinates,
angles,overhang0,embedding0. Neutral/no central enforcers on Native.
Native visibility all0. Rust LIVE visibility at the same points:
[0.9899845123291016,1,1,0.9886844754219055,0.9899845123291016].
Rust LIVE scores start1.305871605873108 vs1.3158870935440063 for the two tied
rearY corners. Both initial index0;after alignment Native index1,left position
(-21.36250114440918,24.79951286315918,3.299999952316284),Core index0,right.
This is not claimed as a direct Native score or upstream expf bit capture.

HOST-ONLY replay from captured31layer plans/KD nodes reproduces ALL live Rust
chosen indices/finalized flags with original scores. Changing ONLY visibility
contribution to0,recomputing initial best candidates through the same comparison
and running unchanged cross-layer index-selection algorithm gives target final
index1,matching LIVE Native. No geometry/order/context/flow/KD/alignment changes.
It does NOT execute a production candidate or validate spline/finalposition math
or claim a full-output GREEN. HOST numerical replay is not mislabeled LIVE.

verify-trace.py0 and HOST-replay.py preserve machine-readable receipts/hashes.
Earlier optimized-out Native final probe and zero-hit Rust pre-align source
locations are retained/excluded as incomplete attempts. Final valid probes use
Native saved live candidate storage plus before/after alignment snapshots and
Rust line132 pre-align/live visibility returns/line171 post-align snapshots.
An initial Native/Native comparison mistakenly used Native/Ares helper mode;
that receipt is excluded,replaced by actual Native-stability helper operation0.

## Public RED and approval gates

After this docs-only commit,add two source-cited public cases in the existing
W6 group using UNCHANGED native_stream_case::check→NativeCase,fixtureSHA,
(false,|_,_|{}). Baseline is positive control;target must be actual strict FAIL,
not a caught expected-error success. Native2 FIRST,then real validated byteAPI
and CLI;full ordered M73/time/motion/comments/metadata/footer retained.
No masks/tolerances/frozen reference text/expected index assertion in public tests.
Use CPU15 taskset,jobs1,threads1,official then seeded,independent empty caches.
Because canonical target assertion panics before final rawassert/helper update,
independently verify actual saved API/CLI raw equality;never claim that later
assertion ran on RED. Both strict raw API/CLI files remain evidence.

Report docsSPEC+tests-onlyRED with exact tips,Native2/public receipts/selection/
ELF provenance/clean scope;HOLD. No production edit,GREEN,FULL,merge/push or
integration implied. A later implementation requires explicit separate approval.
If a different unexpected RED residual appears,STOP and report without repair.
All changed text<400LOC;coauthor trailers required. No tests after production.
