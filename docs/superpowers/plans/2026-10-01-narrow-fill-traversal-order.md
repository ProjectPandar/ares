# Narrow-fill traversal order plan

Base: 9ae520aa. Decisions: ARD-0073. Public seam: native-exported project
bytes to complete ordered G-code, unchanged from the user's approved seam.

1. Reproduce the disabled-support full failure on the committed snapshot.
2. Minimize one input variable at a time; retain all failures and avoid
   substituting a different symptom for the target path-order difference.
3. Test ranked producer/consumer hypotheses against their upstream owners.
   Change production only after proving the cause and ownership boundary.
4. Require full GREEN and a source-owned reversion RED; replay the original
   larger failure without new normalization. Run full gates and independent
   six-axis review/repair/re-review, then commit/push code/docs together.

Status: sensitive full RED/GREEN observed; full gates/review pending.

## Historical diagnosis (before the final parameter repair)

The following no-production-change and pending-GREEN statements describe
their individual earlier probes, not the final candidate status below.

- Immutable 9ae520aa binary is copied to
  `/tmp/ares-parity-9ae520aa.f21xhJ/orca-parity`, SHA-256
  `be2e2b5c4bca01d9d78d0aca2345b0502211de6320239fa971ed4455ed2615d6`.
  The option sweep is live; printer sweep completed 976/1001 PASS,
  15 ORCA_ERROR and 10 VENDOR_INCOMPLETE (modified smoke baseline only).
  Immutable report copied to the printer artifact root before later runs.
- Value filter `false` selects no string `0` case, so run 62700's empty
  domain rejection is not output evidence. Corrected value `0`, 29768
  exit 101 / 11.88s, reproduces three complete DIVERGENT native comparisons:
  `/tmp/ares-enable-support-zero-9ae520aa.4ovbHi`; first difference M73 line120.
- Changing only disable_m73 to 1, 20961 exit100 /4.171s, independently
  verifies no M73 and retains full RED at line497: native travel to
  X113.244 Y113.297 versus Ares X113.388 Y113.135. Complete pair
  `/tmp/ares-disabled-support-no-m73.L34hsQ/case-xVIXsU`.
- Additionally disabling narrow-solid detection, 69253 exit100 /3.974s,
  removes this order difference but exposes separate speed/bridge differences
  at line808; `/tmp/ares-disabled-support-no-narrow.DIRltm/case-PGKM81`.
  It is not full GREEN and is not the candidate regression input.
- Native CLI cut/export minimization rejects --cut (254); no cut fixture
  was produced. Do not treat this as a slicing comparison.
- Restore narrow detection and try the existing short-branch model, 14042
  exit100 /4.419s: first difference perimeter E .00987 versus .00986,
  `/tmp/ares-infill-order-short-branch.Gps9kY/case-l0chdF`. This catches a
  different symptom and is not accepted as a narrow-order regression.
  The diagnostic test has been restored to the original overhang model.

Ranked falsifiable probes: verify no_sort ownership, verify current cursor
at hierarchical sorting, verify nested collection preservation. Inspect
producer point-chain ties if all three consumer contracts match upstream.
No instrumentation or production mutation has yet been applied.

Independent pre-review accepts the diagnostic source boundary, but requires
producer-input comparison before blaming ties: compare every pre-sort
polyline's ordinal, first point, closure and width and the chain indices.
The initialized reference's min_length_factor .5 versus the Rust internal
consumer's 0 remains a candidate upstream-constructor difference to exclude.
The public fixture now explicitly activates and verifies narrow detection 1.
No final commit approval has been given.

Existing ARD-0069 factor-.5 evidence is relevant: full overhang/no-M73 pair
`/tmp/ares-concentric-overhang-no-m73.immqNB/case-dOdseV` first differs at
later speed line859, not the earlier exchanged path groups. This does not
prove identical current inputs; do not skip the current controlled probe.
Next prediction: if extra short producer branches explain the chain input,
changing only internal min_length_factor 0 to the initialized convention .5
will remove the early order difference, while leaving unrelated later RED.
This is a producer-field experiment, not a production fix/acceptance claim.

Current controlled field probe 33880 exit100 /4.176s agrees with that
prediction: first difference advances from exchanged paths line497 to the
later speed line859, F919 versus F862. Full pair
`/tmp/ares-disabled-support-factor-probe.cmwhxO/case-warYnm` remains RED.
The factor split is restored; no sorting or production fix is retained.
Next inspect pre/post short-line filter inputs using the existing external
debugger harnesses; do not add terminal or filesystem instrumentation to core.

