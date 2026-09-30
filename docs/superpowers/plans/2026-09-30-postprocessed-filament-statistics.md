# Postprocessed filament statistics plan

Base: b1050038. Decisions: ARD-0071.

1. Minimize the observed KSR trailing-zero difference to a small native
   two-filament fixture. Require active input and complete ordered RED.
2. Rank falsifiable causes, then port the cited processor-vector behavior
   without altering writer-side omission or unrelated output.
3. Require full ordered GREEN and zero-density/zero-cost gate regressions;
   inspect original KSR output without adding normalizations.
4. Run nextest workspace, fmt/Clippy and Tier-1 checks. Independent read-only
   six-axis review, repair/re-review, commit/push code and docs together.

Status: isolated complete RED followed by complete GREEN; implementation
preserves line gates and formats configured-width vectors. Counterfactual,
line-omission regressions and final full gates passed; independent final
six-axis review approved this finite slice. Global parity remains incomplete.
ARD-0072 zero-slot normalization is now committed and pushed. The native
fixture uses the existing Manual plate-owner binding to isolate statistics.
Immutable 541370f1 full option sweep terminated with 386 PASS / 262 INCOMPLETE
/ 2 FAIL; this remains a separate global observation, not slice acceptance.

Reproduction ledger (all comparisons retain complete output; no new filters):

- Tiny 0.8mm Ender cube, 31551 exit 100 / 1.757s: native writer gates omit
  weight/cost lines, so this does not activate the target behavior and is
  excluded as a statistics RED; `/tmp/ares-filament-stats-vector-red.log`.
- 10mm two-filament Ender cube, 71225 exit 100 / 2.195s: genuine complete
  RED, first line 49 missing `T0`. Independent full diff also has the target
  missing trailing zeros, M73 ordering and prime-tower config differences;
  `/tmp/ares-filament-stats-vector-cube10-red.log`. This is not isolated.
- P1S, explicit disabled M73 and prime tower, 28016 exit 100 / 2.314s:
  complete RED at line 16, raw one-element per-extruder option vectors are
  expanded differently. Target footer divergence remains present;
  `/tmp/ares-filament-stats-vector-bbl-red.TmsF4y/case-x9DR7g`.
- H2D initially has an invalid missing filament-colour fixture, 73186
  exit 100 / 1.809s: native rejects filament count mismatch, not output RED;
  `/tmp/ares-filament-stats-vector-h2d-red.log` is excluded as parity evidence.
- H2D with explicit distinct colours, 9261 exit 100 / 2.355s: complete RED
  at line 17, expected air-filtration vector `1,0`, actual `1,1`;
  full artifacts `/tmp/ares-filament-stats-vector-h2d-colours-red.eejzpR/case-2i4tkh`.
  Final input construction remains WIP, not an accepted regression fixture.

The earlier fixture attempts above did not establish footer acceptance.
Their normalization prerequisite was subsequently resolved in ARD-0072;
they remain retained failures, not replacement GREEN evidence. No output
filtering or blind expected-value updates were accepted to bypass them.

After ARD-0072 and explicit Manual plate-owner binding, 34857 terminated
exit 100 / 1.523s with an isolated complete RED at line 6462: native
`; filament used [g] = 0.70, 0.00` versus Ares `0.70`; the cost line likewise
omits trailing `0.00`. Full pair
`/tmp/ares-filament-stats-zero-prerequisite.NNZiCV/case-abBheR`, expected
162048 / actual 162036 normalized bytes. No normalization was added.

Replacing only the sparse formatter with configured-width zero formatting
gives 34667 exit 0 / 1.523s complete ordered GREEN; log
`/tmp/ares-filament-stats-full-green.log`, complete pair
`/tmp/ares-filament-stats-full-green.iqkRpV/case-qCafNY`. No numeric accounting or line
presence gate changed. This is not a global completion claim.

Pre-review found no production blocker and requested stronger activation/gate
coverage. The public test now verifies exact positive density/cost settings,
exactly two statistic slots with a positive first value and trailing `0.00`,
and cost zero retaining weight while omitting cost. Run 40946 exit 0 / 3.574s
passes both this two-case test and the ARD-0072 zero-density fixture (both
lines absent), with all three complete native comparisons retained under
the dedicated `/tmp/ares-filament-stats-gates.jJoZUR` artifact root:
`case-j44Z3U` positive cost, `case-gpHV1A` zero cost, and `case-iHBStu`
zero density/cost. Each manifest preserves exact full bytes and SHA-256.

KSR replay of the already-built workspace binary, 15284 exit 101 / 41.43s,
remains DIVERGENT at model-time header line 3 (native 1h43m53s versus Ares
1h43m52s), expected 6321717 / actual 6322022 normalized bytes. Full pair
`/tmp/ares-filament-stats-ksr.ewYppd/case-6KcY3H` and replay-summary.json
are retained. Comparing full Ares output with the ARD-0072 baseline shows
exactly two changed lines: weight `34.35` and cost `0.86` now each append
`, 0.00`; no other bytes changed. This repairs the cited footer difference
in the actual KSR project without claiming full KSR parity.

Tier-1 check 58053 terminated exit 0: core/vgcode WASM 1m27s, Windows GNU
11.37s, macOS 11.99s, Linux 10.92s; WASM adapter 11.46s. Logs are
`/tmp/ares-filament-stats-<target>.log` and `-wasm-adapter.log`.

First full gate 96593 exited 100: 7457 PASS / 1 FAIL / 9 skipped, 253.979s;
Clippy was not reached. The old public KSR statistics test required weight
and cost lines not to contain a comma, confusing writer intermediates with
postprocessed output. The actual native KSR full stream independently has
`34.35, 0.00` and `0.86, 0.00`. Its test now requires exactly two slots and
the unused `0.00` on all four existing lines; numeric accounting assertions
remain unchanged. A full rerun is required before approval.

Independent pre-review approves this old-test migration using the retained
native KSR stream, not Ares-derived expectations. Final native fixture run
71602 exit 0 / 2.804s verifies both positive/zero cost complete streams;
`/tmp/ares-filament-stats-final-native.log`.
Restoring only the formatter's old last-Some truncation yields counterfactual
20080 exit 100 / 1.518s, same first difference line 6462, expected 162048 /
actual 162036 normalized bytes. Complete pair
`/tmp/ares-filament-stats-counterfactual.h4rmQo/case-MygqGz`. The configured-width
formatter is restored before final full gate 9693; no counterfactual edit is
part of the candidate. Final fmt/workspace/Clippy logs use `-final.log`.

Final gate 9693 terminated exit 0: workspace 7458 PASS / 9 skipped,
253.721s; fmt clean; Clippy workspace/all-targets 47.13s, existing warnings
only. Full logs `/tmp/ares-filament-stats-workspace-final.log`,
`/tmp/ares-filament-stats-fmt-final.log` and
`/tmp/ares-filament-stats-clippy-final.log`. No new file reaches 400 LOC;
changed production finish.rs is 352 LOC and existing tests.rs is 391 LOC.
