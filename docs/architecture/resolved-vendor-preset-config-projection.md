# Resolved vendor preset configuration projection

## Rewrite boundary

Port the completed vendor-preset configuration returned by
`PresetBundle::resolve_preset_config`, not a new configuration pipeline.
Upstream is `345efec64c7b95a05ce5cf96df0dde66a158835c`; the starting Ares
commit is `8e42def8eaef06bb567e5d5c505a2daadb9fe4fb`.

The Rust destination is `profiles/catalog/compose.rs`, the existing
`profiles/project_refresh/keys` collection-default authority, and preset
normalization before profile composition. `project/domain/profile_input/stl.rs`
consumes the completed selected configuration. The typed/raw representation
remains a temporary compatibility shell for the upstream dynamic config.
Neither the footer writer nor registry static membership constructs a preset.

## Existing construction contracts

Preserve the following existing behavior:

- Print-host, bed, nozzle-volume and residual dynamic-only membership fixes:
  `full-config-printhost-membership.md`, `default-bed-type-membership.md`,
  `default-nozzle-volume-type-membership.md`, and
  `full-config-residual-membership.md`. Keep their corrected false static
  membership flags; a completed preset is a different producer of real keys.
- The nullable filament override default seed implements
  `PresetBundle.cpp:508-518`. Reuse it; do not repeat a nil-key list.
- The completed machine normalization port implements `Preset.cpp:535-541`.
  Preserve it and its raw-member retention.
- `profile-application-membership-and-vector-shape.md` separates retained
  project defaults from sparse replacement files. It does not complete
  manifest-resolved vendor selections; this slice must preserve that distinction.
- The profile-entry Native reference resolves the selected vendor leaves
  through their manifests, rather than treating their authored fields as a
  completed configuration.

## Upstream construction, in order

1. `OrcaSlicer.cpp:2118-2157,2238` resolves an inherited system leaf by its
   source file and vendor manifest. `PresetBundle.cpp:556-651` returns the
   resolved instantiated preset's **config**, not its authored JSON map.
2. `Preset.cpp:1735-1743,1787-1792` builds a collection default from the
   kind-owned option list and registered creation defaults. This can create
   dynamic-only members absent from static `FullPrintConfig`.
3. `PresetBundle.cpp:508-518` sets nullable filament override members of the
   default filament preset to nil. A registered numeric default is not the
   default filament-preset value for those members.
4. `PresetBundle.cpp:7627-7666` starts a root from its collection default or
   a child from its completed parent; applies ordered include diffs then own
   fields; extends variant vectors; removes misplaced keys; normalizes the
   completed instantiated preset. Include diffs are captured before extension.
5. `Preset.cpp:301-357` extends present variant vectors with first-value
   semantics and first-set-wins variant-class precedence. Printer variant-2
   members use twice the selected printer variant count. The nil replacement
   rule excludes filament extruder override keys. `Preset::normalize` then
   performs the separate extruder normalization.
6. The CLI applies these loaded configurations, produces the compatibility
   expression groups at `OrcaSlicer.cpp:3715-3729`, and completes static values
   at `4250-4259`. `GCode.cpp:8244-8347` serializes actual applied members,
   excluding banned and nil entries. It does not manufacture missing presets.

## JSON conversion precedes preset completion

`PresetBundle.cpp:8223-8235` loads each vendor file into `entry.config_src`
through `ConfigBase::load_from_json`. That loader calls
`handle_legacy_composite` after the file dictionary is loaded
(`Config.cpp:1147-1149`); `DynamicPrintConfig` delegates to
`PrintConfigDef` (`PrintConfig.hpp:957-958`). Thus vendor files do receive
composite conversion, but before parent/default/include completion.

`PrintConfig.cpp:10043-10072` rewrites a present, non-empty `thumbnails`
value into comma-space-separated definitions. An absent key is not created.
The selected LH files do not author that key; its collection creation default
is `48x48/PNG,300x300/PNG` (`PrintConfig.cpp:8595-8600`).
`PresetBundle.cpp:7627-7666` subsequently copies defaults or a completed
parent, applies included and own converted values, extends vectors and
normalizes the preset. It does not repeat the JSON composite conversion.

Keep collection-default construction separate from file deserialization in
Rust. Normalize authored thumbnails when each fragment loads, not after the
inheritance merge or when rebuilding a completed machine's typed projection.
A creation default absent from every loaded file keeps its registered token;
an explicitly authored value still receives the existing composite conversion.
Do not change the global thumbnail formatter. Likewise, copy completed raw
defaults through the same CLI transfer rule as authored members:
`OrcaSlicer.cpp:3715-3737` routes filament conditions into expression groups
and skips other filament scalars; the existing `catalog/raw.rs::applies`
remains the authority rather than a new metadata exclusion list.

## Closed witness and per-key ownership

The unchanged public witness is
`runner::profile_entry::profile_entry_stl_load_settings`: the same pre-placed
STL and LH vendor leaves on both sides, selected through their manifests,
Native repeat verification, then full ordered Native/Rust output comparison.
The witness has 20 differing config keys and no differences outside those
config rows. This diagnostic attribution does not weaken the comparator.

Registration citations below are in `PrintConfig.cpp`; ownership citations
are in `Preset.cpp` unless another file is named.

