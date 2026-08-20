# Plan: Task 22O239 persistent extrusion-height processor state

1. Add a focused KSR assertion for the complete current `; LAYER_HEIGHT:` count and confirm the layer reset duplicates path-height transitions.
2. Remove the nominal layer-height assignment from motion layer initialization, use the upstream height-comparison tolerance, and remove the now-unused argument from its only caller.
3. Run the focused assertion, regenerate and structurally compare KSR G-code, then run formatting, Clippy, and workspace nextest; commit and push independently.
