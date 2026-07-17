# Task 22A: Typed Project Slicing Parameters and Fixed Layer Planning Implementation Plan

> **Execution contract:** Follow the approved SDD workflow and this checklist
> in order. No production or test implementation may begin until these exact
> plan bytes receive literal `VERDICT: APPROVE` from both a fresh independent
> Codex reviewer and the required default-model OpenCode reviewer. Execute the
> six bounded vertical packages with fresh implementer subagents and no package
> commits. Every package needs fresh specification-compliance and code-quality
> approval. Update tracked architecture/roadmap documentation, commit, and push
> only after whole-implementation approval and a fresh release matrix.

**Approved specification:**
`docs/superpowers/specs/2026-07-16-ksr-fdmtest-v4-task22a-fixed-layer-planning.md`

**Approved specification bytes / SHA-256:**
`38193` /
`064B42A33353981F4E3750A8C7C71598016F87C96CB936F5B86866D2E4BFD664`

**Pinned OrcaSlicer checkout / SHA:**
`C:\Users\Indexyz\AppData\Local\Temp\Ares-Orca-8500fcdc` /
`8500fcdccaa10b5099ac20d252af3a7c560046f1`

**Ares baseline SHA / branch:**
`4281e913b8eeaaeb6111cbefdf06f896f5c611aa` /
`codex/ksr-fdmtest-v4-parity`

## Goal and immutable behavior ledger

Port only the approved one-dimensional `libslic3r` slicing-parameter and fixed
layer-planning slice. Project bytes are loaded and resolved once; the existing
exact Bambu config block is generated before Task 22A capability gates; the
source object's first instance and every model-part vertex determine its
`ModelObject::max_z()`-compatible height; typed object extruders determine
nozzle limits; and the fixed profile produces complete bounded layer records.
A valid supported project still returns `ProjectSlicingIncomplete`, but only
after its private plan exists.

The fixed upstream boundary is the approved spec's cited portions of:

- `Slicing.hpp` and `Slicing.cpp` for `SlicingParameters`, nozzle helpers,
  `layer_height_profile_from_ranges`, and `generate_object_layers`;
- `Model.cpp::ModelObject::update_min_max_z` for first-instance, all-vertex
  model-part bounds;
- `PrintRegion.cpp::collect_object_printing_extruders`,
  `PrintObject.cpp::object_extruders`, and `Config.hpp::get_at` for object
  extruders and the deliberate zero-based-then-subtract-one fallback;
- `PrintApply.cpp` / `PrintObject.cpp` / `PrintObjectSlice.cpp` for modifier
  ZAA ownership, midpoint slice Z, layer construction, and lifecycle order;
- `Format/bbs_3mf.cpp` for painted profile and range archive ownership.

Executable tests assert behavior only. They must not execute or parse the
pinned source checkout, assert source paths/line numbers/symbol names, or become
source-level pinning tests.

### Non-negotiable exact semantics

- `Metadata/project_settings.config` remains the sole flattened full settings
  snapshot; sparse model/range metadata remains typed and no profile ID is used
  to discover external presets.
- `min_layer_height` and `max_layer_height` are nonempty effective-config
  preconditions, validated after `nozzle_diameter` and before
  `filament_diameter`, `filament_map`, config writing, or planning.
- Unsupported gates are project-wide key-major:
  `layer_height_profile`, `layer_height`, `raft_layers`, `enable_support`,
  `enforce_support_layers`, `precise_z_height`, then `zaa_enabled`.
- Any typed true parameter-modifier `zaa_enabled` is conservatively rejected at
  planning stage, even when definitely nonintersecting. False/absent is not.
- Bounds use the source object's first instance, composed as
  `instance.then(volume)`, every vertex of model-part volumes, and maximum Z
  directly. Triangle references and minimum-Z normalization are forbidden.
- Object-extruder sources are concrete resolved region feature selectors plus
  raw model-part/parameter-modifier volume/object fallback. A bare range
  `extruder` is never directly appended. An occupied range may affect concrete
  feature selectors; a nonintersecting range stays only in released bounded
  usage.
- `filament_map` is not read by nozzle planning. Object-extruder IDs 0, 1, 2,
  and out of range preserve the approved subtract-one / first-value behavior.
- The fixed first pair is unconditional. Candidate midpoint equality stops.
  `precise_z_height=false` never aligns the final top.
- `MAX_PLANNED_LAYERS_PER_PROJECT` is exactly 100,000 across all objects and
  transform groups. The loop computes `next_print_z` once, requires finite
  strict progress, and reuses that value.
- The KSR plan is one object and 460 records; final `print_z` has bits
  `0x4057000000000036`. These are test evidence, never production branches.

Task 21A scaled XY geometry, triangle-plane slicing, contours, regions,
perimeters, fills, supports, toolpaths, successful G-code, and final G-code
parity remain explicitly deferred. The persistent full-parity goal remains
active after this task.

## Frozen baseline and workspace discipline

Before Package V, create ignored evidence file
`.superpowers/sdd/task22a-evidence.md`. Record the approved spec and plan
hashes, baseline SHA/branch/status, command exit codes, test manifests, package
path hashes, reviewer identities, and release evidence. Never stage it. Do not
edit or trust the older `progress.md` or `task20a2-evidence.md` ledgers.

