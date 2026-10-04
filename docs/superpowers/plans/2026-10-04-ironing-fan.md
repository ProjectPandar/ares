# Activated ironing fan plan

Source boundary and non-negotiable decisions:
`docs/architecture/ironing-fan.md`.

1. Activate the existing public option domain and add saved3MF/native
   witnesses for real ironing extrusion and subsequent role exits. Run both
   references against the unchanged production emitter and retain RED.
2. After the overlapping lint slice lands, fetch/rebase current main. Port
   only the cited producer/request consumer behavior; retain separate ironing
   request state and its distinct inactive-END semantics. No legacy fallback.
3. Run both reference suites serially at CPU0, including prior support fan
   controls. Keep new artifacts separate from historical stock/matrix runs.
4. Run full workspace Nextest, Clippy, fmt and core/WASM compilation. Keep
   Cargo build jobs bounded and record exact revisions and exit statuses.
5. Commit/push the feature branch, print `REVIEW-REQUEST <hash> <summary>` and
   stop. Only after matching approval, while main is unfrozen, rebase latest
   main, repeat gates and fast-forward push. Never force a rejected push.

## Preceding matrix progress

Support fan commit61f1521c was approved, rebased without patch changes to
16ac907b and fast-forwarded frome865eb2a. Final evidence:
`/tmp/ares-pi-interface-fan-land.ItSfQk`: official/seeded each12 full outputs,
workspace7595 PASS/9 existing skips, Clippy/fmt/WASM exit0, main push exit0.

At16ac907b the two old Tree FAIL domains now pass both references:
`tree_support_branch_diameter_angle` and `tree_support_tip_diameter`, each
min/max/seeded,6/6 full comparisons per reference and no rejected cases.
Evidence: `/tmp/ares-pi-option-support-16ac907b.tBBWrt`.
This does not relabel the old650-domain scan or close SLA/application and
225 historical INCOMPLETE domains. No old stock evidence was modified.

## Current state

Branch `pi/option-support-recheck`, based on16ac907b. Only test activation,
native witnesses and documentation changed; no production ironing fan code.

RED evidence: `/tmp/ares-pi-ironing-fan-red.DXX0Nz` (archived workspace test
executables; both references and Ares CPU0, references serial). Both runs
exit100 with the two expected test-function failures, not infrastructure
errors. Every one of the six native controls passed saved-setting, positive
ironing extrusion, subsequent other-role extrusion and PWM witnesses.
Disabled output passes; off,45%, closed layers, overhang enabled and ramp
controls all fail complete-output comparison. The first four active request
cases lack M106 at line212;45% expects S114. The closed-layer control first
lacks M106 S0 at line508, independently exposing inactive ironing END.

The activated option domain generates and strictly compares all3 values,
rejects0 and is FAIL in both references. Max100 first lacks M106 S255 at
line3697 (byte79173); all3 budgeted native replays complete without a match.
No comparator or acceptance rule was relaxed. rustfmt and all-target Clippy
exit0; parity warning count stays21 with no new ironing-test warnings.

Next: synchronize the lint owner's `role_fans.rs` refactor from main, then
implement the source-owned producer/consumer behavior. The RED test phase
is not a green release gate. The coordinator explicitly requested a RED-only
checkpoint commit/push before lint handoff; this is not a merge-ready review
request. Keep this branch for resuming ironing implementation after notification.
Meanwhile investigate a separate non-cooling option group from latest main.
