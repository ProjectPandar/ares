# PrintConfig enum declarations, wire admission, and deferred execution

## Upstream and Rust boundary

Pinned upstream: OrcaSlicer `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`.
This ports `libslic3r::PrintConfigDef` enum declarations and type maps, not a
new slicing pipeline. Rust destinations are the existing registry metadata,
`OptionDefinition::enum_type_keys`, typed option enums, and the existing
`UnsupportedProjectFeature` capability boundary. Core remains portable and
contains no filesystem, terminal, or Native subprocess behavior.

All CPP/HPP references below are in `libslic3r/PrintConfig.{cpp,hpp}` unless
another header is explicitly named.

| Key | CPP option | CPP map | HPP enum | Change |
|---|---|---|---|---|
| bottom_surface_pattern | 2627–2635 | 310–340 | 127–139 | Add GUI SpiralInset, shared type key, shifted IDs |
| brim_type | 2116–2140 | 557–567 | 455–466 | Add leading_edge_only |
| filament_map_mode | 3129–3150 | 734–740 | 633–640 | Add Nozzle Manual; Default remains GUI-only sentinel |
| host_type | 5759–5801 | 197–217 | 119–123 | Add ultimaker, shifted IDs |
| internal_solid_infill_pattern | 2681–2689 | 310–340 | 127–139 | Copy top-pattern GUI lists including SpiralInset |
| ironing_type | 5163–5178 | 343–349 | 203–210 | Labels only |
| nozzle_type | 4480–4498 | 624–631 | CommonDefs.hpp:12–22 | Labels only, retain wire undefine |
| print_order | 2275–2303 | 413–420 | 318–326 | Add best_of and snake |
| printer_structure | 4508–4524 | 641–649 | 558–566 | Labels only, retain wire undefine |
| sparse_infill_pattern | 3808–3866 | 310–340 | 127–139 | Fix shifted IDs, do not add GUI SpiralInset |
| support_interface_pattern | 7570–7589 | 470–479 | 374–376 | Add SpiralInset and shifted IDs |
| top_surface_pattern | 2549–2573 | 310–340 | 127–139 | Add SpiralInset and shifted IDs |

Promote exactly these twelve complete metadata records from the corrected
source generator. Their own labels/tooltips belong to the same declaration;
no other record belongs here. Remaining full generated drift must be 211;
stop and report a different count rather than filtering unexpected records.

## Wire and GUI are different contracts

`Config.hpp:2032–2040` looks up the shared enum keys map. GUI lists and their
validation are separate (`PrintConfig.cpp:10519–10559`). Existing Rust scalar
substitution prefers `enum_type_keys` over metadata GUI values
(`options/json_deserialize.rs:249–307`, `model_config_deserialize.rs:153–169`).

Add the missing shared infill type key, including its existing ironing and
support-ironing consumers, but do not broaden those consumers' GUI lists.
Compute metadata IDs from upstream enum maps/headers, not GUI positions or
Rust declaration ordinals. `spiralinset` is a valid InfillPattern type key but
not a sparse GUI value. `filament_map_mode`'s `Default` GUI row has no map key
and keeps its source -1 metadata code; do not invent wire admission from
header-only `fmmDefault`. Likewise `ntE3D` in `CommonDefs.hpp` has no matching
NozzleType wire key. Label `Undefined` does not rename wire `undefine`.

Seven wire-backed typed additions in six existing owners:

- `ProcessInfillPattern::SpiralInset`, `ProcessSupportInterfacePattern::SpiralInset`,
  and `ProcessBrimType::LeadingEdgeOnly` in process object enums.
- `ProcessPrintOrder::{BestOfStrategies, Snake}` in process print enums.
- `PrintHostType::UltiMaker` in printer remaining enums.
- `ProjectFilamentMapMode::NozzleManual` in project G-code enums.

Do not add unsupported header-only serde variants, compatibility aliases,
opaque fallback values, or unrelated enum states.

## Explicit execution boundary and behavior change

