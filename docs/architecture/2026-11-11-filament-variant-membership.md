# Complete the upstream filament variant owning list

## Source boundary

This slice ports three omitted members of
`libslic3r/PrintConfig.cpp::filament_options_with_variant` at upstream
`345efec64c7b95a05ce5cf96df0dde66a158835c`:

| Member | Individual upstream citation |
| --- | --- |
| `filament_retract_length_toolchange` | `PrintConfig.cpp:10171` |
| `filament_retract_restart_extra_toolchange` | `PrintConfig.cpp:10172` |
| `filament_minimal_purge_on_wipe_tower` | `PrintConfig.cpp:10210` |

The Rust destination is only
`crates/ares-core/src/options/registry/key_lists.rs::FILAMENT_OPTIONS_WITH_VARIANT_KEYS`.
The source set has63 active members; Rust currently has60, with exactly these
three missing and no extra or duplicate members. The commented
`filament_extruder_id` entry is not a member. The itemwise audit is retained in
`approved-residuals-r1/VARIANT-MEMBER-AUDIT.json` under the nozzle evidence root.

This is a bounded rewrite correction, not a new Ares configuration pipeline.
The existing raw normalizer, refresh adapter, and typed selectors remain the
compatibility implementation of the cited upstream operations.

## Non-negotiable ownership

Membership owns behavior, not merely registry discoverability:

- `PrintConfig.cpp:10094–10096,11431–11457` normalizes a per-filament or
  singleton value into the variant slots described by `filament_self_index`.
  For `[1,1,1]`, a singleton is copied to all three slots, preserving its value
  and nullable type. Already expanded vectors are not replaced by a new policy.
- `PrintConfig.cpp:10600–10624` chooses filament-variant cardinality for members
  of this set, rather than ordinary extruder cardinality.
- `PresetBundle.cpp:281,306,5590,5685,6342` uses the same ownership for selected
  configuration mapping and preset/variant refresh.

The existing Rust consumers remain unchanged:
`project/raw_settings/filament_variants.rs`,
`profiles/project_refresh/filament.rs::normalize_variants`,
`profiles/project_refresh/filament/slot.rs::apply`, and the existing
`options/project_variants` selection and export paths. Add only the three
source-owned names to their existing shared authority. Do not change option
kinds, defaults, nullable semantics, normalizer algorithms, slot restoration,
exporters, truncation, tower behavior, or adjacent owning lists.

## Why ordinary output is insufficient for the two toolchange members

The retained system H2D primary has three variants for one filament. Its Native
complete output contains `filament_minimal_purge_on_wipe_tower = 15`, while the
current public API/CLI output contains `15,15,15`. After the independent 3MF
allocation-label correction, this is the first primary difference at byte14613,
line207, column44 in both official and seeded runs.

That complete Native output does not export either nullable toolchange member.
Moreover, Rust already explicitly selects both typed vectors in
`options/project_variants/filament_retract.rs:169–190`. A coincidentally correct
scalar G-code result would therefore not demonstrate their owning-list
membership. Neither the purge failure nor a membership-only assertion is an
acceptable substitute for their own behavioral witnesses.

## Native-first public API witnesses

Create three separate retained inputs, one per cited member. Derive each from
the actual retained system H2D project in the post-A replay, binding its actual
archive SHA and every member payload SHA. Change only the selected member in
`Metadata/project_settings.config` from its existing three-slot vector to a
singleton: respectively `1.125`, `0.125`, or `17.5`. Preserve the original
`filament_self_index = [1,1,1]`, ordered variant strings, all sibling options,
templates, model/build metadata, placement, diameters, and geometry. Retain
actual ZIP bytes; do not normalize timestamps or equate payload and archive
identity.

For each input, qualify official and seeded Native before any Ares invocation:

1. Slice the original retained bytes through the existing Native case path.
2. Re-export the same original input through Native `--arrange 0 --export-3mf`.
   Assert the selected exported setting is the original singleton value in
   exactly three ordered slots. Keep the raw exported archive, member hashes,
   exit status, and commands as independently checkable artifacts.
3. Only after the Native premise succeeds, call public `ares_core::load_project`
   on the original input, not the Native-normalized output. Inspect the loaded
   `Project::settings().filament` through its existing public serialization and
   compare the selected vector against Native's three-slot result. This directly
   discriminates each omission: the unpatched owning list leaves that input
   vector at one slot. Finite non-default values prevent default substitution
   from accidentally satisfying the witness.
4. Retain the public observation before asserting equality, then run the existing
   strict Native/API/CLI whole-output comparator and raw API/CLI equality on the
   original input. The settings-vector witness supplements, never filters or
   replaces, whole-output comparison. Do not slice the re-exported archive as
   compensation for an unnormalized Ares import.

Commit this specification first, then the retained inputs and tests. Execute and
bind the actual pre-fix Native premises and public API failures before adding
production members. If export does not expose the intended normalization,
retain that premise failure and revise the witness before invoking Ares; do not
claim a comparison from an absent output.

## Exit criteria and deferrals

After the owning-list-only implementation, all three Native-first public vector
witnesses and their strict complete-output comparisons must pass in both modes.
Re-run frozen21 plus the independent A case and these three new cases:25 tests
per mode, not an expanded interpretation of retained12. The retained controls
still include only four shared strict Native/API/CLI triplets per mode; preserve
each other control's original contract.

Require fresh fmt, locked workspace/all-target Clippy with `-D warnings`,
core/browser-WASM checks, and changed Rust files no larger than400LOC. Preserve
A's loaded/allocated label distinction and original STL22 control. No new public
API, dependencies, filesystem/runtime behavior in core, legacy fallback, or
changes to physical/logical/firmware identities are included. Dynamic inventory,
sequential scheduling, tower algorithms, and adjacent configuration behavior
remain deferred. Rebase only after explicit B54 main landing confirmation, then
perform final exact-tip qualification and submit one combined nozzle review.
