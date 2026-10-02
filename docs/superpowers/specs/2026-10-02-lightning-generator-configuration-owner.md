# Lightning generator configuration owner rewrite

## Source and destination

Rewrite `PrintObject.cpp1010-1020::prepare_lightning_infill_data` and
`Fill/Lightning/Generator.cpp66-100::Generator` configuration operands in
`ares-core/src/project_slice/prepare_infill/bridge_over_infill/transaction/candidate_expansion.rs::build_lightning_generator`.
Preserve `PrintObject.cpp2494-2498`'s separate pattern-only bridge flag.

## Included behavior

Any positive-density Lightning printing region activates the generator, while
the first global region supplies width, density, multiline and angle options.
Changing an empty later Lightning region's width cannot change generated tree
polylines when global first configuration and geometry remain unchanged.
Zero-density Lightning alone must not activate a generator. Existing generator
arithmetic and all passed output behavior remain unchanged.

## Deferred behavior

All-region geometry/expansion and multi-range shared-region ordering require
their own ports and output evidence. Public modifier parity, all printers and
options, and Normal/Tree completion are not claimed by this repair. The existing
single-region generator shell is temporary for those unported operands.

## Acceptance

Two behavioral REDs on clean HEAD through the existing bridge transaction
entry; both GREEN after only the consumed production configuration change.
No visibility widening, private source/structure pins, fixture weakening,
unsupported-feature guard, fallback, or Normal/Tree dependency. Full nextest,
fmt/Clippy, relevant Orca complete-output controls, Tier1 compilation and
independent six-axis review must verify the standalone submission closure.
