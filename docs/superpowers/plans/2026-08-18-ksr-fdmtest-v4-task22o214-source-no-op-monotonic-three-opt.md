# Plan: KSR FDM Test V4 task214 source no-op monotonic three-opt

1. Add a failing `slice_project` assertion for the first KSR bottom-surface monotonic region selected after `G1 X99.635 Y137.851 E.15048`.
2. Remove the Ares-only `monotonic_three_opt` call, implementation, and transition-cost helper; retain source ant generation and path measurement unchanged.
3. Run the focused KSR motion test, slice the fixture, and record the next normalized divergence and output counts.
4. Remove the superseded task206 spec/plan, correct its roadmap record, run line-count checks, rustfmt, strict workspace Clippy, and relevant nextest checks, then commit and push independently.
