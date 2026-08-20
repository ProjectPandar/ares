# Plan: Task 22O238 nearest seam-visibility ray hit

1. Add a focused BVH regression with a near flat triangle and a farther sloped triangle whose bounding box overlaps the near hit; confirm the farther right subtree incorrectly replaces the nearest hit.
2. Reject leaf intersections at or beyond the recursive nearest-distance limit.
3. Run the BVH regression, KSR slice comparison, formatting, Clippy, and workspace nextest; commit and push independently.
