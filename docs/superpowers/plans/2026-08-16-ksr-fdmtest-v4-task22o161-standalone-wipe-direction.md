# Plan: Task 22O.161 standalone extrusion wipe direction

1. Add a failing KSR G-code contract for the first bottom-surface wipe endpoint and dependent spiral-lift center.
2. Reverse the retained wipe payload after standalone path emission, matching `GCode::extrude_path`.
3. Reconstruct forward storage when loop emission aggregates path payloads, matching `GCode::extrude_loop`.
4. Run focused KSR motion and travel contracts, rustfmt, Clippy, and file-size checks.
5. Record the roadmap milestone, commit, and push independently.
