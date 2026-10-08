# Stock-circle Model/BuildVolume port: verification

## Boundary and commits

Approved spec `4e9ce4d3b`; independently committed RED `5abf8ef04`;
GREEN `368bf6284`, based on `aaa89f76a`, then rebased unchanged onto
`7a3216f67`: spec `7f6b5a869`, RED `5eb1f79fb`, GREEN `f8400605e`.
GREEN's stable patch-id is identical before/after rebase:
`08f94f7971f4eb4500f55812d15712459874b0dc`.
Source owners and immutable numeric decisions:
`2026-10-08-build-volume-circle-boundary.md` and
`../superpowers/specs/2026-10-08-stock-circle-print-volume.md`.

The full stock-circle archive (SHA256
`446e19e8f8dc9443c68774bc31fca6d0187798e41031882919cf0930362be8f6`)
is retained as `tests/parity/validation/o6-stock-circle-cylinder.3mf`.
The core RED accepted G-code (`/tmp/worker3-o6-circle-red.log`), not a compile
failure. GREEN asserts Native's exact -50 diagnostic. All seven O6 core E2E
cases passed. No CLI production changes or CLI -50 assertions are included:
worker4's family adapter is not yet merged into this base.

Native's one fresh original slice, stdout/stderr/result.json and matched-symbol
GDB receipts remain in `/tmp/worker3-o6-stock-circle/`. Both sparse original
input-read crashes are archived separately in
`tests/parity/validation/o6-circle-sparse-{input-read-failure,no-brim-input-read-failure}.3mf`;
see `2026-10-08-o6-circle-native-input-read-failure.md`. They are non-parity
observations, not accepted slicing controls.

## AAA-base final-source gates

| Gate | Result | Receipt |
| --- | --- | --- |
| Workspace nextest | 8012/8012 PASS; nine existing skips | `/tmp/worker3-o6-circle-workspace-final.log` |
| Full fresh seeded candidate | 623/623 PASS, zero skips | `/tmp/worker3-o6-circle-seeded-final.log` |
| Fresh clean aaa seeded base | 623/623 PASS, zero skips | `/tmp/worker3-o6-circle-base-aaa-seeded.log` |
| Normal output invariance | 529 vs 529 **raw SHA256 multisets identical** | `/tmp/worker3-o6-circle-seeded-final-byte-audit.json` |
| Clippy workspace/all targets | `-D warnings`, zero warnings | `/tmp/worker3-o6-circle-clippy-final2.log` |
| Rustfmt | PASS | `/tmp/worker3-o6-circle-fmt-final.log` |
| Browser WASM check | PASS | `/tmp/worker3-o6-circle-wasm-final.log` |

The raw-byte audit also records generator-only hashes, but raw equality is the
stronger result; no M73 or motion/body lines are normalized. Audit script:
`/tmp/worker3-o6-circle-byte-audit.py`. Both sides' per-artifact byte length,
raw SHA256, path and test-binary identity are included in its JSON receipt.
The clean base is a detached `/tmp/worker3-o6-circle-base-aaa` worktree, with
its own target directory. Worker5's supplied artifact directory was inspected
read-only but **not reused**: manifests reported revision `unknown`, and the
current directory held 530 outputs versus final2's documented 529. No earlier
e5 stock/seeded receipt is presented as an aaa gate.

## Rebased 7a3216f67 gates

Fresh full seeded run, limited to 16 threads: **646/646 PASS**, zero skips,
`/tmp/worker3-o6-circle-seeded-7a.log`. Its 541 outputs preserve all 529
common AAA outputs' **raw SHA256 multiplicities**, with no missing/changed
old output; the twelve added files are `c1-threshold/*` controls introduced
by main. The 541-vs-529 complete sets are **not** claimed equal. Receipt:
`/tmp/worker3-o6-circle-seeded-7a-byte-audit.json`.
Rebased workspace (16 threads): **8035/8035 PASS**, nine existing skips,
`/tmp/worker3-o6-circle-workspace-7a.log`. Clippy workspace/all targets with
`-D warnings`, fmt, and wasm32 all PASS:
`/tmp/worker3-o6-circle-{clippy,fmt,wasm}-7a.log`.
The prior source-boundary receipts above remain explicitly AAA-base evidence,
not fresh 7a results.

## Official control disposition

Fresh same-round candidate and clean aaa base each ran all 623 controls:
**610 passed / 13 failed / zero skipped**, with exactly identical failed names.
Logs `/tmp/worker3-o6-circle-official-{final,base}.log`; persisted Ares outputs
506 vs 506 have identical raw SHA256 multisets, recorded in
`/tmp/worker3-o6-circle-official-byte-audit.json`.

Five of those failures are Native Slice TIMEOUT on **both** sides:

