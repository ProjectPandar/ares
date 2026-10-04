# Activated ironing fan plan

Source boundary and non-negotiable decisions:
`docs/architecture/ironing-fan.md`.

1. Activate the existing public option domain and add saved3MF/native
   witnesses for real ironing extrusion and subsequent role exits. Run both
   references against the unchanged production emitter and retain RED.
2. Per updated coordinator instruction, implement now on current main, before
   the overlapping lint slice lands. Port only the cited producer/request
   consumer behavior; retain separate ironing request state and its distinct
   inactive-END semantics. No legacy fallback.
3. Run both reference suites serially at CPU0, including prior support fan
   controls. Keep new artifacts separate from historical stock/matrix runs.
4. Run full workspace Nextest, Clippy, fmt and core/WASM compilation. Keep
   Cargo build jobs bounded and record exact revisions and exit statuses.
5. Commit/push GREEN on the feature branch without requesting review yet.
   After lint lands, rebase, resolve role_fans.rs conflicts and repeat gates;
   then print `REVIEW-REQUEST <hash> <summary>` and stop. Only after matching
   approval, while main is unfrozen, rebase latest main, repeat gates and
   fast-forward push. Never force a rejected push.

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

### Implementation authorization update

The coordinator now authorizes implementation before lint handoff. The original
RED checkpoint8b55b910 remains on its remote branch. It was rebased onto
be2a2990 as8ac40fbe; the sole activation conflict was resolved by retaining
all six geometry enablers and the ironing fan enabler. Work continues on
`pi/ironing-fan-port`, avoiding a force push of the published RED branch.
Refresh RED at this baseline, then implement; no REVIEW-REQUEST until the
later lint rebase and revalidation. After commit/push, group the225 remaining
INCOMPLETE domains by cause and prioritize genuine activation coverage.

### GREEN candidate and freeze

Fresh RED on8ac40fbe: `/tmp/ares-pi-ironing-current-red.70DGH7`, both references
exit100 with the same missing START and closed-layer END commands. The
implementation adds an ironing producer marker/state and a separate cooling
request, preserving source priority and unconditional ironing-END restoration.

Final candidate evidence: `/tmp/ares-pi-ironing-green.yUDlI2`.
Official and seeded each38/38 complete outputs PASS (4 test functions each):
23 option cases across8 domains,6 ironing controls,6 support controls and3
interlaced cooling/equalizer controls. No case rejection or comparator change.
Workspace7596 PASS/9 existing skips (1186.404s,8 threads); all-target Clippy,
fmt check and core/adapter WASM compilation exit0. Clippy retains219 core-lib,
220 core-test and21 parity warnings, with no added lint warning. Changed Rust
files remain below400LOC. The tested tracked Rust diff SHA256 is
`71e481aeb67f1e04e686510a949b9165631c66fed8548460af4e12455e8bbcf7`.
WASM verification is compile-only, not a browser-runtime claim.

The coordinator has frozen main for the lint G-code batch. Commit/push only
the feature branch now; do not push main or issue REVIEW-REQUEST before the
later lint rebase and repeated validation. Wait for explicit unfreeze before
any main integration. No temporary probes or suppression attributes were added.

### Lint handoff completed

The coordinator explicitly unfroze main and requested fan revalidation after
95e23008. The speed fix landed asf10c3b17; rebase a07dd972 onto that main.
The RED/implementation commits become b85bf07b/bf7f17d4. The sole conflict
was the FORCE_RESUME block in cooling/role_fans.rs: retain the lint-extracted
resume_fan_commands helper and match-based auxiliary comment selection, adding
ironing_speed to its arguments and its no-active-request condition. Do not
restore the old nested block. Preserve the new region ironing_speed selection.

Publish subsequent work on pi/ironing-fan-lint without force-rewriting the
old candidate branch. Repeat both native references with both fan families,
interlaced controls, filament-speed witnesses and ten ironing-related option
domains. The two unbounded speed domains must stay INCOMPLETE. Then repeat
all four quality gates and request coordinator approval; no unapproved merge.

### Post-lint GREEN evidence

`/tmp/ares-ironing-fan-lint-gates.1gpw70` freezes bf7f17d4 on f10c3b17.
Both official and seeded references pass44 complete outputs each:17 public
controls (6 ironing fan,6 support fan,3 interlaced and2 filament-speed)
plus27 option comparisons. All10 option domains are exercised:8 bounded
PASS and2 honestly INCOMPLETE speed domains,0 rejection or DIVERGENT.
The option commands' exit100 is solely incomplete endpoint coverage.
Workspace7597 PASS/9 existing skips (879.859s,8 threads), all-target Clippy,
fmt and core/adapter WASM compilation all exit0. Lint counts remain at the
new main baseline157 core-lib/158 core-test/21 parity; no suppression added.
WASM evidence remains compile-only. All changed Rust files are at most400LOC.
The tested source diff from f10c3b17 has SHA256
`03c4f227579e07bcda7e029a285818b64744a3e387a00dcd0ac4ce9eb041c936`.
Only documentation changes follow these gates. Request review of the new
pi/ironing-fan-lint head; do not merge before matching coordinator approval.
