# Process variant selection with short authored IDs

## Upstream boundary

At OrcaSlicer 2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`),
`PrintApply.cpp:1421–1434` selects printer and process columns before saving
`m_ori_full_print_config`. The all-extruder branch of
`DynamicPrintConfig::update_values_to_printer_extruders`
(`PrintConfig.cpp:11672–11703`) resolves each physical request and uses column
zero on a miss. `ensure_process_variant_columns` (`:11602–11631`) only
synthesizes metadata when both authored identity vectors are singletons.

`DynamicPrintConfig::get_index_for_extruder` (`PrintConfig.cpp:10830–10879`)
uses authored IDs only if their count is at least the variant count. Otherwise
it generates lookup IDs by flattening the trimmed, nonempty variant tokens of
each physical head in `extruder_variant_list`, repeating the 1-based head ID
for each token. Matching requires both the head ID and the variant. These
lookup IDs are not the authored values exported: the selected indices read
all vectors, including `print_extruder_id`, with `ConfigOptionVector::get_at`
(`Config.hpp:676–680`), which repeats the first value on an out-of-range read.

## Rust boundary and scope

In `crates/ares-core/src/options/project_variants/index.rs`, the shared-variant
shortcut in `resolve_requests` must only apply when the authored ID map is
complete by the above predicate (`ids.len() >= variants.len()`). The existing
shortcut is an optimization of column-zero selection, not a separate source of
identity. Incomplete maps go through the existing generated-ID lookup and the
caller's existing miss rule. Do not change matching, variant synthesis, miss
rules, copied vector values or the exported ID vector.

Deferred: preset import/refresh ordering, hardware metadata normalization,
filament map refresh, and runtime nozzle config indexing. The two-column loss
occurs before the materialized process checkpoint is saved.

## Public witness

Generate a cube project from the stock H2D presets using the parity harness.
Author one process ID `[7]`, two identical `Direct Drive Standard` process
variants, and distinguishable outer and inner speeds. Test both hardware lists
with the same authored process settings:

- Narrowed to one Standard token per head: generated IDs `[1,2]` select
  columns `[0,1]`. Native exports outer `500,200` and inner `300,600`.
- Stock hardware list: the first two generated IDs are both for head 1, so
  the second head has no match and falls back to zero. Both speakers must
  continue to export outer `500,500` and inner `300,300`.

Both exports retain authored `print_extruder_id = 7,7`. Read the materialized
G-code config block through the public slice API and compare it to the pinned
Native output. Prove the narrowed case fails before the fix and keep the stock
control in the same official and seeded profile/variant inventory on both
sides of the change.
