# Plan: Task 220.140 remove simplification checkpoint pinning

1. Remove the direct simplification internals tests and fixture checkpoint/digest suite from the test module tree.
2. Delete their mutation and checkpoint helpers while retaining production simplification and downstream project slicing tests.
3. Run the complete `ares-core` test suite, rustfmt, strict clippy, and LOC checks; commit and push the cleanup.