- `change_filament_flush_blocks::orca_parity_change_filament_flush_blocks`
- `bridge_over_infill_filament::orca_parity_bridge_over_infill_multi_filament_x2d`
- `narrowed_x2d_whole_file::orca_parity_narrowed_x2d_whole_file`
- `option_coverage::tree_activation::orca_parity_explicit_organic_tip_legal_boundaries_and_native_rejection_are_distinct`
- `option_coverage::tree_activation::orca_parity_organic_angle_and_normalized_grid_tip_preserve_complete_output`

These remain **INCONCLUSIVE**, never PASS or byte mismatch. The full logs retain
all failed attempt paths; the two tree inputs are verified to have the unchanged
220×220 rectangular bed, with exact input hashes and command paths in
`/tmp/worker3-o6-circle-official-tree-timeout-bed-proof.json`. The coordinator's
official gate ruling accepts the equal same-round failure set for this bounded
circle slice; seeded complete bytes remain authoritative. No timeout is used
as parity evidence, and no input is retried or altered to select a PASS.

## Stock circle/custom-bed inventory and complete-byte receipts

Both complete 1001-row scans finished, without fixture changes or retry-to-PASS.
Final-source scan: **931 raw PASS / 51 ORCA_ERROR / 9 INPUT_REJECTED /
10 VENDOR_INCOMPLETE**, **zero DIVERGENT / zero ARES_ERROR**. Nextest exit 100
is the sweep incomplete-row assertion, not a Rust regression. The earlier
v1 scan was 932/50/9/10; changing Native verdicts are preserved, not selected:
Prusa XL 5T 0.25 moved ORCA_ERROR→raw PASS, Snapmaker U1 0.8 and WonderMaker
ZR Ultra S 0.4 moved raw PASS→ORCA_ERROR. They are outside circle/custom scope.

Source-profile name/inheritance inventory identifies **43** circle/custom-bed
printers: **38** have complete normal Native output, and all 38 match Ares
**entire bytes** after excluding only `; generated by` lines. Original lengths,
raw hashes, generator-only hashes, input/source-profile hashes, paths and
binary identities are committed in `2026-10-08-stock-circle-byte-record.json`.
No M73/body normalization.

The other **five remain non-PASS**: FLSun QQ-S Pro fails Native slicing -100
because its start G-code references missing `output_filename_format`;
SeeMeCNC BOSSdelta 500 0521 four nozzle variants fail Native export -18
because `extruder_printable_height=2100` exceeds 1000. Exact Native text and
attempt paths are in the JSON; no output is invented or profile repaired.
Source inheritance has 17 unresolved areas; their separate dispositions are
retained in the JSON, not silently treated as normal comparison successes.
The persisted-input bed audit additionally covers all successful Native beds.

