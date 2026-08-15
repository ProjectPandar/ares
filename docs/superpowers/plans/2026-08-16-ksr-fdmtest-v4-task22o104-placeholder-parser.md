# Task 22O.104 implementation plan

1. Add a small typed `Value`/configuration view over the resolved config block
   and derived project variables.
2. Port the expression tokenizer/parser and the nested conditional renderer
   into separate modules, with explicit syntax/runtime errors.
3. Add focused tests for nested branches, vectors/indexing, arithmetic,
   boolean expressions, functions, unresolved values, and malformed blocks.
4. Integrate rendering into project machine-start emission without changing
   legacy `SliceOptions` paths.
5. Run focused tests, strict workspace Clippy, formatting, LOC/macro/diff
   audits, and the ignored KSR golden; record the first remaining difference.
6. Launch an independent read-only six-axis reviewer, fix its findings, rerun
   review, then commit and push.
