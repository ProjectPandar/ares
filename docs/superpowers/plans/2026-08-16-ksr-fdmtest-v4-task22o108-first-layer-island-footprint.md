# Plan: Task 22O.108 first-layer island footprint

1. Traverse the retained first-layer compensated slices and compute their
   translated bounds.
2. Verify the real 3MF changes the G29 placeholders from extrusion-centerline
   bounds to island-footprint bounds.
3. Run fmt, clippy, focused tests and LOC checks; commit and push before porting
   the remaining hull contributors.
