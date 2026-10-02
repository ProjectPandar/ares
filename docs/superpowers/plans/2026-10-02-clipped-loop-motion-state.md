# Clipped-empty loop implementation plan

Source: GCode.cpp5796-5805::extrude_loop, Rust motion/loop_paths.rs::emit.
Architecture: ARD0092. Spec: 2026-10-02-clipped-loop-motion-state.md.

1. On clean submitted5c7ebd2, add a public3MF/native full-output tracer with
   seam_gap50mm and wipe-before-external enabled; verify native clipped-wall
   absence and actual infill. Capture RED with unchanged full comparator.
2. Port native empty-loop return before wipe/state/inward actions. Preserve
   scarf/vase and nonempty behavior. Do not import dependent support/region WIP.
3. Re-run identical complete output, existing emission/native controls,
   nextest workspace no-fail-fast, fmt/Clippy and Tier-1 checks. Compile-only
   target checks do not prove browser/Windows/macOS runtime equivalence.
4. Independent six-axis review of actual code/docs/results, repair/re-review;
   only then commit code and docs together and push. All broad gates stay open.

Agreed test seam: real project bytes/options through core to complete ordered
G-code bytes, compared against real Orca on the identical exported input.
The modified and baseline full streams are retained by the existing harness.

Initial RED67257 /tmp/ares-empty-loop-red.log exits100/3.000s1FAIL283skip.
Artifact /tmp/ares-empty-loop-red.cQ0wEC/case-tf29Tk has native71882 versus
Rust73730 normalized bytes; first difference468/line18 M73 R5/R6, with the
entire stream retained. Original native no-wall and infill witnesses pass.
No comparator change or command filtering. Added positive infill extrusion
witness without changing that public input or the complete-output assertion.
Ported only empty path-count return and wipe-before-external placement;
scarf and nonempty calls retain their original state sequence. No held WIP.
Target /tmp/ares-lightning-owner-clean-target is dedicated to this clean tree,
incremental disabled; main Tree gates retain their independent target.
First independent six-axis review confirms the seven-line production diff
is independent of held support/regional changes and preserves nonempty/scarf/
vase behavior. Mandatory tracer repair: positive E alone could be unretraction.
Require XY movement as well, verify the existing exported relative-E setting
and actual native M83. The RED native stream already has both mode witnesses,
so this assertion repair does not change the public input or oracle comparison.
Initial GREEN27542, Clippy7693 and workspace2026 are already live against
their earlier compiled test snapshot; their results do not replace the repaired
final tracer run. Production source remains unchanged. No submission approval.
First post-loop-fix27542 now exits100/1.382s1FAIL283skip: extra inward moves
are gone, but normalized actual71786/native71882 differ at1428/line67,
native F1550/RustF1539. Full raw diff retains two missing zero-length wipe
sequences after skirt and consequent cooling/M73 differences. Source
GCode.cpp7659 uses stored-path eligibility, not nonempty motion segments;
calculateWipeRetractionLengths350-353 keeps pre-wipe fraction for zero length,
wipe435-497 still emits tags/speed/reset, followed by final top-up.
Continue this same public RED input by using existing wipe_eligible for the
before-wipe and sequence gates. No new movement, config or support changes.
All earlier in-flight runs remain historical for their compiled source; do not
restart them due to timeout or count them as final gates for this next repair.
Source loop SHA c6589505ae324ab85042d4110e4f00bf70497120d8dcf7ac25cf386699a89b9c,
wipe SHA02408660906d2921440045fd1c7517dc97b70db58de2e0c38df582df9127e874,
tracer SHAc5ec98ac5fcbecd86845d06f5b424a906aeaddb5a6f67d390f63129fa84f7721.
Different rounds' ZIP inputs have different SHA (RED27aeb526..., post-loop
40ca1dda...), so they are not exact-input A/B. Exported settings/model are
unchanged; raw native full streams differ only in the generated-by line.
Every comparator pair still slices the same exported project bytes.
Independent static re-review confirms source has_path versus movement-segment
distinction and no manufactured movement/division for empty segments. Hilbert
tail suppression remains intact; firmware/absolute-E combination runtime
coverage remains unproven, not falsely accepted by this single tracer.
Pre-wipe Clippy7693 exits0/2m47s268warnings252duplicates, historical only.
New wipe-GREEN80361 is live /tmp/ares-empty-loop-wipe-green.log, artifacts
/tmp/ares-empty-loop-wipe-green.2mKDMd. New final Clippy/full commands launched
at /tmp/ares-empty-loop-final-{clippy,workspace}.log after the last source
repair. Preserve earlier live handles until terminal; no restart on timeout.
No code submission until the complete expanded independent candidate passes
actual final gates and independent re-review. Held support/region WIP untouched.
Native48951 now exits0/6.978s3run3PASS281skip, after successful compilation
of the repaired candidate: clipped-loop6.884s and both prior nonempty ironing/
timelapse controls6.906/6.974s pass entire streams. Configured official8348
also exits0/1.951s1PASS283skip, original clipped-loop1.947s, artifacts
/tmp/ares-empty-loop-official.NQNZeg/case-51Uunl. No generated-line-only
comparator change, no M73/wipe/XY filtering. Source is frozen at the hashes
above. Producer metadata limitations remain explicit; do not promote unknown
manifest identity to a complete reference-provenance certification.
Final full49386 and Clippy23981 remain live/queued; retained earlier full2026
still runs its compiled earlier production snapshot. Platform19346/3050/65905
and dedicated new tracer80361 are also live; collect their actual terminal
results. Final fmt70660 and diff exit0; no submission approval yet.
Final Clippy23981 exits0/4m10s268warnings252duplicates, matching the prior
candidate warning inventory. Dedicated repaired tracer80361 exits0/1.510s,
1PASS283skip. WASM19346, macOS3050 and Windows65905 checks all exit0
(6m58s/7m37s/7m56s including queue); these are compile-only checks.
Independent review inspected all four native/configured-official comparison
pairs: complete ordered streams equal after only generated-by removal. The
finite clipped-loop and zero-segment wipe causes are closed; no wider support,
Tree, provenance or untested firmware/absolute-E acceptance is inferred.
Historical pre-wipe full2026 exits0/266.705s7474PASS9skip37slow; it does not
replace final49386, which remains running against the frozen final sources.
Final49386 now exits0/254.661s:7474PASS9skip35slow. Original spiral-vase
test10.693s and original four-assertion spacing test31.550s PASS. No final
workspace failure. All candidate execution/quality/compile gates are terminal;
request final independent six-axis closure before submission.
Final independent six-axis review found no remaining mandatory repair and
approved only this seven-file code-plus-docs candidate for commit/push.
Source ownership, state lifecycle, dirty-work independence, quality, complete
output and final regression evidence are closed for this finite slice.
Normal/Tree, unknown producer identity, untested firmware/absolute-E and
Tier-1 runtime/full printer-option matrix remain open. Final fmt/diff exit0.
