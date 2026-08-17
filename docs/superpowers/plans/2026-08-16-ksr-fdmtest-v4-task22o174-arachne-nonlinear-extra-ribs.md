# Plan: KSR FDM Test V4 task174 Arachne nonlinear extra ribs

1. Share the existing central-cell graph fixture and add a focused failing test for one strategy-derived nonlinear radius crossing.
2. Port upward-edge gating, thickness/radius filtering, integer position interpolation, endpoint snapping, and sequential skeletal insertion into `transitions/ribs.rs`.
3. Run transition and skeletal graph tests, line-count checks, formatting, and workspace Clippy.
4. Record the source-cited slice in `docs/roadmap.md`, commit, and push independently.
