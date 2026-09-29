# Option coverage completion gate

## Scope

The existing option sweep can exit successfully with INCOMPLETE domains.
Its reason-string exemptions include missing support generation, random
oracle output, authored interior probes, unbounded domains, and rejected
required values. That exit status does not prove the requested coverage.

This is a correction to the acceptance harness for the source-cited rewrite,
not a new slicing pipeline or a declaration of upstream parity.
The upstream owners remain `libslic3r/PrintConfig.cpp` option definitions
and validation, `Support/SupportMaterial.cpp` and `Support/TreeSupport.cpp`
support generation, and `FuzzySkin.cpp` random displacement.
The Rust destination is the existing CLI test harness:
`tests/orca_parity/option_coverage/sweep.rs`.

## Required behavior

- Only a nonempty set of PASS outcomes completes a requested sweep.
- INCOMPLETE, FAIL, UNBOUNDED, and an empty selection fail acceptance.
- Human-readable diagnostic reasons never grant an exemption.
- Write the summary and preserve paired output artifacts before failing.
- A filtered sweep proves only its selected domains, not the full inventory.

## Boundaries

Keep the existing reason classifications for diagnostics. Do not alter the
byte comparator, generated values, or reference output to make tests pass.
Missing upstream support behavior and stochastic oracle reproducibility are
deferred implementation work, and continue to prevent full completion.
Existing Ares scaffolding remains temporary wherever the cited upstream
behavior is absent; harness success cannot substitute for its rewrite.

## Acceptance

Regression tests reject each previously exempted reason and empty input.
A fresh real-Orca passing boolean sweep succeeds; a value-filtered sweep
writes its paired artifacts and INCOMPLETE report, then exits unsuccessfully.
Run formatting, Clippy, Nextest, and independent six-aspect review.
