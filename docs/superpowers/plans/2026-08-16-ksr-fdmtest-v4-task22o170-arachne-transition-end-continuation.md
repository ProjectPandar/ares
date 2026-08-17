# Plan: KSR FDM Test V4 task170 Arachne transition-end continuation

1. Add a focused failing test in `transitions/ends/tests.rs` for a half-transition crossing from one central edge onto the next.
2. Port recursive central-edge traversal, remaining-distance conversion, and joint transition-state updates using a compact search value.
3. Run all Arachne transition tests, line-count checks, formatting, and workspace Clippy.
4. Record the source-cited slice in `docs/roadmap.md`, commit, and push independently.
