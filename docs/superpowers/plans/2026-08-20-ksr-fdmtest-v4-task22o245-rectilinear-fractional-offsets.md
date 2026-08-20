# Plan: Task 22O.245 rectilinear fractional offsets

1. Compare the next non-timing golden divergence after O244 and add its complete-project assertion.
2. Bisect the first geometry-changing commit and verify the upstream `scale_` macro and `float` conversion.
3. Preserve the fractional scaled value instead of truncating it through `i64`.
4. Run the focused project and rectilinear tests; generate a fresh KSR slice and record the next divergence.
5. Commit and push this source-cited slice independently.
