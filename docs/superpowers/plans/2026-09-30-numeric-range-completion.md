# Numeric range completion implementation plan

Boundary: ARD-0059; `Config.hpp:2475-2479` default numeric limits and existing
native `domains`/`bounds`/`OptionOutcome` schema projection. Include honest
completion reporting and expression interpretation; defer individual missing
endpoint activation/production repairs. No independent Ares pipeline.

1. RED: real generated one-sided and default-bound plans must remain INCOMPLETE
   even with every generated sample compared; preserve compared counts. Keep
   positive tests for previously verified domains.
2. Correct completion flags and omission explanations, preserve generated local
   probes, distinguish unknown expressions from omitted assignments. Remove
   obsolete tests that require a minimum count of falsely complete domains.
3. Run targeted original-binary comparisons for affected float/vector/percent
   domains. Expect retained successful comparisons but a failed completion gate.
   Run workspace Nextest, fmt, Clippy and Tier-1 compiler checks.
4. Independent six-axis review and re-review of fixes; commit code and docs
   together only after validation. Old full-scan PASS totals are not proof of
   the corrected numeric coverage contract.

RED: `/tmp/ares-range-completion-red.log` reports travel_speed incorrectly
PASS rather than INCOMPLETE (1 pass, 1 fail, 0.489s). Bound parser RED:
`/tmp/ares-range-bounds-red.log`, 2 failures in 0.015s, demonstrating numeric
prefix acceptance and commented-out assignment acceptance.

Initial GREEN: `/tmp/ares-range-completion-green.log`, 14 tests pass (1.861s).
Local probes remain unchanged; missing-bound branches no longer certify them.
Shared numeric bound parsing now consumes whole active assignments, allowing
known max_temp and finite numeric literals while preserving unknown-expression
classification. The obsolete >=519 complete-domain count assertion is removed;
named positive domain and limited-width outcome contracts remain.

Concurrent immutable 3996459c scan terminated exit100 after 1354.452s:
`/tmp/ares-options-3996459c.log`, artifacts
`/tmp/ares-options-3996459c.AlwbIq`. It reports 559 PASS / 89 INCOMPLETE /
2 FAIL, 1644 generated / 1486 strict comparisons / 1 rejection. It predates
ARD-0058 and this gate correction; numeric PASS totals are not completion proof.

Original-binary selected sweep ended exit100 as required by the corrected gate:
`/tmp/ares-range-completion-sweep.log`, 17.748s; artifacts
`/tmp/ares-range-completion-sweep.ZUDc0R`. All 26 generated cases strictly
compare, with zero rejections. Seven limited numeric domains remain INCOMPLETE;
hot-plate, skirt height and vitrification are the three PASS domains. Verified
all 26 manifests' input/output lengths and SHA256 hashes. This is actual full
output evidence, but not evidence for the endpoints still missing. Manifest
producer identity remains unknown, separate from the recorded invocation.

WASM, Windows GNU and macOS ARM compiler checks completed successfully;
logs `/tmp/ares-range-completion-{wasm,windows,macos}.log`. These are not
cross-platform runtime-parity claims.

At this intermediate stage, full checks and independent review were pending;
the final results are recorded below.

Review found two remaining gaps: width extraction still accepted expression
prefixes, and explicit bounded absolute seeds used GUI max_literal rather than
the full range. Both were reproduced (`/tmp/ares-range-review-red.log`, 2 FAIL,
2.062s), fixed by sharing numeric_bounds and using the real maximum, then
regressed (`/tmp/ares-range-review-green.log`, 16 PASS, 2.967s).
The missing-bound local probes remain unchanged; explicit bounded literal
seeds now use their full interval.

Additional official sweep `/tmp/ares-range-seed-sweep.log` ended exit100 in
26.368s, artifacts `/tmp/ares-range-seed-sweep.AxGjEc`: spiral smoothing 6/6
strict comparisons PASS; support overlap 6 cases remain INCOMPLETE because
the upstream support generator is unimplemented. No rejection or missing
support output was counted as strict parity. Exact-version checks and
independent re-review followed, as recorded below.

Final selected regression (including width-schema parsing) ended exit100 in
42.523s: `/tmp/ares-range-completion-final.log`, artifacts
`/tmp/ares-range-completion-final.19UUmh`. All 31 generated cases strictly
compare; four domains PASS and the same seven missing-range domains remain
INCOMPLETE, with zero rejections. The added line_width domain compares 5/5.
All 31 manifests have verified file lengths and SHA256 hashes. The separate
seed sweep also has verified hashes for six successful spiral outputs and
18 divergent support artifacts (three oracle attempts per case); those
divergent artifacts are not counted as strict comparison successes.

Exact final-version validation chain completed exit0: workspace 7432 PASS /
9 skipped in 264.743s; Clippy successful with existing 248 core / 28 parity
test warnings; rustfmt and diff checks clean. Logs:
`/tmp/ares-range-completion-{workspace-final,clippy-final,test-clippy-final,fmt-final}.log`.
Independent final six-axis review approves this ten-file finite slice, with
no remaining required code fixes; its request to replace stale pending statuses
with final evidence is applied here. Actual missing numeric endpoints and
whole-program parity remain open.
