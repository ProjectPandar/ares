# Spec: KSR FDM Test V4 task168 Arachne transition module partition

## Observable contract

No slicing behavior changes. Arachne transition-middle generation remains in `trapezoidation/transitions.rs`; transition-middle filtering moves to the real Rust submodule `trapezoidation/transitions/filtering.rs`. Production and test files remain below 400 lines without `include!` or generated source composition.

## Source boundary

The partition follows the existing OrcaSlicer 2.4.2 boundary between `SkeletalTrapezoidation::generateTransitionMids()` and `filterTransitionMids()` plus its recursive helpers in `Arachne/SkeletalTrapezoidation.cpp:784-1058`. It changes only the Ares Rust module layout required before porting transition-end generation.

## Acceptance

All Arachne transition tests produce unchanged results. Both production files and the transition test module are below 400 lines. Workspace formatting and Clippy remain clean.
