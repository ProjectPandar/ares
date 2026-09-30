# Spec: Obsolete silent-mode input evidence

Follow ARD-0052's PrintConfig handle_legacy/default boundaries and existing
typed_legacy Rust destination. Both generated Boolean inputs must be retained
in proof JSON, while effective `silent_mode` must be the canonical false.
Include explicit obsolete-normalization provenance, not a feature-on claim.

Acceptance: offline proof tests accept both requests only against effective
false, reject wrong/missing effective values and preserve unrelated checks;
two direct original full-output comparisons plus the ordinary Boolean sweep
pass. Reference config must report false and no silent-mode progress stream.
No production slicing change, normalization of G-code, or reduction in values.
