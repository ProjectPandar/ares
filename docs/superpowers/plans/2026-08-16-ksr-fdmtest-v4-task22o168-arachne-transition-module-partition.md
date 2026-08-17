# Plan: KSR FDM Test V4 task168 Arachne transition module partition

1. Move transition-middle filtering and its recursive helpers into `transitions/filtering.rs`, retaining shared search/reference types at the parent seam.
2. Expose only the sibling visibility required by focused tests and later transition orchestration.
3. Run all Arachne transition tests, line-count checks, formatting, and workspace Clippy.
4. Record the partition in `docs/roadmap.md`, commit, and push independently.
