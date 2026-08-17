# Plan: Task 22O.149 arc-offset number formatting

1. Add a failing focused assertion for the first retained arc whose positive J offset is below one.
2. Route emitted I/J words through the existing source-style offset formatter while leaving X/Y unchanged.
3. Run the focused KSR contract, formatter tests, rustfmt, and Clippy.
4. Commit and push the slice independently.
