# Per-object draft shield pure inputs

SPEC: `docs/architecture/per-object-draft-shield-spec.md`.
`manifest.json` records original discovery provenance. Twelve 3MFs are pure inputs; no Native output committed.

Each original disjoint/overlap/transitive/copies geometry has disabled/all/single
variants changing only draft_shield and single_loop_draft_shield. Min0, no brim,
one filament, by-layer, loops2, skirt_height1; objects have20 physical layers.
Native discovery accepted24/24 exports: disabled skirts onlyZ0.2, enabled both
loop modes at20 Zs0.2..4.0, group counts2/1/1/1 at each eligible Z. First-layer
all/single motion is identical; later single-loop motion has each group's
innermost envelope and less extrusion. Counts/bounds do not substitute for
strict stream parity. Public success tests first preserve Native per-Z real
motion, then require API/CLI success, existing strict full ordered Native
comparisons and raw API/CLI equality. Rejection is never expected as success.

Selection: `test(orca_parity_per_object_draft_shield_)`, required Native
producer environment, fresh caches/artifacts and explicit target.
Baseline public RED should be4disabledPASS+8enabledguardFAIL each mode;
fixtures/assertions are fixed for GREEN.
Artifact includes input/hash/launch/result/fullNative G-code/per-Z witness,
API/CLI output or errors and comparison. No stream normalization.
