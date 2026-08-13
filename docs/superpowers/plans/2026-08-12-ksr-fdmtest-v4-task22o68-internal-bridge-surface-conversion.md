# Task 22O.68 implementation plan

1. Cite pinned `PrintObject.cpp:3352-3367` and `Surface.hpp` vocabulary; freeze
   included/deferred behavior in the ADR and specification.
2. Add `InternalBridge = 6`, exhaustive bridge classification, and an ordinary
   private operation/test module, keeping every Rust file below 400 LOC.
3. Preserve a behavioral RED for source-index matching, metadata, angle, union,
   and ordering before implementing the production seam.
4. Implement exact region/index/kind selection, one default-NonZero union per
   accepted candidate, source metadata cloning, retagging, angle replacement,
   ordered owned output, and first-error propagation.
5. Add focused topology/order/empty/error/nonmutation discriminators and kill
   all compiling mutations with byte-exact restoration.
6. Run focused and dependency Nextest, workspace Nextest, strict Clippy/rustfmt,
   wasm32, x86_64/aarch64 Windows and macOS, diff/LOC/static/include,
   pinned-Orca, and no-staged gates.
7. Start an independent read-only six-axis review, return its repair list to the
   main thread, fix only there, and re-review until unconditional approval.

## Completion evidence

Steps 1-6 pass: focused 6/6, dependency 788/788, workspace 6,448/6,448,
strict/portability/static gates, and 14/14 compiling mutation kills. Production
restored SHA-256 is
`d8f2e21dccc653c867bbaf5950061a264589a5b2f007b4b373686e2d2e21290b`.
Step 7 passed: independent six-axis review approved without repairs.
