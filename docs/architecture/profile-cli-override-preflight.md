# Validated profile entry: CLI override preflight ordering

## Upstream boundary and Rust destination

Port the setup-order boundary of `OrcaSlicer.cpp:8228–8247 CLI::setup`, called before
project/config loading by `:1329–1332 CLI::run`, at source revision
`345efec64c7b95a05ce5cf96df0dde66a158835c`.

Native calls `m_config.validate(true)` over CLI-supplied values before loading
project and selected presets. An invalid override produces the sorted
`Params in command line error:` diagnostic, followed by `setup params error`, and
exits CLI_INVALID_PARAMS. Later merged-project validation is a different source
boundary and must not replace this first error.

The Rust destination is
`crates/ares-core/src/profiles/entry.rs::slice_validated_profile_source`, shared by
`slice_with_profiles_validated` and `slice_with_profiles_processed_validated`.
Invoke existing `ConfigOverrides::validate_cli_config` before composing/loading
the profile request, matching the bare validated adapter in
`project_slice/validated.rs`. Existing exact numeric/schema logic and error text
remain owned by `options/overrides.rs`; this slice adds no new validation policy.

This is a source-owned ordering port, not an independent pipeline feature. The
profile wrapper currently marks validation flags but omits the setup preflight;
it is a temporary incomplete adapter shell around Native's setup/run lifecycle.

## Included behavior and invariants

- Invalid caller CLI-style overrides fail at the setup stage before project load,
  preset composition, application, placement, Print validation, or processing.
- Reuse the existing override checker and exact error representation; do not map a
  later error into the desired earlier error after the fact.
- Both validated profile slicing entries share the call. Successful overrides
  continue through the same existing lifecycle.
- Pure processing (`slice_with_profiles`, `_processed`) and settings/placement
  queries are unchanged; do not make every profile API implicitly validated.
- No fixture exception, error-tolerance widening, catalog fallback, environment
  lookup, changed comparator/assertion, or registry metadata adjustment.
- No missing-catalog/selection protocol redesign or optional-preset miss change.

Deferred: remaining CLI supply protocol conflicts and synthetic-oracle argv,
replay/scarf behavior, velocity/SYNC time/motion/vector work, other registry kind
migrations, and whole-inventory acceptance. This branch starts from accepted main;
it does not rebase or stack the held refresh branch.

## Test-first public qualification

Native CLI runs first with an existing original 3MF and invalid `--bridge-flow 0`.
Require its exact early setup exit and diagnostic, and no sliced output. Supply
original vendor envelopes and a valid explicit selection to the existing public
validated profile APIs; require `SliceError::InvalidInput("bridge_flow: invalid value 0.000000")`
before later project validation. A baseline `InvalidValuesInProject` (or a later
stage failure) is a runnable behavioral RED, not a setup/compile RED.

Exercise both validated byte-output and processed-output entry points with the
same request, writing original input hash, Native producer/command receipt and
public error artifacts. Existing eleven C1 threshold assertions remain unchanged
and must be rerun when the held refresh integrates this approved ordering slice.

Failure modes to cover before implementation: setup errors hidden by composition
or model loading; byte versus processed entry divergence; valid overrides rejected;
and accidental setup validation added to pure processing. Prefer existing live
E2E controls for the latter two; do not invent isolated scalar-only mocks.

Acceptance: qualified Native-first RED then GREEN on both oracle modes, unchanged
source-owned diagnostics, rustfmt/Clippy, and a separate review request. Held-refresh
whole inventory remains a separate gate; no broader landing claim follows from a
single setup-stage witness.
