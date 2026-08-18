# Plan: KSR FDM Test V4 task204 skeletal beading recovery

1. Add a failing focused test for a node with resolved bead count but absent beading.
2. Port source beading creation and storage, then replace the junction-generation unwrap.
3. Port the bounded nearest-beading priority walk and unresolved bead-count fallback over incident central edges.
4. Re-run fixture slicing and record the next exact differential or reached assertion.
5. Run focused transitions/Arachne tests, line-count checks, formatting, and workspace Clippy.
6. Record the source-cited slice in `docs/roadmap.md`, commit, and push independently.