Upstream has SpiralInset: `Fill/FillBase.cpp:65–70` constructs
`FillSpiralInset`, whose class is in `Fill/FillSpiralInset.hpp:13–40` and entry
points are in `Fill/FillSpiralInset.cpp:310,347`. Ares does not yet implement
that algorithm. Concentric/Archimedean/Octagram are not substitutes.

Deferred: SpiralInset fill/support/raft generation; belt leading-edge brim
(`Brim.cpp:506–508`); Best/Snake ordering (`GCode.cpp:3685–3687`);
NozzleManual physical planning; UltiMaker host transport. Settings import,
metadata and codecs may represent these states, but slicing must return an
explicit `SliceError::UnsupportedProjectFeature` for each of the seven states.
No mapping to an existing algorithm, silent success, `panic!`, or
`unreachable!` is permitted for user-admitted input. Extend the existing
capability/error boundary, not a new execution architecture.

The previous public project path silently substitutes defaults for all seven
states and successfully slices. This change intentionally replaces that
success with a named unsupported error after source-correct admission:

| Authored state | Previous substituted value |
|---|---|
| top_surface_pattern=spiralinset | monotonicline |
| support_interface_pattern=spiralinset | auto |
| brim_type=leading_edge_only | auto_brim |
| print_order=best_of | default |
| print_order=snake | default |
| host_type=ultimaker | octoprint |
| filament_map_mode=Nozzle Manual | Auto For Flush |

RR must compare these per-state outcomes and disclose this intentional API
behavior change. Existing unrelated inputs must not acquire failures.

Before implementation, audit tracked fixture/parity text and compressed 3MF
configuration members for all seven states. Report any existing affected
parity cases before continuing. Keep actual paired P→F checking distinct from
this static inventory.

## Failure inventory and E2E plan

Tests precede production edits at public boundaries:

1. Twelve public metadata cases compare fresh pinned generator output with
   `print_config_def_json()`: enum values, labels, integer codes and owning
   display fields. Expected metadata is not hardcoded. Native runtime metadata
   is unavailable: `OrcaSlicer.cpp:8318–8327` comments out metadata help and
   `--export-settings` exports values, not metadata. No Native patch/probe.
2. Nine new GUI-state wire observations use original Native `--export-settings`
   and public Ares import/codec roundtrip: top/bottom/internal/support SpiralInset,
   leading-edge brim, NozzleManual, UltiMaker, Best, Snake. Retain producer,
   exact runtime input, input SHA, export and comparison. If a field is omitted
   by Native export, report visibility rather than skipping or inventing data.
3. Seven independent public slicing cases must get a precise unsupported error,
   not default-substituted success, a different input error, or a panic. Retain
   loaded values, substitutions, and the slice result for before/after review.
4. Preserve existing valid members, unknown-token substitution, sparse GUI
   exclusion, source Default sentinel behavior, and undefine wire spelling.
5. Audit exhaustive and non-exhaustive readers before admitting new states:
   raft/fill projections, smoothability predicates, object-instance ordering,
   brim/perimeter decisions, filament recommendation/variant/bed-exclusion logic.
   Minimal predicate updates must follow source; unsupported paths must fail
   explicitly, never reuse another strategy.

Failure modes include GUI/type-map confusion, shifted ordinal IDs, valid-token
substitution, invented header-only wire keys, serde rejection after apparent
admission, hidden execution fallback, broad metadata promotion, omitted Native
fields counted as success, and missing recorded public behavior transitions.
Run identical relevant baseline/final subsets; stop and report P→F. FULL is
left to integration. Run nextest, rustfmt and strict Clippy.

## Web impact

`web/app/src/sidebar/OptionField.tsx:47–48,100–110` uses enum labels with value
fallback, builds choices from `enum_values`, and commits the wire string.
Correct metadata exposes the source choices without a new widget or frontend
whitelist. This is not a claim of execution support: the existing public error
path must show explicit unsupported results. No UI redesign, feature flags,
metadata filtering, or browser execution claim belongs here.
