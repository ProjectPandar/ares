# F2 multi-group public Native RED

Coordinator ruling791a9fbe9 requires this RED before F1/F2 production.
Fresh root: `/tmp/worker3-fuzzy-f2-public-red-26s_wcqg`.
`plan.json` records commands/environments, CPU12 single-core, jobs2/threads2,
sequential official/seeded legs and fresh per-leg Native cache/artifact roots.
Native identity, input/modifier contract, raw output and API/CLI evidence are
external artifacts. Repository 3MFs are inputs only; no Native output is committed.

Both modes run7 tests:0 PASS/7 FAIL, nextest exit100. All14 Native slices exit0
with full G-code. Every validated API call rejects existing fuzzy_skin scope,
and every real CLI exits1. No strict comparison is performed on rejected Ares
output; required and actually performed comparison fields are separate.
All inputs use deterministic Ripple, so GREEN requires unchanged strict full
ordered generator bytes in both modes, plus raw API/CLI equality. No retry,
normalization or weakened comparator. Existing F1 tests remain unchanged.

Inputs cover equal wall settings with None/External in both directions,
two different active effects, identical-effect types, inner eligibility,
later overlapping modifier priority and an open variable-width crossing.
Public load verifies modifier volume identities; input XML settings/overrides
and Native effective global fuzzy settings are checked before Ares calls.
`comparison.json` and `native/result.json` bind the failure to successful Native
admission followed by the old Ares guard. `red-evidence.json` audits raw hashes.

Source owners: FuzzySkin.cpp415–473/696–814, Algorithm/LineSplit.cpp64–314.
No production source is changed before this RED commit/report. Test-target
strict clippy and fmt pass on CPU12; all new files remain under400LOC.
F1 noise/junction production follows as one commit; F2 masks/ordered split
follows separately. Review/full once on the complete stack, merge together.
