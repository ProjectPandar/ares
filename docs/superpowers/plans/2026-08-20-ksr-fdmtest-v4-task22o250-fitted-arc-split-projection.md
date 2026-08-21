# Plan: Task 22o250 fitted-arc split projection arithmetic

1. Extend the focused KSR G-code test with the expected first wipe move and run it red against the current rounded endpoint.
2. Replace fitted-circle projection with source-order integral center/delta arithmetic: normalize in scaled coordinates, truncate each radius-vector component, then add the center.
3. Remove diagnosis-only traces and verify the focused seam/wipe test advances the normalized fixture comparison.
4. Run rustfmt and clippy for the affected crate, then commit and push this source-cited slice.
