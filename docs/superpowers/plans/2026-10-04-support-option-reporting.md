# Support Option reporting plan

Boundary: ARD-0101 and matching spec. Baseline853f8465 is clean and pushed.

1. Run current public support domains; retain complete-output differences.
2. Add a subprocess/report regression at the existing public acceptance seam;
   show RED when compared Tree output is incorrectly omitted from counters.
3. Remove only stale unimplemented-support classification/state/assertions.
4. Verify final report behavior without fixing or hiding actual geometry RED;
   run full gates and independent six-axis review, commit/push finite slice.

Baseline12694 terminal100:75.967s, four domains (enable_support, support_type,
support_style, tree_support_branch_angle), two incomplete. Paired manifests
show all support_style values PASS, while tree(auto) and branch-angle cases
are real DIVERGENT, incorrectly described as port pending. Exact artifacts
/tmp/ares-support-sweep-853f8465-seeded, log with same prefix. Tree(auto) first
diff byte897/line33: M73 R26 versus R25, with894034/896309 normalized bytes;
this is an early visible symptom, not a claim that only time differs.

Full stock seeded scan22812 started on immutable853f8465; root/log prefix
/tmp/ares-stock-matrix-853f8465-seeded. It is live, not current complete parity.
Do not relabel this loaded baseline binary as the subsequent reporting patch.

Subprocess regression49107 terminal100:1 failed,20.453s,
/tmp/ares-support-report-red.log. It sees all four paired support_type
manifests but the report counts only3 comparisons. This establishes the
reporting bug independently of whether Tree geometry is correct. Remove
the stale classifier and counter; do not alter slicing, presets or comparator.
Retain future subprocess paired artifacts in the configured external root.

First post-removal green63236/workspace64681/Clippy64321 terminate101 at
compile (E0061): ten existing OptionOutcome fixtures still pass the removed
counter. No behavioral test ran. Adapt those call sites only, retain their
assertions; fresh gates must replace these failed compile receipts.

Corrected seeded76782 and official CPU0 3664 both terminal0: subprocess/report
regression PASS. Each keeps six actual pair manifests (three normal/manual
PASS and three retained Tree-auto DIVERGENT retries), four unique public enum
values, and a truthful FAIL support_type row with4/4 strict comparisons.
These are reporting GREEN receipts, not Tree slicing parity. The first
difference and nonzero child exit remain required. Artifacts/log roots:
/tmp/ares-support-report-fixed-green and /tmp/ares-support-report-official.
Clippy12064 terminal0,19.86s: core253/269 unchanged, CLI parity30 (previous31),
KSR10 unchanged. Fmt5789 and diff checks terminal0; WASM core/adapter check
terminal0,0.25s, compile-only (neither crate changed).

Full650-domain seeded scan49613 started with the frozen reporting WIP at
/tmp/ares-option-matrix-support-report-wip-seeded, distinct from the old
baseline scan; its binary provenance stays unknown rather than claiming HEAD.
Final workspace37786 terminal0:7611/7611 PASS,36 slow,9 skipped,266.600s,
/tmp/ares-support-report-fixed-workspace.log. Independent static six-axis
review reports no new must-fix; immutable-patch approval is required before
commit/push. Scope is8 modified existing Rust files,1 new public report test
and3 source-boundary docs; all changed Rust files remain below400LOC.

Reporting GREEN times: seeded68.639s, official50.491s. Official Tree-auto
three retry inputs share SHA e45bff975bfcb3eaf157ad5427a8b7d15d30a799d6e5cb22fa7491b2e07c23a1.
Full native/native diffs for case-m0v7Hz versus case-fVnz3B and case-PHXuQp
show only automatic header line2 dates; no geometry differences. Ares output
SHA is identical for all three7f8fc687333ed58896c66a4ccebfc5120a3625716bb7efd7c33d560026b6fc28.
Native/Ares movement already differs at the first travel:95.987/100.314 versus
93.746/99.414, followed by different skirt polygons/extrusion. This is a real
reproducible geometry gap, not an M73-only explanation or reference waiver.
No comparator normalization was changed by this diagnostic.

Subsequent execution-domain correction (ARD0102): the above reproducible
native/native output and differing Ares geometry do not by themselves prove
an algorithm gap. Ares ran on the unrestricted host while Orca ran on CPU0;
Organic merges consume worker count. Same saved pair with Ares also on CPU0
passes complete comparison. Reporting behavior remains verified and unchanged;
the earlier algorithm-gap interpretation is superseded, not its raw evidence.
