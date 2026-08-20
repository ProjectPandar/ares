# Plan: Task 22O242 rounded fitted-circle centers

1. Change the focused fitted-circle boundary assertion from toward-zero truncation to the upstream nearest-integer result and confirm it fails.
2. Round fitted center coordinates on the scaled grid while retaining truncation for perpendicular-point assignment.
3. Run focused arc and KSR output checks, regenerate the fixture output, then run formatting, Clippy, and workspace nextest; commit and push independently.