The frozen baseline worktree contains only the approved untracked spec before
this plan. Preserve unrelated user changes if any appear. Ordinary `git diff`
omits untracked files, so inspect them by full read and
`git diff --no-index -- /dev/null <path>`.

Baseline invariants:

- dynamic baseline: 675 LF rows, SHA-256
  `0DCEA4C112EF10F0D6E8C8EE7F63CFEF1831D7C2AE2E399016F1E38372543BE7`;
- dynamic allowlist: 2 rows, SHA-256
  `6B9C3BA6A1C52118A14D66F607CF85A9D13C27185B1FA22D670983E9371A94B6`;
- KSR 3MF: 183,007 bytes, SHA-256
  `698F40F13C9075B818ABEDD3D10F022FBB5D8200AED48FBDDE651F6BFB21B8A9`;
- KSR G-code: 6,339,134 bytes, SHA-256
  `10AEC9A156849F59929B578429A764A61453996A5834056F600C0ADBB5D6A1B3`;
- KSR option oracle: 456,004 bytes, SHA-256
  `33C99EE71594ED7F80B44ABC3007DF8E9AE4EC0800411E3B5DBA500F47FD085B`;
- exact KSR config block: 49,004 bytes, SHA-256
  `b33c979097a4900700d1e5dfcaa16f1454a79ce5fec48da7eb9458cfa2fdeeb8`.

Fresh baseline `nextest list` counts are:

- `cargo +1.91.0 nextest list -p ares-core task22a_`: 0;
- `cargo +1.91.0 nextest list -p ares-core -E 'test(/project_slice/)'`: 0;
- `cargo +1.91.0 nextest list -p ares-core -E 'test(/^project::/)'`: 261;
- `cargo +1.91.0 nextest list -p ares-core -E 'test(/config_export/)'`: 29;
- `cargo +1.91.0 nextest list -p ares-core --test no_unapproved_dynamic_values`:
  29;
- `cargo +1.91.0 nextest list -p ares-core`: 4,637.

All new or renamed Task 22A tests must begin with `task22a_`. Package GREEN
counts for that prefix are frozen below as 3, 14, 18, 26, 36, and 42. A count
drift requires review before implementation continues.

## Subagent-Driven execution and review discipline

Implementation packages are deliberately serial because each later vertical
slice consumes the same `project_slice.rs` lifecycle and shared test module.
Parallel writes in the shared worktree would corrupt genuine RED/GREEN
evidence. Parallelism is used for the two independent package reviewers and
for whole-review roles.

For every package:

1. dispatch one fresh implementer with only its owned paths, approved spec/plan
   hashes, dependencies, expected RED, and acceptance commands;
2. require the implementer to add or rewrite tests first, run the stated RED,
   record the exact missing-symbol/assertion failures, then write production;
3. inspect the complete owned patch, run `git diff --check` plus no-index
   checks for untracked files, and freeze path/SHA-256 hashes;
4. dispatch a fresh specification-compliance reviewer and a different fresh
   code-quality reviewer in parallel; both must return literal
   `VERDICT: APPROVE`;
5. on revision, use a bounded fixer, rerun affected tests and the package gate,
   refreeze, and rerun both reviews.

Do not commit between packages. Do not add `#[allow(...)]`,
`#[expect(...)]`, a temporary legacy fallback, test-only production behavior,
or placeholder G-code. Every changed/new Rust file must stay below 400 physical
lines.

After each package GREEN, run all applicable gates below. Package V still has
the frozen zero-test `project_slice` manifest: list it and require exactly zero,
but do not invoke a zero-match run because nextest correctly exits 4. Starting
with Package G, require the list to be nonempty and run the `project_slice`
filter normally.

```powershell
cargo +1.91.0 nextest list -p ares-core task22a_
cargo +1.91.0 nextest list -p ares-core -E 'test(/project_slice/)'
cargo +1.91.0 nextest run -p ares-core task22a_
# Package G and later only; Package V requires zero from the preceding list.
cargo +1.91.0 nextest run -p ares-core project_slice
cargo +1.91.0 nextest run -p ares-core project
cargo +1.91.0 nextest run -p ares-core config_export
cargo +1.91.0 nextest run -p ares-core --test no_unapproved_dynamic_values
cargo +1.91.0 fmt --all -- --check
cargo +1.91.0 clippy --workspace --all-targets --all-features -- -D warnings
cargo +1.91.0 check --workspace --all-targets --all-features
cargo +1.91.0 check -p ares-core --target wasm32-unknown-unknown
```

Require the package's exact Task 22A count, 675 unchanged dynamic rows, the
unchanged empty allowlist behavior, and no new warning or ignored failure.

## Exact tracked manifest

**Create:**

