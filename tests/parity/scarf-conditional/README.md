# Conditional scarf eligibility public E2E

SPEC `5189e53a586ccffde246c3e798b8563aac0f97e9`, upstream
`8500fcdccaa10b5099ac20d252af3a7c560046f1`:
`GCode.cpp::GCode::extrude_loop5777-5791`, delegated
`ExtrusionEntity.cpp::ExtrusionLoop::is_smooth369-424`; nonzero dependency
`SeamPlacer.cpp1083-1092,1550-1563` raw unsupported-distance handoff.
No expected Ares error, threshold fallback or new pipeline.

Reproduce inputs/whole and ZIP-entry SHAs: `python3 generate.py`. Uses the frozen
seam-rear aligned prism with original ZIP metadata. Control/positive are byte-exact
approved discovery inputs. Sharp changes only positive angle0→155. Taper retains
bottom/z and scales top XY1.15 about base centroid(0,-5/3); low/high differ ONLY
scarf_overhang_threshold1%→100%, conditional1/angle0 otherwise identical.

Four actual Native qualification commands at approved SPEC, official+seeded,
CPU15, fresh cache/TMPDIR, before permanent RED: low0 slope moves/high48 in BOTH
modes. High first slope `G1 X109.72 Y114.313 Z.23 E.0027 ; perimeter`, layer2
Outer wall. Artifact root
`/home/indexyz/.ares-evidence/worker4/scarf-conditional-qualification-5189-v1`.
Original control/positive Native48; positive first slope
`G1 X109.72 Y113.938 Z.23 E.0027 ; perimeter`. Sharp0 is a source-predicate
negative premise, checked by the formal Native run, not relabeled discovery.

Test `conditional_scarf_matches_native_public_slicing` uses Native FIRST per
case, then validated public API(default overrides/seed0) and real bare CLI on
identical frozen bytes. All five cases' complete streams/admission receipts,
Native activity, and BOTH entire comparisons are durable BEFORE final assertions.
Native-ACTIVE/INACTIVE/first-layer/role/setting assertions are mandatory. Controls
require whole PASS; both active positives must ultimately require whole PASS too
(they are genuine target failures at RED, never expected-error PASS).
Missing output/admission/activity premise failure is STOP/NOT EVALUATED, not RED.
Unchanged generator-only whole comparator includes M73/arcs/statistics/config/
footer. Native slope counts are activation evidence ONLY, never a filtered oracle.

Run only with live Native profile-entry gate, fresh canonical official+seeded
roots, CPU15/jobs1/nextestthreads1, load<60, START/END receipts; classify official
before seeded, no retries-to-success. Exact RED must be reviewed before separate
GREEN authorization. Owner's additional GREEN gate: ALL seam + scarf + core-seam
families in BOTH modes; any residual whole mismatch means STOP. Full/workspace/
integration remains coordinator-only; RED integrates only with GREEN.
