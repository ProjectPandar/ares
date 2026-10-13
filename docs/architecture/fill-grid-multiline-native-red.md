# FillGrid mode0: actual public Native-backed RED

Approved [SPEC](fill-grid-multiline-trapezoidal-spec.md) `44c55597e`;
rebased SPEC `73c8977d6`. Base `c822927a1`. Test code `c62faddd6`
(original preparation `c3e779fa3`), no production changes. The source test
preparation becomes a genuine RED only through the results below.

Fresh CPU11, outer taskset, jobs2/threads2, fixed CPU11 test/Native runner:
`/tmp/worker6-grid-mode0-red.761kO3/{official,seeded}/{log,exit}`.
Both complete before the00:15 window close: exit100, 2 PASS/7 FAIL.
This is not a timeout, dependency skip or compile-failure RED.

G1 and H1 count1 whole-output controls PASS. Each of the nine tests also
first verifies its own count1 input against runtime Native: all nine inactive
Native/validated API/CLI complete streams PASS in each mode, including angle0,
density20 and infill-combination controls. All seven active Native slices
succeed; the validated API and CLI also admit them, preserve Z/HEIGHT rows,
but fail complete ordered byte/line-count equality. No comparator is changed.

Input requests are checked against the runtime exported project's effective
values. Authored prism/hole STL inputs remain unchanged. The exported inputs,
Native output, stdout/stderr/cache receipts and comparison JSON stay outside
git. Cross-mode inputs have identical ZIP member names/bytes; each Ares API
stream is raw-byte identical to its CLI and opposite-mode streams. Native
streams also agree byte-for-byte after the same generated-by-only banner
normalization on both Native sides, separately from the unchanged
Ares-versus-Native comparator.

| Active control | First whole difference line | Native / Ares lines | First differing sparse layer Z | First sparse Native XY / Ares XY |
| --- | --- | --- | --- | --- |
| G2 count2 | 18 | 21495 / 13148 |0.8|141.02,133.02 /133.324,133.02|
| G3 count3 | 18 |17315 /8929|0.8|132.417,133.02 /141.02,128.93|
| G10 count10 |18|12845 /7655|0.8|141.02,115.345 /135.61,133.02|
| G2-A0 angle0 |32|22499 /12601|0.8|126.487,133.02 /136.935,131.507|
| G2-D20 density20 |69|13019 /8404|0.8|141.02,113.81 /133.028,132.74|
| H2 hole count2 |38|22005 /13951|0.8|141.02,133.02 /133.324,133.02|
| G2-C2 combined |32|13439 /9016|1.0|141.02,110.611 /132.732,133.02|

Whole first differences include M73 placement/time. The later actual sparse
commands establish a generator difference; they are diagnostics, not a
filtered replacement for the complete comparison. The combined control
uses upstream coBool infill_combination=1 and max height0.4 over0.2mm layers
(`PrintConfig.cpp:3974/4093`); it actually emits0.4mm-high sparse paths before
its equality assertion. This is the approved two-layer combination witness,
not an invalid boolean value2 or an inactive config-comment check.

Audit with exact paths, digests, per-case comparisons and first sparse rows:
`/tmp/worker6-grid-mode0-red.761kO3/red-audit.json`.
Wrapper SHA256:
`1689a2ff46564cf723ad90c663051b58115920c3cce639bd2961340955cebbf3`.
Affinity records: root `outer-affinity`, each mode `parent-affinity`; the target
runner explicitly tasksets11 and exports ORCA_REPLAY_CPU11.

Earlier preparation failures remain excluded: `worker6-grid-mode0-red.MS4juf`
was interrupted during compilation because the canonical launcher would
replace the directed CPU11 runner; `worker6-grid-mode0-red.eDZA2w` exited101
before Native on an inaccessible profiles_root helper. The test preparation
fixed that lookup to the same repo_root/resources path used by the existing
harness. Neither earlier batch establishes RED or GREEN.

RED is now frozen. Production starts only after rebasing onto main containing
W2 C2-T. Only the Grid anchoring arm is in scope; bridge-anchor projection/
admission is not. Any common Stars-tail extraction needs a separate mechanical
commit and raw Stars invariance proof. Count1/non-RED bytes must remain unchanged.
No production or GREEN claim accompanies this evidence-only update.