- `crates/ares-core/src/project_slice/capabilities.rs`
- `crates/ares-core/src/project_slice/bounds.rs`
- `crates/ares-core/src/project_slice/extruders.rs`
- `crates/ares-core/src/project_slice/parameters.rs`
- `crates/ares-core/src/project_slice/profile.rs`
- `crates/ares-core/src/project_slice/layers.rs`
- `crates/ares-core/src/project_slice/state.rs`
- `crates/ares-core/src/project_slice/tests.rs`
- `crates/ares-core/src/project_slice/tests/support.rs`
- `crates/ares-core/src/project_slice/tests/capabilities.rs`
- `crates/ares-core/src/project_slice/tests/bounds.rs`
- `crates/ares-core/src/project_slice/tests/parameters.rs`
- `crates/ares-core/src/project_slice/tests/profile_layers.rs`
- `crates/ares-core/src/project_slice/tests/integration.rs`
- `crates/ares-core/src/project_slice/tests/fixture.rs`
- `crates/ares-core/src/project/tests/model/layer_height_profile.rs`
- the already approved specification file;
- this plan file.

**Modify:**

- `crates/ares-core/src/project_slice.rs`
- `crates/ares-core/src/project/domain.rs`
- `crates/ares-core/src/project/load.rs`
- `crates/ares-core/src/project/layer_config_ranges.rs`
- `crates/ares-core/src/project/effective_config/cardinality.rs`
- `crates/ares-core/src/project/effective_config/types.rs`
- `crates/ares-core/src/project/effective_config/candidates.rs`
- `crates/ares-core/src/project/tests/model.rs`
- `crates/ares-core/src/project/tests/layer_config_ranges/association.rs`
- `crates/ares-core/src/project/tests/layer_config_ranges/invalid.rs`
- `crates/ares-core/src/project/tests/effective_config/cardinality.rs`
- `crates/ares-core/src/project/tests/effective_config/candidates.rs`
- `crates/ares-core/src/project/tests/effective_config/usage.rs`
- `crates/ares-core/src/project/tests/effective_config/config_export.rs`
- after whole approval only: `docs/architecture/option-parity-v4.md` and
  `docs/roadmap.md`.

**Delete:** nothing.

No other tracked path may change. In particular, keep
`scripts/dynamic_value_baseline.txt`, `scripts/dynamic_value_allowlist.toml`,
the committed 3MF/G-code/option oracle, old STL `planning.rs`, `pipeline`,
`segments`, `contours`, CLI/WASM signatures, workspace membership, dependencies,
and unrelated `PrintApply` scaffolds byte-identical. An indispensable extra path
requires a spec revision and fresh dual spec approval before it is touched.

---

## Package V: Enforce nonempty nozzle-height vectors before the writer

**Owned paths:**

- `crates/ares-core/src/project/effective_config/cardinality.rs`
- `crates/ares-core/src/project/tests/effective_config/cardinality.rs`
- `crates/ares-core/src/project/tests/effective_config/config_export.rs`

This first vertical package has no new module and must remain independent of
layer planning.

### V.1: Establish the behavioral RED

Add exactly these three prefix tests:

1. `task22a_nozzle_height_vectors_reject_each_empty_vector_in_fixed_order`;
2. `task22a_nozzle_height_vectors_accept_singletons_for_many_extruders`;
3. `task22a_nozzle_height_vector_errors_precede_config_writer`.

The first test clears `min_layer_height` and `max_layer_height` separately, then
both together; it requires keyed `InvalidInput` and min-before-max order. The
second proves one value remains valid for multiple physical extruders. The
third starts from the existing invalid-flush-matrix KSR helper, makes each vector
empty in turn, and proves step-2 vector validation wins over the step-3 writer
failure.

Run before production:

```powershell
cargo +1.91.0 nextest list -p ares-core task22a_nozzle_height_vector
cargo +1.91.0 nextest run -p ares-core task22a_nozzle_height_vector --no-capture
```

Require three listed names and nonzero behavioral failures: current validation
incorrectly accepts the empties, and the lifecycle case reaches the flush-matrix
error. The singleton case may already pass; at least the two empty-vector
contracts must be genuine REDs.

### V.2: Make only the helper precondition GREEN

Immediately after the existing nonempty `nozzle_diameter` check, require length
one for `min_layer_height` and then `max_layer_height` using the existing
`validate_minimum_len` / `invalid_option` path. Do not require physical or
logical cardinality equality and do not resize, default, or map either vector.
Preserve the remaining order beginning with `filament_diameter` and
`filament_map`.

Run the package gate with the explicit Package V zero-`project_slice` exception
above and require exactly three `task22a_` tests. Freeze and dual-review
Package V before G.

---

## Package G: Retain typed deferred inputs, stable identity, and capability gates

**Owned paths:**

- `crates/ares-core/src/project_slice.rs`
- `crates/ares-core/src/project_slice/capabilities.rs`
- `crates/ares-core/src/project_slice/tests.rs`
- `crates/ares-core/src/project_slice/tests/support.rs`
- `crates/ares-core/src/project_slice/tests/capabilities.rs`
- `crates/ares-core/src/project/domain.rs`
- `crates/ares-core/src/project/load.rs`
- `crates/ares-core/src/project/layer_config_ranges.rs`
- `crates/ares-core/src/project/effective_config/types.rs`
- `crates/ares-core/src/project/effective_config/candidates.rs`
- `crates/ares-core/src/project/tests/model.rs`
- `crates/ares-core/src/project/tests/model/layer_height_profile.rs`
- `crates/ares-core/src/project/tests/layer_config_ranges/association.rs`
- `crates/ares-core/src/project/tests/layer_config_ranges/invalid.rs`
- `crates/ares-core/src/project/tests/effective_config/candidates.rs`
- `crates/ares-core/src/project/tests/effective_config/usage.rs`

