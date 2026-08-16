# Plan: KSR FDM v4 task22o110 retract, wipe, and spiral lift

1. Add failing focused assertions for effective retract/wipe/lift options and the first inter-path wipe/spiral-lift block.
2. Resolve typed printer, filament override, process, and project options into the private motion options module.
3. Track the prior emitted path, retract backward over its configured wipe distance, and emit relative-E wipe moves without fixture constants.
4. Port minimum-travel and `reduce_infill_retraction` gating using final internal layer surfaces and Clipper open-path difference.
5. Port configured spiral z-hop, raised-Z travel, lowering, and deretraction; split role and travel logic into normal modules so every Rust file remains below 400 LOC.
6. Run focused tests, generate the complete KSR output to record the next structural divergence, run formatter/clippy checks, then commit and push this vertical slice.
