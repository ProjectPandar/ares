# Complete catalog user-machine variant vectors

## Source boundary

Port completed user-machine printer vectors from OrcaSlicer v2.5.0
(`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`). This is preset loading, not
G-code formatting, a new variant system or a change to registered defaults.

`Preset.cpp:1927–1932` copies the completed parent, applies user differences,
and calls `pad_printer_variant_vectors` (`:375–389`). This inherited operation
only grows nonempty printer vectors to the existing printer-variant layout;
it does not truncate longer child vectors. Missing/empty printer variant
metadata leaves inherited padding unchanged. Empty option vectors remain empty.

A root user preset instead copies collection defaults and calls
`extend_default_config_length` (`Preset.cpp:1942–1944`). Its sizing rules at
`:313–368` use explicit printer variant count, otherwise nozzle count, otherwise
one. The stride-one owning set gets that count; the stride-two set gets twice
that count. Root completion can both grow and shrink.

The stride-two set is `PrintConfig.cpp:10273–10290`, including
`machine_max_junction_deviation` at10289. It stores normal/silent pairs per
variant. The registered singleton default (`:5444–5452`) is not its completed
cardinality. Completed vendor parents already pass through preset extension
at `PresetBundle.cpp:7848` before user inheritance copies them.

## Reuse, without rebuilding printer metadata

The existing Rust `SliceOptions::extended_preset_variants` in
`options/extruder_variant.rs` owns preset counts, owning-set precedence and
existing vector resizing. Reuse it for both printer sets rather than adding
another size/resize algorithm:

- Select printer stride-one/stride-two members plus nozzle-count metadata.
- For inherited presets, apply a computed replacement only when the source is
  nonempty and the target is longer. Retain longer authored vectors, empty
  vectors and the existing ID/variant association.
- For roots, apply the full existing extension result.

Do **not** directly call `SliceOptions::extend_extruder_variant` for this
operation. Its source counterpart, `PrintConfig.cpp:10595–10632`, rebuilds
printer IDs/variants from the extruder variant list and resizes machine limits;
calling it directly would clear/reconstruct metadata and may truncate values.
That is different from the inherited grow-only pad at `Preset.cpp:375–389`.
`get_parameter_size` (`PrintConfig.cpp:10563–10591`) shares the stride contract,
but does not make these two operations interchangeable.

Existing non-SEMM `Preset::normalize` behavior (`Preset.cpp:555–573`) remains
independent and unchanged. The actual SEMM=true witness deliberately skips its
`set_num_extruders` branch. Do not flip that guard to repair preset completion.

## Rust destination and invariants

At `profiles/catalog/compose.rs`, selected catalog user machines still bypass
the vendor-only preset-length call. Complete their raw printer vectors after
resolved layers merge, then synchronize the typed printer from that same raw
view, analogous to the existing user-process completion.

Use whether the selected document has a nonempty inherited parent to choose
inherited padding versus root extension. Exclude `direct_imports`, retain
existing vendor completion and preserve the later machine normalization policy.
Reuse the existing typed printer rebuild and project-runtime overlay.

Do not change static/default membership flags, footer serialization, register
a two-value junction default, force runtime activation, or special-case one
machine-limit key. This catalog remains a temporary compatibility shell around
upstream completed-preset loading, not an independent slicing pipeline.

## Verification

Reuse `inherited_catalog_user_presets_preserve_collection_defaults` without
changing its inputs, subcases, Native setup or strict whole-output comparator.
Its machine subcase starts at byte126801, line5122 column40: Native junction
values0.01,0.01 versus Ares0.01 despite one nozzle/ID/variant on both sides.
The process and filament subcases already pass and must stay passing.

Commit this specification before the fix; no new test is required for this
already failing public-interface witness. Run the unchanged fixed60-case
profile/catalog/STL/inheritance filter on baseline and tip in both modes, with
identical test sources and complete reference bindings. Record exact filter,
expanded inventory, per-test transitions and full original artifacts. Require
zero formerly passing cases to fail. The machine witness must pass or advance
its first difference; do not extend the port to a new unrelated difference.

Inherited long/empty-vector and root behaviors are source-bound decisions;
the existing witness is not a claim of new exhaustive parent/child permutation
coverage. Keep direct sparse membership and nil controls in the regression set.
Run workspace fmt, all-target clippy and wasm32 checks; bind final committed
diff to the tested source. Other unresolved slicing/filament work stays separate.
