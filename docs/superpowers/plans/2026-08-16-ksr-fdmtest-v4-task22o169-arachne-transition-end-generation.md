# Plan: KSR FDM Test V4 task169 Arachne transition-end generation

1. Add a focused failing test for lower and upper endpoints generated around one transition middle on a straight upward central edge.
2. Add retained transition-end storage and port the source endpoint coordinate, orientation, ordering, and payload rules into `transitions/ends.rs`.
3. Run all Arachne transition tests, formatting, and workspace Clippy.
4. Record included and deferred source behavior in `docs/roadmap.md`, commit, and push independently.