External debugger evidence closes the producer-input distinction:
`/tmp/ares-ard73-native-producer.log` and
`/tmp/ares-ard73-rust-producer.log` ordinal5 have identical complete input
polygons and raw paths `[tiny odd, closed B, closed A]`, including points,
widths and closure flags. Native factor .5 deletes the short odd path by
swap-last removal, yielding `[A,B]`. Rust after-filter capture
`/tmp/ares-ard73-rust-after-final.log`, caller breakpoint before simplification,
confirms factor0 retains `[tiny odd,B,A]`. Run76818 debugger terminal0,
inferior replay101 /0.64s, full RED unchanged. Earlier finish-breakpoint probe
32211 crashed GDB134; caller probe54327 stopped on a nested-Vec type parsing
error, not parity evidence. Final external harness corrects the type parser.
No core instrumentation or production change is retained.

Independent read-only review supports the ARD0069 parameter-consumer pivot,
not sorting changes. Complete sensitive GREEN is still required.
Minimization19331: existing short-branch model, wall_loops0, full PASS /3.320s
with old factor0. This is an insensitive control, not a regression witness.
Next change only top_shell_layers0 to expose internal rather than top fill.

## Sensitive regression and final candidate

Probe90769 top_shell_layers0: full RED /3.078s, line388 travels to different
paths and Ares retains a tiny extra extrusion/retraction. Pair
`/tmp/ares-ard73-short-top0.2vLjXy/case-sAigEA`.
Change only min_length_factor to .5: 53412 full GREEN /3.081s.
Final public test verifies exported support/M73/narrow/wall/top settings and
native Internal solid infill activation. Runs3148 and16902 both terminal0;
their shared log is not used for distinct timing claims. Separate complete
pairs `/tmp/ares-ard73-short-final.PI9fGu/case-pwGJup` and
`/tmp/ares-ard73-short-final.7jRc9z/case-3C9f7F` pass.
Counterfactual uses immutable original9ae executable against the exact final
project/reference bytes (production difference is solely the factor):
exit101 /0.61s, `/tmp/ares-ard73-short-counterfactual.NBIYXV/output/case-LZ4Dno`.
Remove private source_cases assertion requiring the UB-dependent tiny branch;
replace it with the sensitive native complete-output public test, not newly
pinned private point coordinates. Sorting and filter algorithm are unchanged.

Immutable9ae option sweep92710 completed exit101 /3020.69s:
386/650 domains complete,1661 generated,1517 strictly compared,1 rejected.
The remaining264 domains are not accepted. Full report and streams remain at
`/tmp/ares-options-9ae520aa-initialized.OIYjGU`.

Final dedicated native run5932 terminal0 /2.06s:
`/tmp/ares-ard73-native-dedicated.tQF6kr/case-C7PKqH` full GREEN.
Original overhang replay remains full RED101 /0.77s,
`/tmp/ares-ard73-overhang-replay.cfkW4J/output/case-AGZReD`, line859 speed.
Original KSR replay19161 terminal101 /25.27s:
`/tmp/ares-ard73-ksr.ImyJAd/case-ymi1Ne`, earliest difference is now config
comment `different_settings_to_system` line88, not model-time seconds.
Candidate immutable executable `/tmp/ares-ard73-candidate-orca-parity`
SHA256 `1eede3a70ceb8b9221129886f361c32e2483eed5a35ecb3b291eef2adf87c31c`.

Tier-1 run67065 terminal0: core WASM120s, Windows11.25s, macOS12.12s,
Linux10.95s, WASM adapter7.60s. Fmt passes; Clippy21735 terminal0 /48.42s.
Workspace49508 fails:5694 PASS,1 FAIL,9 skipped,1763 not run /102.676s.
Existing KSR semantic diagnostic expects the historical official tiny branch
at layer48. Do not remove its comparison or add tolerance/filtering.
Fresh initialized native59325 same original input SHA698f40f1...b8a9,
`/tmp/ares-ard73-ksr-oracle.mh78O9/plate_1.gcode`, matches the preserved
initialized reference except generator timestamp. A temporary semantic
reference substitution76158 is still RED /20.214s at config-block line75:
missing `different_settings_to_system`. The substitution has been restored;
old fixture streams remain unchanged. Investigate the source-owned config
export before migrating the diagnostic's reference contract. Full workspace
no-fail-fast45100 is live to identify any additional failures.

Independent review confirms the sensitive complete RED/GREEN and parameter
owner, requests explicit initialized-reference identity in the historical
printer report (fixed), and withholds final approval until workspace closes.
No commit/push has been made for this unfinished slice.

The original project bytes omit `different_settings_to_system`; this is not
a raw-token serializer failure. `OrcaSlicer.cpp:2815-2818` creates/resizes the
CLI vector to filament_count+2; `PrintConfig.cpp:1828-1830` defines coStrings,
and `GCode.cpp::append_full_config:5636-5655` serializes configured keys.
For the two-filament KSR native CLI inserts four empty strings (`;;;`).
Rust typed legacy deliberately defers this bookkeeping and existing export
only echoes typed or supplied raw keys. Any repair must first specify the
PrintConfig coStrings/GCode export source boundary; CLI preset orchestration
is only a temporary validation shell, not a new Ares-owned pipeline.
Do not hard-code the KSR token or add a blanket default to every printer
without sensitive native absent/present/vector-width witnesses.
