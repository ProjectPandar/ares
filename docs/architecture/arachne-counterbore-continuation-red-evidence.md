# Arachne non-None counterbore continuation — refreshed RED

Approved SPEC: `97aa276f9f2377fd54f1afb024c4ed3491bb0664`.
Base: `5a9687ac020962f254b6262f2184f3345025c70f`.
Branch: `worker4/arachne-counterbore-next`.
Frozen public RED test commit: `19eaf9b964ec19c46839a192ca22c38af5eb8869`.
Production, comparator and ten old capability/lifecycle assertions are unchanged.

## Frozen public surface

Extracted ten non-None tests/helper from archived `eb1230fe7` without any assertion
change; exact text reconstruction passed after omitting only its None test.
Matrix retains only two modes × five original geometries. Main-owned
`arachne_counterbore_none` supplies the original None regression instead.
STLs, geometry manifest and parent-chain SHA checks remain unchanged. Both APIs
explicitly select Arachne and the tested mode; stock process defaults are not
claimed to be Arachne. Placement, arcs, deterministic metadata and complete
ordered-byte generator-only comparator are unchanged.

## Directed execution

Root: `/tmp/worker4-arachne-counterbore-refresh-Ghwt`.
Launcher: `/tmp/worker4-arachne-counterbore-refresh.sh`, outer `taskset -c 15`.
W4 independent compile target: `/tmp/worker4-arc-paired-eyIt/target`.
Jobs1, `ARES_PARITY_THREADS=1`, nextest test-threads1. Fresh official/seeded
Native caches and artifact roots. Official then seeded sequentially.
Command: `cargo nextest run -p ares-cli --test orca_parity
-E 'test(counterbore_)' --no-fail-fast --test-threads 1`.
START/END inbox receipts were recorded; no full/workspace test run or push.
The launcher's inherited historical `range-diff.log` is not evidence of this
fresh branch's history; this RED is bound to the explicit head/base above.

| Producer | Result | Runtime | Exit |
| --- | --- | --- | --- |
| Official | 22 run:14 PASS,8 exact guard RED | 20.174s | 100 |
| Seeded | 22 run:14 PASS,8 exact guard RED | 25.950s | 100 |

1205 unrelated tests were excluded by the filter, not tested or ignored.
Each mode's stepped-hole, first-layer-hole, straight-hole and independent
two-islands public cases return exactly
`UnsupportedProjectFeature("counterbore_hole_bridging")` after Native success.
Both primary modes have Native-vs-None complete-output and motion activation
witnesses, with both Native streams normalized as Orca. These supplements do
not replace the public Ares complete-stream acceptance or claim all branches.

PASS controls: two non-None solid cases, main-owned Arachne None, and11 classic
partial/None/Filled cases. All14 control Ares raw SHA values match across modes
and are frozen before production in `verified-red-receipt.json`.
Arachne None remains full-output PASS, with merged-prerequisite raw SHA:
`5f55844348571cd26b85fecde5bdc0ae8d8994b14bd569885bdeb0e6170da972`.
There is no additional divergent baseline case in this refreshed run.

`verified-red-receipt.json` also stores every case's artifact path, hashes,
settings/overrides, exact error, status and comparison detail. Raw Native/Ares
streams/errors, commands/logs and Native reachability witnesses stay under each
mode's artifact tree. This is verified RED, not GREEN or merge evidence.

## Checks and boundaries

Test-target `cargo clippy -p ares-cli --test orca_parity -- -D warnings` and
rustfmt/fmt-check PASS (exit0). Shared Native before/after SHA unchanged:
`64515d01f887b4797105530751a3ad59b0fa8537fbe3a294c420e1e14bba3b60`.
Official wrapper SHA (setarch -R):
`1689a2ff46564cf723ad90c663051b58115920c3cce639bd2961340955cebbf3`.

No new None assertion, geometry replacement, disabled arc, movement-only
comparator, retry-to-green or producer fallback. No classic/W3/W5/W6 production
changes. Nested-island and compatible-multiregion public reachability gaps remain.
Next step is a separately reviewed per-case exact diff for the ten old Arachne
rejection/lifecycle assertions; no such assertions have been modified here.
GREEN remains restricted to the three production files in the approved SPEC.