| Key | Native / Rust baseline | Actual producing boundary |
|---|---|---|
| `bbl_use_printhost` | `0` / absent | Registration 1177-1182; printer list 1592; collection default, not static membership |
| `compatible_machine_expression_group` | `;;` / absent | Registration 2248; CLI 3715-3722 builds the process/filament/machine slots from loaded conditions |
| `compatible_process_expression_group` | `""` / absent | Registration 2251; CLI 3723-3729 builds loaded filament condition slots |
| `default_bed_type` | empty / absent | Registration 1471; printer list 1601; real preset member even when empty |
| `default_nozzle_volume_type` | `Standard` / absent | Registration 6306-6324; printer list 1585; enum-vector preset default |
| `extruder_variant_list` | `"Direct Drive Standard"` / absent | Registration 6326-6331; printer list 1585; completed-preset normalization retains the default/derived vector |
| `flashforge_serial_number` | empty / absent | Registration 1229-1234; printer list 1592 |
| `group_algo_with_time` | `0` / absent | Registration 8829-8831; printer list 1609 |
| `host_type` | `octoprint` / absent | Registration 5759-5800; printer list 1592 |
| `ironing_expansion` | `0` / absent | Registration 5248-5256; process list 1490 |
| `machine_max_junction_deviation` | `0.01,0.01` / `0.01` | Registration 5444; static member PrintConfig.hpp:1672; limit list 1564 and printer variant-2 list PrintConfig.cpp:10286; extend_default_config_length owns the two slots |
| `pellet_modded_printer` | `0` / absent | Registration 4681-4685; printer list 1601 |
| `plugins` | empty / absent | Registration 6007; process list 1364 |
| `printer_agent` | empty / absent | Registration 1192-1200; printer list 1592 |
| `printer_plugin_config_overrides` | empty / absent | Registration loop 1304-1313; printer list 1614 |
| `printhost_authorization_type` | `key` / absent | Registration 1282-1292; printer list 1594 |
| `printhost_ssl_ignore_revoke` | `0` / absent | Registration 1269-1274; printer list 1595 |
| `thumbnails_format` | `PNG` / absent | Registration 8602-8617; printer list 1595 |
| `upward_compatible_machine` | empty / absent | Registration 2220-2224; printer list 1590; present because this is a completed vendor preset, not because STL import creates it |
| `filament_retract_length_nc` | absent / `10` | Registration 8928-8937 is nullable; filament list 1548; static nullable member PrintConfig.hpp:1931; default filament constructor nils it, and GCode's nil exclusion omits it |

There are 18 Native-only members, one vector-shape difference and one
Rust-only non-nil default. The expression groups are generated metadata,
not additional preset member-list entries. Do not add the 20 witness names
as a new runtime policy table.

## Required Rust construction

For instantiated vendor selections, make the selected raw configuration and
its typed profile agree with the collection-completed configuration before
CLI profile application. Reuse the existing kind-owned membership and
canonical default/nil constructor. Explicit inherited, included and leaf
values retain upstream precedence. Default-valued false, zero, empty string
and empty vector members are real values, not absence predicates.

Apply the existing source-owned variant membership and the upstream
extension/normalization order at the completed-preset boundary. Reuse the
existing normalizers where they already implement the rule. Do not resize
machine limits at the footer or infer their length from palette cardinality.
Preserve explicit nil handling, first-value padding and variant-set priority.

The complete filament config must reach the existing profile composition
and `catalog::cli_metadata::filament_conditions` path, so empty compatibility
conditions produce the same reserved groups as Native. Do not insert those
group rows as exporter exceptions. The default filament nil seed must reach
composition before absent registration defaults can turn into an applied 10.

Standalone sparse imported replacement files keep their current CLI
construction. Ordinary byte-only 3MF entry and retained project/catalog
refresh remain separate paths. No global registry-default expansion, changed
static membership bit, footer filter, vendor-name exception, new filesystem
lookup, public API or fallback is permitted.

## Public verification and failure model

The public seam is `ares_core::slice_with_profiles` versus live pinned Native
through the existing complete-stream runner. Preserve the original LH witness's
input, placement, selection, Native manifest premise and comparator. Require
complete ordered-byte equality in both runner modes, with repeatable input
and output artifacts.

Before implementation, verify that the test coverage distinguishes:

- Completed vendor defaults from truly sparse direct-import replacements.
- Real false/zero/empty/default-valued dynamic members from absent members.
- Default filament nil from explicitly authored numeric/nozzle-change values.
- One printer variant's two machine-limit slots from multi-variant extension.
- Complete output equality from a mere key-inventory comparison.

Use existing public controls where they already cover these distinctions;
add only a missing control before its implementation. Never infer PASS by
removing the 20 rows. Retain all original complete ordered-byte assertions.
A new mismatch beyond the closed set is a reported residual, not authority
to change another subsystem.

Require the same witness and existing profile-entry, raw direct-STL membership,
nullable override, include/inheritance and retained project controls in both
runner modes. Verify WASM compilation, cargo fmt and strict workspace/all-target
Clippy.

Deferred: the profile override-reference test premise, Web caller selection,
locked/enum UI test synchronization, CI oracle builder, new profile resolution
policy, geometry/motion/timing algorithms and general settings-view completeness.