Package G is a vertical gate: every new production field/accessor is consumed
by `capabilities::validate`, and `slice_project` calls that function only after
the existing Bambu writer. No temporary dead-code suppression is allowed.

### G.1: Freeze eleven new/renamed tests as RED

The cumulative `task22a_` manifest must grow from 3 to 14 with exactly:

1. `task22a_painted_layer_height_profile_presence_is_case_insensitive_and_opaque`;
2. `task22a_painted_layer_height_profile_absence_is_false`;
3. `task22a_range_layer_height_is_typed_separate_and_last_write_wins`;
4. `task22a_range_layer_height_invalid_value_is_bounded`;
5. `task22a_range_duplicate_replacement_clears_prior_layer_height`;
6. `task22a_source_object_index_survives_filtered_object`;
7. `task22a_capability_gates_each_named_feature`;
8. `task22a_capability_gate_order_is_project_key_major`;
9. `task22a_zaa_gate_scans_candidate_and_nonintersecting_modifier`;
10. `task22a_zaa_false_modifier_is_supported`;
11. `task22a_zaa_gate_runs_after_config_writer`.

Write tests before production. Reuse the project test `ProjectParts` helper for
reader tests. In the new project-slice test support, use small typed objects for
domain tests and a test-only ZIP rewriter around committed KSR 3MF bytes for
lifecycle mutations. Never read the reference G-code.

Required distinctions:

- painted profile presence is `false` when absent and `true` for exact,
  mixed-case, empty, or opaque payload entries; the payload is not parsed;
- range `layer_height` is `Option<OrcaFloat>`, is excluded from
  `RegionOptionOverrides::present_keys`, later duplicate keys win, malformed
  text remains bounded, and complete duplicate-range replacement can clear it;
- object zero with no print groups followed by printable object one yields one
  resolved object whose source index is one in both shell and candidate passes;
- every named capability is independently rejected, combined sources use the
  complete project key-major order, and object-one `layer_height` beats
  object-zero `raft_layers`;
- candidate ZAA and raw true modifier ZAA are rejected. A definitely
  nonintersecting true modifier is still rejected, while false/absent is not;
- with both invalid flush matrix and true modifier ZAA, the writer error wins.
  Repairing only the matrix exposes `UnsupportedProjectFeature("zaa_enabled")`.

Run:

```powershell
cargo +1.91.0 nextest list -p ares-core task22a_
cargo +1.91.0 nextest run -p ares-core task22a_ --no-capture
```

Require compile REDs for the missing profile/range accessors,
`source_object_index`, and capability module, followed by behavioral REDs for
the new gates. Any unrelated compile error is a test defect.

### G.2: Retain painted-profile presence without decoding

In `load.rs`, scan the already validated archive-path set with
`eq_ignore_ascii_case("Metadata/layer_heights_profile.txt")` and retain only a
boolean. Add it to `ProjectDocuments` and expose a crate-private `Project`
accessor. Do not open the entry, validate payload syntax, keep bytes, or reject
multiple case variants.

### G.3: Retain exact range `layer_height` separately

Add private `Option<OrcaFloat>` and a crate-private value accessor to
`LayerConfigRange`. Intercept only the exact canonical key. Decode it through
the existing typed object-model codec, use later-option-wins, and route every
other option through the unchanged region codec. Preserve duplicate-range
whole replacement and lexicographic sorting exactly; do not add the field to
`RegionOptionOverrides` or recognize aliases.

### G.4: Thread stable source identity

Add `pub(crate) source_object_index: usize` to `ResolvedProjectObject` and copy
`groups.source_object_index` in the sole production constructor for both shell
and candidate resolution. Update the one manual usage-test literal. Do not infer
source identity from final resolved vector position.

### G.5: Apply project-wide key-major capability gates after writing

Implement a private typed validator. Evaluate project-global painted-profile
presence, then for each remaining key scan participating resolved objects in
final order through their stable source indices. Gate range `layer_height`,
object `raft_layers` / support / precise Z, candidate region ZAA, and raw typed
true parameter-modifier ZAA in the exact approved order.

Wire the validator into `slice_project` after the existing optional config-block
writer and before `ProjectSlicingIncomplete`. Do not add parameter planning yet.
The call is the real consumer for every G field and must keep Clippy clean.

Run the package gate and require exactly 14 `task22a_` tests. Freeze and
dual-review Package G before B.

---

## Package B: Port source-first-instance all-vertex object bounds

**Owned paths:**

- `crates/ares-core/src/project_slice.rs`
- `crates/ares-core/src/project_slice/bounds.rs`
- `crates/ares-core/src/project_slice/tests.rs`
- `crates/ares-core/src/project_slice/tests/support.rs`
- `crates/ares-core/src/project_slice/tests/bounds.rs`

