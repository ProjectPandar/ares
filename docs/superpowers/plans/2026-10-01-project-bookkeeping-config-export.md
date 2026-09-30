# Project bookkeeping config export plan

Base9ae520aa, ARD-0074; prerequisite to ARD-0073 oracle migration.
Public seam is the approved native project bytes to complete G-code.

1. Reproduce KSR missing config-comment RED and inspect source owner.
2. Remove only this key from exported one/two-filament projects; full RED
   and native width proof. Supplied nonempty metadata is a preservation control.
3. Implement absent-key creation, require full GREEN and counterfactual RED.
4. Preserve official fixture, capture independent complete initialized KSR
   diagnostic oracle. Keep semantic rules unchanged and ordered-byte RED visible.
5. Full gates and independent six-axis review, then code/docs commit/push.

Known full RED: `/tmp/ares-ard73-ksr.ImyJAd/case-ymi1Ne` line88.
Fresh native `/tmp/ares-ard73-ksr-oracle.mh78O9/plate_1.gcode` uses original
input SHA698f40f13c9075b818abedd3d10f022fbb5d8200aed48fbdde651f6bfb21b8a9
and matches preserved initialized oracle except timestamp. No fix yet.

Historical probes:7974 RED /1.369s missing bookkeeping line3697 at
`/tmp/ares-ard74-red.gwl2Hi/case-7C42JN`. Absence repair makes one-filament
complete GREEN; two-filament first differs at missing T0. Source-owned
initial selection fixes T0, then reveals an independent prime-tower config
echo discrepancy (native1/Ares0), retained under
`/tmp/ares-ard74-diameter-green.nKCc1G/case-1tNw88`. Set prime_tower0 in the
minimized bookkeeping witness; retain that full original failure for later
Print map-producer comparison. Speculative broadcast-predicate edit was
restored; no production workaround for that residual is retained.

Typed diameter length is required: old export split a comma-separated float
vector by semicolon, falsely counting one. Color vector cardinality is also
not a substitute (the two-filament Ender export has one color). Final writer
uses typed diameter count; supplied test vectors are already sized correctly.
Both absent bookkeeping vectors are created from this count. AMS automatic
preparation now follows upstream's mode gate without the empirical empty-only
gate; original KSR complete config block has only these three differences.

Independent review found initial G92 must reset writer state, not just emit
text. Both shared/private E slots now follow Extruder::reset_E. Add a public
absolute-E filament-start e_position[0]=12 writeback case. Native rejects the
Ender preset's before_layer_change G92 under absolute E (30393 exit100); this
is not a parity RED. Clear that incompatible template for this legal witness.
Its final complete validation is pending; do not claim state repair complete.

New complete native fixture is byte-identical to the independently captured
output (SHA7e52b76a...98b7); old official fixture remains unchanged. Manifest
records exact input, wrapper/ELF, seed/CPU and command. Semantic comparator
rules are unchanged. Migration now exposes a real bridge gap at layer26:
9 expected/9 actual segments displaced .161mm on Y (49163 exit100 /20.324s),
`/tmp/ares-ard74-semantic-complete.log`. This remains a required failure,
not excused by config-block or smaller complete cases. No commit approval.
Config test extraction mistakenly included the entire stream tail; correct
the END-boundary slice using the existing reference-block contract. Final
config test and lint gate2557 is live.

Legal absolute-E run95911 terminal0 /4.554s: all four complete native cases
PASS, including nonzero filament-start writeback followed by T0/G92 and
the entire emitted stream. Artifact root
`/tmp/ares-ard74-absolute-legal.mHfozw`.
Config-block final2557 subcommand PASS /0.138s, full block49065 bytes from
the independent native fixture (not the entire remaining G-code tail).
Clippy2557 terminal0 /47.56s; fmt passes. Full current workspace and Tier-1 gates must be
rerun after the remaining bridge difference is repaired. Independent static
review accepts shared/private E reset; source-owned counterfactual still
required for the new state-sensitive witness. Do not approve commit yet.

First state-only reversion26472 passed /4.533s: the absolute extrusion output
alone did not observe the private counter, so it was an insensitive control.
Add a legal before-layer `;LAYER_E {e_position[0]}` observation without G92.
Still revert only the two reset assignments:79989 full RED /4.581s at line53,
native `;LAYER_E 0` versus Ares `;LAYER_E 12`, complete pair
`/tmp/ares-ard74-state-observation-red.31PBbR/case-9pQrFR`.
Shared/private reset is restored; strengthened complete GREEN is pending.

Current source snapshot immutable binary
`/tmp/ares-ard74-final-candidate-orca-parity`,
SHA256c7b57a924f5e58aabd5bc14cec6d3f5a322e4c27c5f4b4f7ce18a7b175ee7738,
full original KSR79051 RED101 /18.69s at M73 position line2408.
Complete current pair `/tmp/ares-ard74-ksr-current.t4DPO3/case-Y5tPw8`;
its entire CONFIG_BLOCK is exactly native (raw diff0). Model outputs remain
6321717/6321910 normalized bytes, so neither config equality nor semantic
diagnostics establish full acceptance. Accidental old-executable run99679
uses earlier ARD73 SHA1eede3a7...31c and is not evidence for current changes.

Strengthened state test98201 complete GREEN /4.577s after restoring only
the shared/private reset assignments; all four cases pass. Full streams
remain at the suffixed `/tmp/ares-ard74-state-observation-green` artifact root.
Independent read-only review confirms this is a sensitive public witness.
Review also flags that removing the AMS empty-only early return widened the
existing flush_multiplier/one-color flush-matrix compatibility shell. Before
final approval, isolate unconditional AMS override from those unchanged
legacy shell conditions unless their own source-owned witnesses justify
changing them. Color-count1 is not proof of filament-count1.
Workspace no-fail-fast71983 terminated100: 7457 passed, 2 failed, 9 skipped
in257.311s. The second failure is the synthetic banned-key writer case's
old expectation omitting the now-native absent bookkeeping defaults; keep its
complete assertion and update those two lines from the independent one-filament
native witness. KSR layer26 Bridge remains a geometry failure. Tier-1 checks32874
terminated0, including the browser WASM adapter. This is not workspace approval.
AMS review repair now preserves the old empty-input gate for flush preparation
while overriding AMS for both empty and supplied mappings in automatic mode.
No flush calculation behavior is newly ported or accepted by this change.
The scope-repaired four-case public native comparison40282 terminated0:
1 test passed in4.670s; all four complete ordered streams match. Artifacts:
`/tmp/ares-ard74-ams-scope-recheck`. Clippy61822 terminated0. Independent
read-only review accepts the scope repair and complete synthetic expectation;
commit approval remains withheld for the KSR bridge geometry failure.
After the scope repair, config-export29461 terminated0: all23 tests passed
in0.148s, including the full initialized KSR configuration block. The focused
banned-key expectation28646 also terminated0. `cargo fmt --check` and cached
patch whitespace checks pass. Full workspace has not been rerun after these
repairs; its recorded KSR bridge failure remains unresolved.
Candidate staged patch SHA before this evidence
update: c4856fded75be05c2c87545fe3638d23bf1d0196ba912cd628078b4c605ee9a6.
