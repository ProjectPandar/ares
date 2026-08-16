# Plan: Task 22o.118 selected conditional template whitespace

1. Add a failing KSR output assertion for the exact selected nested fan-control whitespace.
2. Preserve the closing directive blank for selected single-branch conditionals only.
3. Extend focused renderer tests for selected and unselected single branches.
4. Run renderer and KSR layer-change tests, smoke-slice the 3MF, then run rustfmt and clippy before committing and pushing.
