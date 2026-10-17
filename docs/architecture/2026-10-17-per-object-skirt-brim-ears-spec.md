# Per-object skirt BrimEars admission SPEC

## Boundary and witness

This slice is based on the approved Painted admission `4c778efa54a55e333ad7c84b27ac4a934348e88d` (stack-24 source boundary `4351a644d`). The frozen probe input is `/tmp/w2-brim-next-brim_ears.3mf`: two separated objects, `skirt_type=perobject`, `brim_type=brim_ears`, `combine_brims=0`, one filament and one instance per object, ByLayer, no tower/support/raft, with the retained `Metadata/brim_ear_points.txt`.

Native official rc0 evidence is `/tmp/w2-next-native-off-brim_ears-cQwH`; seeded rc0 is `/tmp/w2-next-native-seed-brim_ears-bEKg`. Native emits two Brim blocks (lines 170 and 999) and four Skirt blocks (74, 902, 1740, 1935), proving active brim geometry and per-object grouping. Current Ares rejects the same input at the per-object UnsupportedProjectFeature admission. A temporary match-only BrimEars admission produced `/tmp/w2-next-ares-brim_ears-6uvG`; its normalized 5302-line output is byte-equivalent to Native in both modes. The slice is therefore admission-only.

Auto was probed separately and deferred because Native produced no Brim block.

## Upstream and destination

`Print.cpp::_make_skirt:2780-2977` and footprint grouping `2847-2971` consume each object's computed brim footprint. BrimEars geometry comes from `Brim.cpp:354-412` (`make_brim_ears`) and its dispatch/flow path `537-551`; shared support/no-brim masking is `590-634`.

The exact Ares boundary is `crates/ares-core/src/project_slice/gcode_emit/skirt/generate.rs:48-56`; `skirt/occupied.rs:61-80` already consumes `BrimPlan::covered_hull()`. The existing guard is an unsupported-feature admission branch, not defensive validation.

## Included and RED plan

GREEN adds only `ProcessBrimType::BrimEars` to the existing admitted match. The combine-brims, instance-count, and tower/flush conditions remain unchanged; Auto and Painted are separate slices.

The public RED will commit the frozen input and assert Native rc0, at least two Skirt blocks, and at least one Brim block before unchanged strict API and CLI whole-output comparison. Official CPU13 and seeded CPU14 use fresh roots. The target must fail at the current guard while the existing controls pass; no Native artifacts are committed.
