# Seeded oracle identity assertion

## Scope

Ares test infrastructure, no `libslic3r` boundary. This record fixes what
`crates/ares-cli/tests/orca_parity/runner/initialized_loop_chain.rs` asserts about the
executable that produces its seeded reference, and why the assertion is neither a launcher
path pin nor removable.

## Problem

The test asserted

```rust
assert_eq!(runner.bin(), &repo_root().join("scripts/orca-seeded-parity.sh").canonicalize().unwrap());
```

`runner.bin()` is the canonicalized `ARES_ORCA_BIN` (`runner/environment.rs:62-78`). That
equality holds only when the runner binds `ARES_ORCA_BIN` to the in-repo wrapper. The batch
runner binds it to the installed seed launcher
(`gcroots/orca-seed-launcher-250`, a symlink to
`/nix/store/…-orca-seed-launcher-250.sh`), so the assertion fails for reasons unrelated to
slicing. Dropping it would remove the only check that the reference comes from the seed
replay rather than the official release, which is exactly the check worth keeping: a seeded
comparison run against the official binary would silently compare Ares with the wrong oracle.

Both launchers accept the same contract and end in the same exec, so the identity has to be
asserted on what the launcher does and on the binary it wraps, not on its path:

```sh
exec taskset -c "$ORCA_REPLAY_CPU" setarch -R "$ORCA_REPLAY_BIN" "$@"
```

## Assertion

`assert_seeded_oracle` requires all three, in this order:

1. **The launcher runs the replay binary.** The launcher file must contain
   `setarch -R "$ORCA_REPLAY_BIN"`, so a direct binding to a slicer binary, to the official
   launcher, or to any wrapper that does not route through `ORCA_REPLAY_BIN` is rejected.
2. **The launcher is the binding the environment declares.** When `ARES_ORCA_SEEDED_BIN` is
   set, as the batch runner does, `runner.bin()` must equal its canonicalized value. When it
   is unset, as in the local seeded environment, the check is skipped and step 1 carries the
   identity.
3. **The native binary the test compares against is the seed replay.** The SHA-256 of
   `ARES_ORCA_NATIVE_BIN` must equal `ARES_ORCA_SEEDED_NATIVE_SHA256` when the environment
   pins it, else the pinned constant
   `84a7780653be3b46fe909e43ae98b3576907c561e8ff337c208ff97c2b23a84a`.

Included: the seeded lane only; the test remains seeded-only, and the official lane keeps
returning early. Deferred: no change to `OrcaRunner`, to the environment reader, or to any
other seeded test that pins the in-repo launcher (`stock_project_native_stream/producer.rs`,
`option_coverage/safc_reference.rs`) — those select their own bindings and are out of scope
here.
