# Plan: Task 22O.152 retained-polyline fitted-arc wipe

1. Add a failing KSR assertion for all three wipe moves after the fitted first-layer outer wall and the resulting spiral-lift center.
2. Keep arc fitting solely as the emitted extrusion-command representation; retain the complete clipped source polyline as `EmitState::wipe_path`.
3. Reuse the existing option-driven wipe-distance clipping and proportional retraction path.
4. Run focused KSR and G-code motion contracts, generate the CLI slice, and identify the next exact divergence.
5. Run rustfmt and Clippy, record the roadmap milestone, then commit and push independently.
