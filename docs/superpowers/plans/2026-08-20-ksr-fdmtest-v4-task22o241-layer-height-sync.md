# Plan: Task 22O241 layer-height processor synchronization

1. Restore the focused first-perimeter assertion that rejects a duplicate nominal height tag after the layer prologue.
2. Pass nominal layer height into motion layer initialization and synchronize processor state there while retaining the upstream path-height tolerance.
3. Run focused output tests, regenerate and compare KSR structure, then run formatting, Clippy, and workspace nextest; commit and push independently.