### B.1: Add four bounds REDs

Grow the cumulative manifest from 14 to 18:

1. `task22a_bounds_use_first_instance_all_vertices_and_model_parts_only`;
2. `task22a_bounds_reuse_source_height_across_transform_groups`;
3. `task22a_bounds_accept_negative_or_nonzero_min_without_normalization`;
4. `task22a_bounds_reject_empty_nonpositive_and_nonfinite_results`.

Synthetic typed objects must distinguish:

- first source instance from a later grouped transform;
- all vertices from triangle-referenced vertices, including a vertex-only
  model-part and an unreferenced highest vertex;
- model parts from negative, modifier, support-enforcer, and support-blocker
  volumes;
- direct maximum Z from extent or minimum-Z normalization;
- no finite sample, nonpositive maximum, and finite inputs whose transform
  arithmetic overflows to nonfinite.

Add tests first and require the missing `bounds` module/symbol compile RED.

### B.2: Implement and consume bounds

Compose the source object's first instance transform with every model-part
volume transform using `then`. Transform every mesh vertex in `f64`, retaining
finite min/max only for diagnostics. Do not inspect triangle indices. Return a
bounded project-object-Z `InvalidInput` for no sample, any nonfinite result, or
nonpositive maximum. Return maximum Z unchanged otherwise.

After capability validation, make `slice_project` walk participating resolved
objects by `source_object_index` and call the bounds function once per source
object before returning incomplete. This vertical call is temporary only in
placement, not behavior: later packages reuse the same result in full planning.
No value-based fixture special case is allowed.

Run the package gate and require exactly 18 `task22a_` tests. Freeze and
dual-review Package B before N.

---

## Package N: Derive object extruders and typed slicing parameters

**Owned paths:**

- `crates/ares-core/src/project_slice.rs`
- `crates/ares-core/src/project_slice/extruders.rs`
- `crates/ares-core/src/project_slice/parameters.rs`
- `crates/ares-core/src/project_slice/tests.rs`
- `crates/ares-core/src/project_slice/tests/support.rs`
- `crates/ares-core/src/project_slice/tests/parameters.rs`

### N.1: Add eight parameter REDs

Grow the cumulative manifest from 18 to 26:

1. `task22a_object_extruders_cover_six_feature_gates_and_print_wide_brim`;
2. `task22a_object_extruders_include_model_modifier_and_object_fallbacks`;
3. `task22a_range_extruder_reaches_nozzles_only_through_occupied_feature_fallback`;
4. `task22a_nozzle_lookup_preserves_zero_one_two_and_out_of_range_fallback`;
5. `task22a_nozzle_limits_apply_defaults_clamps_and_multi_source_aggregation`;
6. `task22a_filament_map_does_not_affect_nozzle_limits`;
7. `task22a_first_layer_height_uses_positive_value_or_regular_fallback`;
8. `task22a_invalid_slicing_parameter_numbers_are_keyed`.

Required source distinctions:

- exercise all six feature selectors with each wall/infill/shell gate, plus a
  print-wide brim on another qualifying object;
- for every model-part/parameter-modifier volume, positive volume selector wins,
  otherwise object selector, otherwise one; explicit zero adds no raw source;
- an occupied range generic `extruder` may flow into active concrete feature
  selectors, while an identical definitely nonintersecting raw range selector
  remains present in released bounded usage and absent from this object vector;
- helper IDs 0 and 1 both select vector index zero, ID 2 selects index one, and
  every resulting out-of-range index selects the first value;
- zero min becomes 0.07, configured min clamps to 0.01, zero max becomes
  75 percent of nozzle, max is at least min, multiple IDs aggregate max/min,
  and regular `layer_height` expands the final pair outward;
- changing only a nonidentity `filament_map` leaves all parameters identical;
- positive initial height wins and nonpositive initial height uses regular
  `layer_height`; finite/positive validation is keyed.

Add tests first and require missing `extruders` / `parameters` symbol REDs.

### N.2: Collect complete per-object extruders

From every resolved candidate model-part region, apply the six exact feature
gates and Orca region selector normalization. Compute print-wide supported brim
once across all participating objects. Separately add raw model-part and
parameter-modifier volume/object fallbacks. Sort and deduplicate after both
families.

Do not call or modify `effective_config/usage.rs`. Do not append a bare range
`extruder`. Do not scan painted MMU facets. Do not use `filament_map`.

### N.3: Construct the supported `SlicingParameters` subset

Define one private typed parameter record with the fields required by profile,
pair, and later mesh slicing. Preserve the source helper's odd index operation
without Rust unsigned overflow:

```text
lookup = object_extruder_id.checked_sub(1).unwrap_or(usize::MAX)
value  = vector.get(lookup).unwrap_or(vector.first())
```

Vector emptiness is impossible only because Package V validated it; do not add
a fallback member. Start aggregate min at 0.01 and max at `f64::MAX`, use ID
zero when the object vector is empty, apply nozzle rules, then expand around
regular layer height. Retain first/object heights, Z minima/maxima, identity
shrinkage, zero raft values, and effective nozzle bounds.

After bounds in `slice_project`, derive object extruders and parameters for each
participating source object. Use the result so every new function is a real
lifecycle consumer; layer generation remains Package L.

