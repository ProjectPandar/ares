# ARD: CLI retained-output -102 error adapter

Date: 2026-10-08. Core base: 1cf47e9a7.

## Owning upstream boundary

`Utils.hpp:74` defines CLI_GCODE_PATH_IN_UNPRINTABLE_AREA (-102).
`OrcaSlicer.cpp:6209-6215` chooses/writes plate_N.gcode;
`:6218-6227` records the failure only after export and flushes with -102.
`:158` supplies the generic diagnostic. `record_exit_reson` :416-463
records sorted, tab-indented JSON; :426/:461 catch reporting failures.
`flush_and_exit` :396-413 supplies platform-specific stdout and negative
return. Already-exported G-code is not deleted.

Rust destination: ares-cli error adapter and process E2E. The existing
core processor check stays in its own upstream rewrite slice; this slice
must not reimplement geometry/checking or classify an InvalidInput string.

## Non-negotiable decisions

- Match the typed ToolpathOutsidePrintableArea error exhaustively, transfer
  its existing G-code bytes to the requested output file first, then record
  -102 and exit (Unix154). Keep the output even though slicing is rejected.
- Do not fabricate success or discard the retained stream.
- Add plate_index/layer_height/wall_loops/
  sparse_infill_density fields with the same types as SliceError::Validate.
  Populate at the core error producer from the already-applied typed config,
  using the same f32 widening semantics as Validate. No generic snapshot type,
  no reloading a project, and no parsing generated G-code for config values.
- Linux-only CLI metadata fields/assignments stay cfg-gated, with no lint
  suppression. Typed JSON string serialization propagates Result; the caller
  ignores it using let _ = like the upstream try/catch. No dynamic Value,
  unwrap, baseline/audit exemptions, or legacy placeholder for this variant.

## RED evidence and exact assertions

Fresh Native seed0 single-core capture: Unix154,
empty stderr, retained167943 bytes; input SHA256
8958ba6462946f57d1fc2d6443e6c7fac736a87be5e89dc68d2bbfc8837c1b13.
Native ELF SHA256
59fb88a2cb441713c2f572cb06e30e4ab58c22a676cacca1812372b9575e73ff.
Retained raw SHA256
87b10c65ad792167a1dad1dd664e3e5df030bd9279b3ce7f46b860b35ff5ccca.
A prior attempted capture without ORCA_REPLAY_SEED was setup-invalid
(-100/basic_string null), preserved
and excluded rather than misclassified.

`cli_retained_exit.rs` asserts Unix154, full stdout/stderr/result.json
bytes and whole retained plate_1.gcode, with only the established generator
line normalization (timestamp syntax validated). No motion/M73 normalization.
Native captures live in tests/parity/validation/cli-exit/path-outside.*.
Commit RED tests/captures separately before production edits.

## Included/deferred and exit gates

Included: typed post-export rejection and retention, native failure channels
and configuration metadata. Existing CLI error scaffolding is an adapter
around these cited upstream concepts, not an Ares pipeline design.
Deferred: -100 until this combined slice lands; success result.json and
unrelated CLI flags remain separate. Core check predicates/bits0-3 are not
changed here; commented bit4 behavior stays deferred by the owning core spec.

After GREEN: rebase on latest main, same-round fresh base/candidate
seeded/official with separate caches, workspace nextest, clippy -D warnings,
actual CLI wasm32 warning check, fmt.
Official failures/timeouts require paired byte/ND evidence, not a PASS count.
Land fast-forward only.
