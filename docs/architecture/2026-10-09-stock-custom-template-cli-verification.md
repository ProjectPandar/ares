# Five stock custom-template CLI rejections: verification

Approved spec: `02ffadb9c`; coordinator split this first five-case slice from
four Kobra3 eligibility (-50) and four MK3S postprocess (-19) cases. Those eight
remain a separate follow-up awaiting worker4's merged public EarlyExit recipe.
No new rejection guard or production change was necessary.

## Source boundary and inputs

FLSun QQ-S Pro0.4 and Ginger G1 1.2/3.0/5.0/8.0 retain their stock custom
G-code. PlaceholderParser.cpp:931/999 owns the missing/invalid variable detail;
GCode.cpp:3636-3645 owns the ordered failed-template aggregate; OrcaSlicer.cpp:
6286-6291 prints the exception and records CLI_SLICING_ERROR (-100), with POSIX
exit156. Existing Rust parser/emission and CLI error transport already match.

`orca_parity/residual_stock_cli.rs` uses the stock sweep's unchanged vendor
catalog, select_printer, machine/process/filament selection, empty overrides,
cube10.stl and existing runner.export_case recipe/cache. Native exports the3MF
at runtime. Public exported bytes are written only to the external artifact
directory, checked unchanged, then the same file feeds custom_gcode_reference
and the public Ares CLI. No Native-generated inputs or products are committed.

The #97 custom_gcode_reference producer/cache receipt and cache-hit checks are
reused, not rewritten. Complete stdout/stderr compare without normalization;
complete result bytes use only shared native_wall_clock::result_comparison_bytes
for prepare_time/export_time. All other bytes remain exact. No final G-code
is exported. Each runtime raw input SHA and the historical stock input SHA are
recorded separately; historical exported archive bytes are not a runtime golden.
Missing optional Native prints visible SKIPPED before artifact-root access;
required absence fails closed through the existing environment/runner guard.

## Actual acceptance executions

All final gates executed at test-only
`304f49867b22dfbe18a50bd7c34a8fb474b2062d`, based on
`da16b918566efcd13019a312022f675fe331badf`. Sixteen test threads/build jobs8;
one heavy job at a time. Required runs used the then-current
`${TMP_ROOT}/ares-parity-control-set-v2.sh` and separate fresh producer caches.

| Gate | Actual result |
|---|---|
| clippy --workspace --all-targets --all-features -- -D warnings | exit0 |
| cargo fmt --all -- --check | exit0 |
| default workspace, every ARES_* removed | ab308d3e-e77f-4f69-b123-006a4450e1f2;8193PASS/6existing skips;exit0 |
| required seeded five cases | 00dc5ca9-b227-40c8-94ea-d21d34829f7a;5PASS;exit0 |
| required official five cases | 7509c4f8-64bf-4348-8996-7c601a96e589;5PASS;exit0 |

Default output explicitly contains five SKIPPED Native-comparison notices for
these tests, with no panic. Those early-return test successes are not Native
parity evidence. Both required producers independently return156 and match
Ares's full channels for all five cases. Receipts, input hashes, actual Native
command statuses, stdout/stderr captures and product inventories were reread
for all ten records. Result-byte parity is enforced by the unchanged shared
comparator assertions, not a new JSON semantic comparison.

The earlier81d96a0e2 seeded execution a91145f2-79e2-473e-8d78-f74b6b540f55
also passed5; it is not relabeled as execution on the final commit. Compilation
failures v2(E0583), v3(E0616), the input-storage correction interruption and
superseded narrower lint interruption remain external, with no GREEN claim.
Compilation fixes were squashed into the single test commit before final gates;
none of those failed intermediate commits or the prohibited binary-input
attempt is an ancestor of the merge candidate.

## Conclusion and evidence

No production RED was observed: this is tests/docs-only rejection-channel
closure, not successful-stream PASS promotion of the stock ORCA_ERROR rows.
The12 Native SIGSEGV/SIGABRT cases remain record-only as listed in the stock
audit; Ares does not imitate crashes or add guards. No worker4-owned cache or
cli_exit_channels file was edited; no roadmap history was restored.

Machine review record: `/tmp/worker3-residual-cli-final-review.json`.
Commands/environments/exit/log receipts and external artifacts:
`/tmp/worker3-residual-cli-final-*`. Older attempts retain their distinct
`/tmp/worker3-residual-cli-runtime-v*` bindings. This conclusion is documentation
added after the actual acceptance executions, not another runtime execution.
