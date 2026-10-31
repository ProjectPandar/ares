# C2-T legacy validation GREEN

PLAN `7840bea1e` approved; carried to `16c6b52db` on main `512ee565e`.
Pre-repair rebase range-diff is 8/8 `=`. The proposed helper patch is applied
verbatim, confirmed by reverse-check, with no assertion changes.

## Source boundary and final production change

`PrintConfig.cpp:2079-2086,2102,2111` defines the correct eight-value global
surface domains. `PrintConfig.cpp:10401-10419` validates four global patterns
without density/role/under_cli conditions. Before validation,
`Config.cpp:927-929,577-579` loads the token through handle_legacy;
`PrintConfig.cpp:8201-8207` maps exact zig-zag to rectilinear for these keys.

The Rust `validate_pattern_option` helper now views this exact alias as the
canonical token before membership testing. Domains remain unchanged; zigzag
is still the distinct canonical enum spelling. Other inputs and diagnostics
are unchanged. Raw value preservation, typed enum/Fill behavior and broader
normalization integration remain deferred as the approved PLAN specifies.
The earlier loaded validation call and only-top-guard deletion remain;
internal-solid and numeric guard work stay in their separate slices.

## Verified gates

Official CPU13 and seeded CPU14, outer taskset, jobs2/threads2, each with a
fresh Native cache/artifact root, both 255/255 PASS, zero FAIL/TIMEOUT:

- official `/tmp/worker2-c2t-legacy-green-official.xzzw9w`: 753.608 seconds.
- seeded `/tmp/worker2-c2t-legacy-green-seeded.Orl2ZL`: 1102.683 seconds.

Each leg includes finite7, option_coverage220, bridge/anchor11 and
profile_entry loaded-config17. The previously failing unchanged
o3_matrix_sweep_loaded_config passes in both modes, including
the three Anycubic profiles; the prior STOP evidence remains preserved.
Local positive cases compare whole Native/API/CLI output, while global
negative cases retain exact Native config-error diagnostics and receipts.

Strict clippy (workspace/all-targets/all-features, -D warnings), fmt check,
and wasm32 core/vgcode/wasm check all exit0, CPU13, standard env:
`/tmp/worker2-c2t-engineering.Wu947P`.
External evidence `/tmp/worker2-c2t-legacy-green-evidence-final.json` records
all test names/counts, log and patch SHA, diagnostics and Native receipts.
No Native artifacts are committed. Code REVIEW and FULL-READY are requested;
full still requires explicit FULL-GO, and only the coordinator merges main.
