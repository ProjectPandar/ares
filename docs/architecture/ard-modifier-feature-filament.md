# ARD: modifier feature filament overrides use upstream region config

Status: accepted. SPEC a2ffbb77b; public RED with per-role witnesses.
Associated [SPEC](modifier-feature-filament-spec.md). No production/guard change
in this RED. Whole strict public control passed in both modes before extension;
all six individual roles now have their own Native local-T1 qualification.

## Owner and non-negotiable boundary

`PrintObject.cpp:3592-3705` owns feature-explicit masks, inherited modifier
parent and positive base-extruder fallbacks. Explicit positive sparse ID2 remains
2 even when modifier extruder1; clear bits fall back to1. Modifiers start clear
masks, not a copy of parent explicit bits. Existing Rust RegionBase::Modifier /
FeatureFilamentOverrideMask already expresses this concept. Do not design a new
modifier material pipeline or reinterpret inheritance to bypass an admission gap.

`PrintApply.cpp:1032-1055` constructs parent-linked modifier regions; its
`:810-830` / `:1060-1075` painting paths later override all six role IDs, not the
base modifier extruder policy. Painting behavior is deferred, not reassigned.
`PrintRegion.cpp:13-15/81-86` selects/collects sparse role filament2 as tool1.

The exact admission boundary is `project/effective_config/candidates.rs`'s
six-key `first_unsupported_modifier_key` guard. `project/region_graph.rs` and
`options/region_options/merge.rs` remain the source-cited destinations around it.
Only a separate GREEN can retire any of those six branches;
unrelated guards remain untouched. No resolver rewrite is justified by a guard
alone. Each branch may retire only with its own public RED. The initial
sparse witness is now extended by five independently Native-qualified role cases;
no unqualified role is admitted by this matrix. Keep any unqualified guard branch.
This remains neither an all-combination parity claim nor nonzero-extruder proof.

## Evidence and rejected shortcuts

Frozen control/positive share accepted modifier geometry plus complete donor
two-filament settings; only modifier sparse_infill_filament_id=2 differs. Four
Native legs accept, diverge, and positive has780 spatially local T1 sparse
segments in each mode. An initial comment-dialect observer defect was corrected
offline on the same G-code, not by rerunning Native or changing the input.

Public control must independently pass whole strict Native/API/CLI and raw
API==CLI. Positive must fail target equality before GREEN, not pass an error
expectation. No missing-stream equality, Native-only bypass, M73 filtering or
downstream repair. Reject role-only diagnostic equality as a substitute for the
complete comparator. Nonzero modifier-extruder/material/painting/overlap matrices
remain explicitly deferred runtime evidence, with source precedence preserved.
