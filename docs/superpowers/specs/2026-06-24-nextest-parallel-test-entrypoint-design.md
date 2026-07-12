# Nextest Parallel Test Entrypoint Design

## Scope

Update the repository's executable test-entrypoint coverage so test commands are guarded as parallel `cargo nextest run` invocations. This responds to the request to modify tests to use nextest for parallel execution.

The executable boundary is:

- `scripts/test.sh`
- `.cargo/config.toml`
- `.config/nextest.toml`
- `crates/ares-cli/tests/test_script.rs`

Historical `docs/superpowers/**`, `docs/milestones/**`, and roadmap entries are archival records of earlier work and are out of scope for this slice.

## Current Behavior

`scripts/test.sh` already dispatches no-argument runs as `cargo nextest run --workspace` and argument runs as `cargo nextest run "$@"`. `.cargo/config.toml` already exposes `cargo xtest` as `cargo nextest run`. `.config/nextest.toml` already sets `[profile.default] test-threads = "num-cpus"`.

The current tests verify the shell script basics, but they do not yet explicitly protect the Cargo alias as a nextest entrypoint. They also must continue protecting executable entrypoints or nextest command construction from adding serial overrides such as `--test-threads 1` or `RUST_TEST_THREADS=1`.

## Required Behavior

- `scripts/test.sh` remains the default workspace test command and uses `cargo nextest run --workspace` when no arguments are provided.
- `scripts/test.sh` continues to forward provided arguments through `cargo nextest run`.
- `.cargo/config.toml` keeps `xtest = "nextest run"` so local `cargo xtest` invocations use nextest instead of `cargo test`.
- Tests must assert executable test entrypoints do not use `cargo test`.
- Tests must assert executable test entrypoints do not set Rust's serial test environment knobs, including `RUST_TEST_THREADS`.
- Tests must assert the scripted nextest command line does not pass `--test-threads` or `-j`, so `.config/nextest.toml` remains the single source of concurrency policy.
- Tests must assert the nextest default profile contains `test-threads = "num-cpus"` and does not contain known serial overrides.
- Tests must assert the Cargo alias table preserves the `xtest` nextest alias and does not introduce any alias entry named `test`.

## Non-Goals

- Do not rewrite historical specs, plans, milestone documents, or archived OMX artifacts.
- Do not add a new CI system, dependency, test framework, or Make/Just wrapper.
- Do not change actual slicing or G-code behavior.
- Do not force a fixed numeric parallelism value; use nextest's `num-cpus` profile setting.

## Acceptance Criteria

- A focused nextest run for the CLI test-script coverage passes:
  `cargo nextest run -p ares-cli test_script`
- Full workspace tests run through nextest:
  `cargo nextest run --workspace`
- Standard verification passes:
  `cargo fmt --check`
  `cargo clippy --workspace --all-targets -- -D warnings`
  `cargo check -p ares-core --target wasm32-unknown-unknown`
  `git diff --check`
- Touched Rust files stay at or below 400 LOC.

## External Reference

Nextest configuration supports `profile.<name>.test-threads = "num-cpus"` and allows `--test-threads` to override profile concurrency. This slice treats the checked-in profile as the concurrency source and verifies the script does not override it.
