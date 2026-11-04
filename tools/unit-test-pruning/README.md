# Unit-test pruning evidence (E2E subsumption)

`e2e-coverage-summary.json`: per-production-file E2E-only line coverage
(`covered` line count + `uncoveredRanges`), generated with `cargo llvm-cov`
over `orca_parity` (seeded replay, `ORCA_REPLAY_SEED=0`) and `ksr_fdmtest_v4`.
Toolchain: rustc 1.99.0 (LLVM 23.1.1); llvm-cov/llvm-profdata LLVM 23.1.0-rc1
(nix `llvmPackages_23.llvm`).

Regenerate with `collect-e2e-coverage.sh` then
`cargo llvm-cov report --json --output-path <out>/cov-e2e.json`.

Per-test subsets: `cargo llvm-cov nextest` writes one `LLVM_PROFILE_FILE`
per test process; merging each profraw and comparing its executed line set
against this summary decides which unit tests are subsumed by E2E.