Run the package gate and require exactly 26 `task22a_` tests. Freeze and
dual-review Package N before L.

---

## Package L: Port fixed profiles, bounded pairs, and complete layer records

**Owned paths:**

- `crates/ares-core/src/project_slice.rs`
- `crates/ares-core/src/project_slice/profile.rs`
- `crates/ares-core/src/project_slice/layers.rs`
- `crates/ares-core/src/project_slice/tests.rs`
- `crates/ares-core/src/project_slice/tests/profile_layers.rs`

Target `profile.rs < 150` and `layers.rs < 250` physical lines. Keep profile
compression separate from pair/record generation.

### L.1: Add ten pure-behavior REDs

Grow the cumulative manifest from 26 to 36:

1. `task22a_fixed_profile_compresses_equal_and_preserves_unequal_heights`;
2. `task22a_fixed_profile_uses_strict_epsilon_and_unrounded_top`;
3. `task22a_fixed_first_pair_is_unconditional_at_and_above_top`;
4. `task22a_midpoint_equal_to_top_stops_before_candidate`;
5. `task22a_layer_series_preserves_nondivisible_unaligned_top_and_records`;
6. `task22a_layer_series_is_deterministic`;
7. `task22a_smallest_positive_regular_height_rejects_nonprogress`;
8. `task22a_layer_budget_allows_exact_limit_and_rejects_next`;
9. `task22a_layer_budget_spans_objects_and_groups`;
10. `task22a_layer_generation_error_precedence_is_fixed`.

Compare complete profile vectors, complete bottom/top pairs, and every
`PlannedLayer { id, height, print_z, slice_z }`. The strict epsilon cases use a
representable zero-to-`0.0001` unequal boundary and a below-boundary case.
Required edge cases include:

- equal and unequal first/regular heights;
- first height equal to and greater than object height, both retaining one pair;
- a next midpoint exactly equal to object top, which must not append;
- a nondivisible height whose final top is not aligned;
- smallest positive regular height after a 0.2 first layer, which must report
  non-progress without a zero-height pair;
- exactly 100,000 records allowed, record 100,001 rejected, and one shared
  budget across multiple objects/groups;
- nonfinite candidate/intermediate, midpoint stop, non-progress, budget, append
  precedence.

Add tests first and require missing profile/layer symbols as the RED.

### L.2: Port the empty-range fixed profile

Implement only `layer_height_profile_from_ranges` with no explicit height
ranges: fixed first interval, uncovered regular interval, and source append
compression using strict `abs(delta) < 1e-4`. Do not round Z or height values.
For KSR parameters the vector must be `[0, 0.2, 92, 0.2]` by ordinary typed
input, not a fixture branch.

### L.3: Port bounded source-order pair generation

Define private `LayerPair`, `PlannedLayer`, `PlannedPrintObject`, and one
project-total budget. Emit the fixed first pair before the loop. Follow the
source profile cursor/interpolation and midpoint stop. For a continuing
candidate:

1. reject nonfinite derived values;
2. compute `next_print_z = print_z + height` once;
3. require it finite and strictly greater than `print_z`;
4. require one remaining project budget record;
5. append using that exact value.

The first pair consumes budget before emission. Exactly 100,000 total records
succeeds only when the series ends there. Convert pair index to ID, `hi-lo` to
height, `hi` to print Z, and `0.5 * (lo+hi)` to slice Z.

### L.4: Build one plan per resolved transform group

Factor a private `plan_project` path in `project_slice.rs`. Run gates once; for
each resolved source object compute bounds/extruders/parameters/profile once,
then generate one deterministic record vector for every transform-group index
using the shared budget. Preserve final resolved order. A source object without
a print group produces no plan.

The local planned vector must be fully produced before
`ProjectSlicingIncomplete`. Package I will move it into the final owning state;
do not expose it publicly or serialize it.

Run the package gate and require exactly 36 `task22a_` tests. Freeze and
dual-review Package L before I.

---

## Package I: Own the final state and prove lifecycle/KSR integration

**Owned paths:**

- `crates/ares-core/src/project_slice.rs`
- `crates/ares-core/src/project_slice/state.rs`
- `crates/ares-core/src/project_slice/tests.rs`
- `crates/ares-core/src/project_slice/tests/support.rs`
- `crates/ares-core/src/project_slice/tests/integration.rs`
- `crates/ares-core/src/project_slice/tests/fixture.rs`

### I.1: Add six integration REDs

Grow the cumulative manifest from 36 to exactly 42:

1. `task22a_lifecycle_preserves_archive_effective_and_writer_precedence`;
2. `task22a_lifecycle_reaches_planning_error_then_incomplete`;
3. `task22a_non_bambu_skips_writer_but_runs_planning`;
4. `task22a_private_state_owns_single_project_config_block_and_plans`;
5. `task22a_ksr_fixture_plans_exact_460_layers_from_3mf_only`;
6. `task22a_ksr_fixture_plan_is_deterministic_and_config_block_unchanged`.

Use deterministic `GenerationMetadata`. The lifecycle matrix must prove:

