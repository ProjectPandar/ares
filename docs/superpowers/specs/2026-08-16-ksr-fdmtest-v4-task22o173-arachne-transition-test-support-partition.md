# Spec: KSR FDM Test V4 task173 Arachne transition test-support partition

## Observable contract

No slicing or test behavior changes. Shared Arachne transition strategy, configuration, and central-chain fixtures move from the near-limit parent test file into the real Rust module `transitions/test_support.rs`. Parent transition tests and specialized endpoint/application test modules consume one shared fixture interface; no `include!` or generated source composition is used.

All transition tests produce unchanged results. Every affected Rust source remains below 400 lines. Workspace formatting and Clippy remain clean.
