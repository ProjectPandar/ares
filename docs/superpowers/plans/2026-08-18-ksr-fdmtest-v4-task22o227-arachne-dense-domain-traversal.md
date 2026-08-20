# Plan: KSR FDM Test V4 task227 Arachne dense domain traversal

1. Add a failing focused regression for insertion-order iteration and swap-with-last erase semantics across non-front removals.
2. Replace randomized `HashSet` iteration for unprocessed domain starts with a compact edge-indexed dense set; retain hash membership for passed odd edges.
3. Run focused Arachne tests, then slice the complete KSR fixture twice and require byte-identical normalized output.
4. Compare the deterministic fixture with Orca and record the next normalized divergence plus line, arc, and wipe counts.
5. Remove diagnostic tracing, run formatting, focused Clippy, and file-size checks; commit and push this source-cited slice independently.
