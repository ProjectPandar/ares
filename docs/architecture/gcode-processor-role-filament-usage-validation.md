# Role filament usage GREEN validation

The core result now carries Native's complete sparse per-role meters/grams map.
Fresh paired seeded controls both pass 685/685; all 527 retained Ares G-code
streams have identical **raw** SHA256 values between base and candidate.
No case changes status and no candidate-only failure exists.

## Source boundary and scope

Baseline: `b1f65d40d`. Approved RED `bcae9218a` rebased without production
changes to `1fd859b3c`; supplemental RED `923da199c` precedes production.
GREEN `06f499d8311014f919e1597ffb92457ed42d470e` changes three production
files (42 additions, one deletion): core role ordering/serialization, the
public statistics field, and the existing UsedFilaments compatibility shell.
Upstream is OrcaSlicer `8500fcdcc`.

- `GCodeProcessor.cpp:1621-1639`: flush the double role cache using current
  filament metadata, assign the first pair or add to an existing pair, reset.
- `:5720-5724`: flush before assigning every role tag, including repeated tags.
- `:1641-1649,6031-6037`: finalize and expose `used_filaments_per_role`.
- `:1525-1547,3895-3911`: model, support and tower ordinary extrusion adds
  the existing f32 per-move volume to the f64 cache.
- `:5937-5950`: ToolChange does **not** flush this cache. Physical/virtual
  flushing also does not contribute to it (`:1606-1619,3955-3968,4306-4324`).

Diameter/density begin as f32 metadata and promote to f64. Formula order is
`PI * (radius * radius)`, then `cache / area * 0.001` and
`cache * density * 0.001`. Density vector scalar semantics match
`GCodeProcessor.cpp:2083` and `Config.hpp:624-628::get_at`; this is the
upstream scalar expansion, not first-object substitution. Existing G-code,
color-change, timing and categorized-volume behavior is unchanged.

WASM production code is unchanged: existing statistics serialization carries
the map. A public slicing test verifies JSON pass-through byte for byte.
Per the coordinator's GREEN instruction, FeatureType projection/UI is deferred
from the broader original spec. No browser UI implementation is claimed.

## Native oracle and TDD evidence

The approved real project is the unmodified Kobra3 fixture documented in
`tests/parity/role-filament-usage/README.md`. The existing Native ELF SHA is
`59f9ea7e16165df6979a243de7d6acd49d8cf523ac5de6d3fec08d6e25c7896e`.
The gdb reader validates the ELF, disassembly-derived offsets, complete map,
node counts and final cache reset. No Native rebuild was used.

Native's same slicing process yields eight roles totaling
343.16809722384134 mm / 1.0235158534141071 g. Rounded to its footer format,
these equal its 343.17 mm / 1.02 g footer. An uninstrumented Native run agrees
on the entire G-code with generated-by-only normalization. Ares's entire
G-code also agrees; normalized SHA256 is
`431cecf3a124f0cde14e1933332245ea60abff21198904314ca962b74003b128`.
The RED fails because the public map is missing, while the G-code control and
Native extraction/footer proof pass. GREEN compares every role and both f64
bit patterns without tolerance.

Supplemental RED tests cover a role cache spanning unequal-material T0→T1,
repeated/revisited tags, support/interface/transition, tower, ColorChange,
pause, retract/travel/wipe, physical/virtual flushing, EOF and WASM transfer.
These tests were observed failing before production changes. The supplemental
Native six-role map comes from the same validated memory reader in process
mode, with exact pair-bit comparison. Its footer check is explicitly null:
flush-inclusive totals and unequal-material cache conversion have different
scopes. The original same-process slicing/footer proof remains mandatory.

The supplemental input's wipe tags were corrected to the Native compatible
producer dialect and the reference was re-extracted from real Native; no
assertion was relaxed. The independent preexisting producer-tag difference
is recorded separately at `/tmp/worker6-processor-bbl-wipe-tag-finding.md`,
outside this slice. Final input SHA256:
`c1b111998cd46434ff0ae974ee001248a7e2269911a55c5008ef326e713940ac`.

