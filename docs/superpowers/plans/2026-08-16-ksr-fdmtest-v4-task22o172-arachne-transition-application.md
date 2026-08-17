# Plan: KSR FDM Test V4 task172 Arachne transition application

1. Add a focused failing cell-graph test for applying one upper transition end into a central edge.
2. Port twin normalization, endpoint sorting, 0.02 mm snapping, integer edge interpolation, and sequential `SkeletalGraph::insert_node` calls into `transitions/apply.rs`.
3. Run transition and skeletal graph tests, line-count checks, formatting, and workspace Clippy.
4. Record the source-cited slice in `docs/roadmap.md`, commit, and push independently.