- malformed archive remains the reader error;
- empty min/max and other effective-config errors precede writer/planning;
- Bambu writer failure precedes capability and numeric planning errors;
- after repairing each earlier input, the next exact gate/error appears;
- a supported project with invalid finite/positive planning input reaches its
  keyed planning error;
- a fully valid supported Bambu or non-Bambu project reaches
  `ProjectSlicingIncomplete` only after planning;
- non-Bambu skips invalid writer-only data but still runs gates/bounds/plan.

The fixture tests include only the committed 3MF, never the G-code or
`options-v242.json`. Assert one planned object, 460 complete records, first
`(0,0.2,0.2,0.1)`, final print-Z bits `0x4057000000000036`, every ID/height/
print-Z/slice-Z invariant, strict slice-Z increase, and deterministic repeated
state preparation. Separately assert generated config block length/hash using
the already frozen Task 19C test constant.

Add tests first and require the missing private preparation/state seam as the
genuine RED.

### I.2: Introduce one owning private state

Implement private synchronous preparation called by the existing async byte
API. It performs exactly:

1. `load_project`;
2. `resolve_bounded_project_config` once;
3. optional Bambu `write_config_block` once;
4. `plan_project` once;
5. construct `ProjectSliceState` owning the `Project`, the single resolved
   config, optional block bytes, and planned objects.

`slice_project` consumes that state into `ProjectSlicingIncomplete`. A private
test-only accessor may borrow complete state fields from tests inside the same
module; do not add a public/crate API, clone/reload project bytes, or rebuild the
resolved config. Ensure non-test code reads/destructures every state field
without lint suppression.

Run the package gate and require exactly 42 named `task22a_` tests. Also require
the existing Task 19C fixture config-export tests and all five existing
project-config-export lifecycle tests GREEN. Freeze and dual-review Package I.

---

## Integrated package gate

Freeze the V, G, B, N, L, and I submanifests plus the complete implementation
manifest. Re-run fresh specification-compliance and code-quality review for
each package against integrated bytes and GREEN evidence. Every role must again
end in literal `VERDICT: APPROVE`; a correction invalidates both reviews for
every affected package.

Run:

```powershell
cargo +1.91.0 nextest list -p ares-core task22a_
cargo +1.91.0 nextest run -p ares-core task22a_
cargo +1.91.0 nextest run -p ares-core project_slice
cargo +1.91.0 nextest run -p ares-core project
cargo +1.91.0 nextest run -p ares-core config_export
cargo +1.91.0 nextest run -p ares-core --test no_unapproved_dynamic_values
```

Require exactly 42 `task22a_` names, 675 dynamic baseline rows, unchanged
allowlist, exact 49,004-byte KSR config block/hash, one 460-layer KSR plan, and
the public incomplete boundary.

## Freeze, structural audits, and whole implementation reviews

Build ignored SHA-256 manifests for every tracked/untracked implementation path
and the complete patch. Require `git diff --check` plus no-index whitespace
checks for untracked paths. Reject any path outside the exact manifest.

Audit all added production/test lines and full new files for:

- no `ksr_fdmtest_v4`, fixture/reference hash, `460`, `92.0`, timestamp,
  `generated by`, or filename branch in production;
- no reference-G-code or `options-v242.json` access from new tests;
- no `serde_json::Value`, `Map<String, _>`, runtime registry/option dispatch,
  JSON round-trip, `Any`, C++ binding, Orca process, filesystem, terminal, UI,
  OpenGL, or platform-specific core code;
- no direct bare-range-extruder append, `filament_map` nozzle mapping, grouped
  transform bounds, triangle-index bounds, min-Z normalization, final-top
  alignment, or legacy STL fallback;
- exact painted/range/modifier unsupported gates and project key-major order;
- exact first-value nozzle indexing, nozzle min/max rules, fixed profile,
  first-pair/midpoint behavior, one-computation progress check, and shared
  100,000-record budget;
- no added `#[allow(...)]` / `#[expect(...)]` or test-only production branch;
- every changed/new Rust file below 400 physical lines. Keep targets:
  `project_slice.rs < 220`, each production child `< 300`,
  `tests/support.rs < 360`, and every focused test child `< 390`.

Prove `scripts/dynamic_value_baseline.txt`,
`scripts/dynamic_value_allowlist.toml`, all three KSR fixture/oracle files,
old STL planning modules, and unrelated user paths are byte-identical.

Run the fresh local release matrix:

```powershell
cargo +1.91.0 nextest run --workspace
cargo +1.91.0 fmt --all -- --check
cargo +1.91.0 clippy --workspace --all-targets --all-features -- -D warnings
cargo +1.91.0 check --workspace --all-targets --all-features
cargo +1.91.0 check -p ares-core --target wasm32-unknown-unknown
cargo +1.91.0 check -p ares-wasm --target wasm32-unknown-unknown
cargo +1.91.0 build -p ares-wasm --release --target wasm32-unknown-unknown
cargo +1.91.0 nextest run -p ares-cli
cargo +1.91.0 install --locked wasm-bindgen-cli --version 0.2.121
wasm-bindgen --version
wasm-bindgen target/wasm32-unknown-unknown/release/ares_wasm.wasm `
  --target web --out-dir target/wasm-browser
