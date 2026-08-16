# Plan: Task 22o.119 layer cooling fan transition

1. Add a failing KSR assertion for the first non-zero part and auxiliary cooling commands.
2. Read typed fan options from the resolved 3MF configuration and retain emitted fan state.
3. Emit deduplicated Orca-compatible PWM commands before the affected layer block.
4. Run the focused nextest, rustfmt, and clippy; commit and push the slice.