## Gates and repeatable artifacts

| Gate | Result | Log/artifact |
| --- | --- | --- |
| Directed + live Native | 11/11 PASS | `/tmp/worker6-role-directed-green-final.log` |
| Fresh seeded base | 685/685 PASS | `/tmp/worker6-role-seeded-base.log` |
| Fresh seeded candidate | 685/685 PASS | `/tmp/worker6-role-seeded-cand.log` |
| Raw G-code invariance | 527/527 identical; zero status changes | `/tmp/worker6-role-seeded-comparison.json` |
| Strict workspace clippy, all targets/features | PASS, `-D warnings` | `/tmp/worker6-role-clippy.log` |
| rustfmt | PASS | `/tmp/worker6-role-fmt.log` |
| wasm32 core/vgcode/wasm, all features | PASS | `/tmp/worker6-role-wasm.log` |
| Workspace nextest | 8093/8093 PASS; 9 existing ignored tests skipped | `/tmp/worker6-role-workspace.log` |

Seeded receipts are retained under
`/tmp/worker6-role-seeded-base-verified-artifacts` and
`/tmp/worker6-role-seeded-cand-artifacts`, with independent fresh caches and
target directories. The pairing checks labels and the sorted uncompressed
ZIP member names/content SHA256 values: generated 3MF ZIP metadata differs
between runs, while every paired input member agrees. The **output** SHA
comparison performs no normalization. Reproduce with `/tmp/worker6-role-compare-seeded.py`.
Changed-case list: **empty**. Candidate-only failures: **empty**.

The first two base startup attempts lacked the worktree's readonly
OrcaSlicer resource link. They are excluded prerequisite failures, retained
as `worker6-role-seeded-base-precondition.log` and
`worker6-role-seeded-base-resource-startup.log` under `/tmp`. The valid fresh
base run began after the resource link existed and passed all 685 cases.
No timeout/failure from an excluded startup is counted as a comparison PASS.

Workspace uses `/tmp/worker6-role-workspace.sh`, a copy of control-set v2
whose nextest entry is `cargo nextest run --workspace`, retaining all seeded
Native env, resource references and 16 test threads. The coordinator's current
GREEN gate asks for seeded G-code invariance, not a new official round.
Native golden G-code retains its original whitespace/EOF bytes; source diff
checks exclude those byte-exact files, rather than trimming the oracle.

## Evidence retention

The sealed RED index `/tmp/worker6-role-oracle/evidence-index.json` remains
unchanged. Its 40 original files are also archived under
`/tmp/worker6-role-red-archive/index.json`, each verified against that sealed
SHA256. Two subsequently edited tracked files are restored from the RED Git
blob. The overwritten runtime Ares statistics JSON is restored by removing
only the new GREEN field and verifying the resulting bytes against its
original sealed SHA256; current GREEN runtime output remains intact.

GREEN index: `/tmp/worker6-role-green-evidence-index.json`, 40,887 files,
SHA256 `7c8171c027a8667c3d81252f895b00e6a5870025175c3a127e0ddbf63f44acf3`.
It records tested HEAD `a58f3e9de` and preserves logs, Native captures,
per-case retained outputs, input/manifest receipts and runtime statistics.
The final report is outside the index to avoid a circular content hash.
RED original index SHA256:
`c62ae74228862b4373b00e51088a1917fe8b5e12d04366bbaac1b978ab3ab8ca`;
RED archive index SHA256:
`87a75c22aaad401ce1f7158dd0335978e083fdac0e674cef0e77ea0d041ca7bf`.

Workspace run ID `1b171863-bf1b-414e-aa64-1b5243e59056` finishes with
8093 passed (31 slow), 9 skipped, in 753.084 seconds after compilation.
The nine ignored tests already exist on the baseline; this slice adds none.
This final-tree run includes the final corrected fixture and passes all three
public role-oracle/WASM tests plus the supplemental processor boundary test.
No redundant second full run is needed. Validation-only documentation is the
only change after that tested tree. Review is requested before any push.