npm --prefix crates/ares-wasm/tests/browser ci
npm --prefix crates/ares-wasm/tests/browser audit --audit-level=low
npx --prefix crates/ares-wasm/tests/browser playwright install chromium
npm --prefix crates/ares-wasm/tests/browser test
```

Require `wasm-bindgen 0.2.121`, zero npm vulnerabilities, and the real-project
headless Chromium test GREEN. On Windows capture `$LASTEXITCODE` immediately;
do not use Playwright `--with-deps` locally.

Dispatch three fresh reviewers against the identical frozen manifest, patch,
and evidence:

1. whole-specification implementation reviewer:
   literal `VERDICT: APPROVE`;
2. whole-code-quality reviewer:
   literal `VERDICT: APPROVE`;
3. default-model OpenCode implementation reviewer invoked without `-m`:
   literal `VERDICT: APPROVE`.

Any revision requires a focused regression where applicable, rerunning affected
checks, rebuilding all hashes, and rerunning all three whole reviews. Do not
update tracked architecture/roadmap documentation before all three approve.

## Documentation gate

Only after whole implementation approval, modify:

- `docs/architecture/option-parity-v4.md`;
- `docs/roadmap.md`.

Document only approved Task 22A behavior:

- the pinned upstream boundary and private `SlicingParameters` /
  `PlannedLayer` ownership;
- typed painted-profile/range-height presence and conservative modifier ZAA
  rejection;
- stable source identity, first-instance all-vertex max-Z bounds, complete
  object-extruder source partition, and deliberate nozzle helper indexing;
- fixed profile/pair/record semantics and the generic 100,000-record limit;
- the KSR one-object/460-record evidence and unchanged exact config block;
- the unchanged public `ProjectSlicingIncomplete` boundary.

Keep variable/adaptive layers, modifier geometry, scaled XY/Clipper, mesh-plane
slicing, paths/G-code, generated metadata, and successful full KSR parity
explicitly deferred. Do not call Task 22A released before exact-pushed-SHA Tier
1 succeeds.

Require a fresh documentation reviewer to return:

```text
ROLE: DOCUMENTATION
VERDICT: APPROVE
```

Revise/re-review until approved. Add both docs to the frozen final manifest and
rerun the complete focused gates and local release matrix from approved doc
bytes. Any implementation change invalidates whole and documentation reviews.

## Conventional commit, push, and exact-SHA Tier 1

Apply the Conventional Commits skill only after all approvals and the fresh
post-documentation matrix are GREEN.

Stage only the frozen manifest; never use `git add -A`:

```powershell
git status --short
git diff --check
git add -- <exact reviewed manifest paths>
git diff --cached --name-status
git diff --cached --check
```

Confirm ignored evidence, generated WASM/npm output, pinned Orca checkout,
dynamic baseline/allowlist, fixture/reference/oracle files, old STL modules,
and unrelated user changes are not staged. Use reviewed subject:

```text
feat(slicing): plan fixed project layers
```

Push normally without force:

```powershell
git push origin codex/ksr-fdmtest-v4-parity
```

If remote advanced, fetch/rebase without dropping user changes, rerun relevant
verification, and push normally. Require local/tracking/direct remote SHA
identity and a clean worktree:

```powershell
$branch = 'codex/ksr-fdmtest-v4-parity'
$local = git rev-parse HEAD
$tracking = git rev-parse "origin/$branch"
$direct = ((git ls-remote origin "refs/heads/$branch") -split '\s+')[0]
git status --short
```

Locate only the Tier 1 push run whose `headSha` equals `$local`, watch it to
completion, and require all five jobs GREEN:

```powershell
gh run list --workflow tier1.yml --branch $branch --commit $local --event push `
  --json databaseId,headSha,status,conclusion,createdAt --limit 10
gh run watch <exact-run-id> --exit-status
gh run view <exact-run-id> --json headSha,conclusion,jobs
```

Required jobs are `format`, `ubuntu-latest`, `wasm`, `macos-latest`, and
`windows-latest`. Only then record Task 22A as released in ignored evidence.
The persistent full-G-code-parity goal remains active.

## Plan exit criteria

This plan is complete only when:

- exact spec and plan bytes were dual-approved before implementation;
- Packages V, G, B, N, L, and I each followed genuine test-first RED/GREEN and
  received independent spec-compliance and code-quality approvals;
- all 42 frozen Task 22A tests, complete project/config/dynamic regressions, and
  the real 3MF 460-layer plan are GREEN;
- the dynamic baseline/allowlist and fixture/oracle files remain byte-identical;
- whole specification, whole quality, default OpenCode, and documentation
  reviews approved identical bytes;
- the fresh native/WASM/browser release matrix passed;
- only the exact frozen manifest was conventionally committed and pushed
  normally;
- local/tracking/direct SHAs match and all five exact-pushed-SHA Tier 1 jobs are
  GREEN.

**Status: DRAFT — production and test implementation is forbidden until a
fresh independent Codex plan reviewer and the required default-model OpenCode
plan reviewer both return literal `VERDICT: APPROVE` for these exact bytes.**
