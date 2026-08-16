# Plan: Task 22o.114 internal-bridge processor role

1. Add a failing KSR project-output assertion requiring both `Bridge` and `Internal Bridge` feature tags.
2. Split the conflated Rust role mapping according to `ExtrusionEntity.cpp`.
3. Run the focused project-output test, smoke-slice the fixture, then run rustfmt and clippy before committing and pushing.
