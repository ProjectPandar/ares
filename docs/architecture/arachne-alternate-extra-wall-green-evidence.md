# Arachne alternate extra wall: bounded public GREEN

Fixtures/Native RED 8989e26eb were committed
before production. The one obsolete rejection test was separately retired in
76b8fbe43; its commit message cites
`PerimeterGenerator.cpp:2131–2134` and RED8989e26eb. Every other test body and
assertion is byte-identical; only the unused import had rustfmt line wrapping.

## Production boundary

`arachne/config.rs` deletes only the alternate rejection. `materialize.rs`
uses the existing record's layer/spiral context and region settings to compute
the upstream odd/non-spiral/positive-integer-density predicate once, then passes
configured wall count plus its increment into the existing per-surface caller.
`walls.rs` still adds surface extra perimeters, applies bottom/top overrides,
captures inner_loop_number and invokes the existing Arachne generation and
top-surface regeneration. Moving the additive count into the caller preserves
the arithmetic and ordering without changing this private parameter interface.
This uses the materialize context seam permitted by the SPEC's spiral predicate
boundary. No new guard, Classic fallback, independent wall algorithm or first
region shortcut was introduced. Fuzzy skin and overhang remain separate slices.

## Finite public GREEN

Official10/10 PASS, 12.002 seconds; seeded10/10 PASS, 11.744 seconds; exit0 both.
Each paired test checks both inputs, so all40 Native/API/CLI cases pass complete
strict generator-stream comparison with independent Native wall witnesses.
All40 API/CLI raw output pairs are identical. All48 previously admitted off/inert
control streams are byte-identical RED→GREEN. Test/input payload SHA256s equal
the RED payload; no E2E assertion, fixture or comparator changed after RED.
Commands, input/binary/output hashes and raw case output stay outside Git.

The run began with parent RED8989e26eb plus the working production patch;
retirement76b8fbe43 landed separately while the same production bytes ran.
The source hashes bind the finite result to the later GREEN commit; do not
mislabel the parent hash as the committed production candidate tested.

## Remaining gates

Workspace/all-targets/all-features strict clippy and fmt exit 0; WASM32
core/adapter check passes. Review this feature separately before subsequent
fuzzy skin or overhang implementation.