| Printer | Disposition | Whole-byte evidence |
| --- | --- | --- |
| stock/Anycubic/Anycubic Predator 0.4 nozzle | PASS | complete Ares/Native bytes equal |
| stock/Artillery/Artillery M1 Pro 0.2 nozzle | PASS | complete Ares/Native bytes equal |
| stock/Artillery/Artillery M1 Pro 0.6 nozzle | PASS | complete Ares/Native bytes equal |
| stock/Artillery/Artillery M1 Pro 0.8 nozzle | PASS | complete Ares/Native bytes equal |
| stock/DeltaMaker/DeltaMaker 2 0.35 nozzle | PASS | complete Ares/Native bytes equal |
| stock/DeltaMaker/DeltaMaker 2T 0.5 nozzle | PASS | complete Ares/Native bytes equal |
| stock/DeltaMaker/DeltaMaker 2XT 0.5 nozzle | PASS | complete Ares/Native bytes equal |
| stock/FLSun/FLSun Q5 0.4 nozzle | PASS | complete Ares/Native bytes equal |
| stock/FLSun/FLSun QQ-S Pro 0.4 nozzle | ORCA_ERROR | Native failure; parity_pass=false |
| stock/FLSun/FLSun S1 0.4 nozzle | PASS | complete Ares/Native bytes equal |
| stock/FLSun/FLSun Super Racer 0.4 nozzle | PASS | complete Ares/Native bytes equal |
| stock/FLSun/FLSun T1 0.4 nozzle | PASS | complete Ares/Native bytes equal |
| stock/FLSun/FLSun V400 0.4 nozzle | PASS | complete Ares/Native bytes equal |
| stock/Folgertech/Folgertech i3 0.6 nozzle | PASS | complete Ares/Native bytes equal |
| stock/RolohaunDesign/Rolohaun Delta Flyer Refit 0.4 nozzle | PASS | complete Ares/Native bytes equal |
| stock/SeeMeCNC/SeeMeCNC Artemis 0.4 nozzle | PASS | complete Ares/Native bytes equal |
| stock/SeeMeCNC/SeeMeCNC Artemis 0.5 nozzle | PASS | complete Ares/Native bytes equal |
| stock/SeeMeCNC/SeeMeCNC Artemis 0.7 nozzle | PASS | complete Ares/Native bytes equal |
| stock/SeeMeCNC/SeeMeCNC Artemis 1.0 nozzle | PASS | complete Ares/Native bytes equal |
| stock/SeeMeCNC/SeeMeCNC BOSSdelta 300 0.4 nozzle | PASS | complete Ares/Native bytes equal |
| stock/SeeMeCNC/SeeMeCNC BOSSdelta 300 0.5 nozzle | PASS | complete Ares/Native bytes equal |
| stock/SeeMeCNC/SeeMeCNC BOSSdelta 300 0.7 nozzle | PASS | complete Ares/Native bytes equal |
| stock/SeeMeCNC/SeeMeCNC BOSSdelta 300 1.0 nozzle | PASS | complete Ares/Native bytes equal |
| stock/SeeMeCNC/SeeMeCNC BOSSdelta 500 0505 0.4 nozzle | PASS | complete Ares/Native bytes equal |
| stock/SeeMeCNC/SeeMeCNC BOSSdelta 500 0505 0.5 nozzle | PASS | complete Ares/Native bytes equal |
| stock/SeeMeCNC/SeeMeCNC BOSSdelta 500 0505 0.7 nozzle | PASS | complete Ares/Native bytes equal |
| stock/SeeMeCNC/SeeMeCNC BOSSdelta 500 0505 1.0 nozzle | PASS | complete Ares/Native bytes equal |
| stock/SeeMeCNC/SeeMeCNC BOSSdelta 500 0510 0.4 nozzle | PASS | complete Ares/Native bytes equal |
| stock/SeeMeCNC/SeeMeCNC BOSSdelta 500 0510 0.5 nozzle | PASS | complete Ares/Native bytes equal |
| stock/SeeMeCNC/SeeMeCNC BOSSdelta 500 0510 0.7 nozzle | PASS | complete Ares/Native bytes equal |
| stock/SeeMeCNC/SeeMeCNC BOSSdelta 500 0510 1.0 nozzle | PASS | complete Ares/Native bytes equal |
| stock/SeeMeCNC/SeeMeCNC BOSSdelta 500 0521 0.4 nozzle | ORCA_ERROR | Native failure; parity_pass=false |
| stock/SeeMeCNC/SeeMeCNC BOSSdelta 500 0521 0.5 nozzle | ORCA_ERROR | Native failure; parity_pass=false |
| stock/SeeMeCNC/SeeMeCNC BOSSdelta 500 0521 0.7 nozzle | ORCA_ERROR | Native failure; parity_pass=false |
| stock/SeeMeCNC/SeeMeCNC BOSSdelta 500 0521 1.0 nozzle | ORCA_ERROR | Native failure; parity_pass=false |
| stock/SeeMeCNC/SeeMeCNC RostockMAX v3.2 0.4 nozzle | PASS | complete Ares/Native bytes equal |
| stock/SeeMeCNC/SeeMeCNC RostockMAX v3.2 0.5 nozzle | PASS | complete Ares/Native bytes equal |
| stock/SeeMeCNC/SeeMeCNC RostockMAX v3.2 0.7 nozzle | PASS | complete Ares/Native bytes equal |
| stock/SeeMeCNC/SeeMeCNC RostockMAX v3.2 1.0 nozzle | PASS | complete Ares/Native bytes equal |
| stock/SeeMeCNC/SeeMeCNC RostockMAX v4 0.4 nozzle | PASS | complete Ares/Native bytes equal |
| stock/SeeMeCNC/SeeMeCNC RostockMAX v4 0.5 nozzle | PASS | complete Ares/Native bytes equal |
| stock/SeeMeCNC/SeeMeCNC RostockMAX v4 0.7 nozzle | PASS | complete Ares/Native bytes equal |
| stock/SeeMeCNC/SeeMeCNC RostockMAX v4 1.0 nozzle | PASS | complete Ares/Native bytes equal |


Full receipts: `/tmp/worker3-o6-circle-stock-aaa-final-artifacts/stock-printer-summary.json`,
`/tmp/worker3-o6-circle-stock-bed-final.json`, and
`/tmp/worker3-o6-circle-stock-source-bed-inventory.json`.
The selector audit includes every persisted bed except exactly four
axis-aligned bbox corners (a conservative superset of circle/custom beds).
Generic cached-byte manifest producer fields say unknown; supplementary
cache `receipt.json` binds Native ELF SHA256
`59fb88a2cb441713c2f572cb06e30e4ab58c22a676cacca1812372b9575e73ff`.
Do not treat generic manifests alone as producer proof.

Known Native selector UB must retain its non-PASS overlay irrespective of
incidental successful byte matches. Timelapse source remains isolated and
must rebase after this independent Model slice is approved and merged.
