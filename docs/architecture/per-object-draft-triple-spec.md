# SPEC: remaining triple draft / positive minimum / outer brim


## Upstream boundary and exact proposed change

This is completion of the existing libslic3r _make_skirt / GCode group rewrite,
not an independently designed pipeline or a new parameter/admission policy.

Print.cpp:575-586,2646-2696 defines positive-loop infinite shield hull/lifetime.
Print.cpp:2703-2778 fixes initial flow/filament-area conversion and a fresh
positive-only filament budget per generated group, before reversed export.
Extruder.hpp:38-41 owns filament-flow conversion.
Print.cpp:2794-2819 appends plate-frame brim-area outer contours before occupied
hull/grouping;2830-2900 fixed-point grouping uses configured nominal reach,
not additional minimum loops.2898-2918/2972-2996 retain members, reverse loops
and retain independent brim owners. GCode.cpp:4260-4331 allocates actual loops
and advances physical Z;4391-4452 prints all first-layer loops, then own-range
innermost only for single-loop mode;4456-4540/5325-5336 dispatch once per group
and Z, then each member's first-layer independent brim then content.
GCode.cpp:5748-5775 owns current-position splitting; brim bytes can change
with minimum/mode even when brim area is unchanged. No cross-input equality.

Exact destination: crates/ares-core/src/project_slice/gcode_emit/skirt/
generate.rs: remove ONLY the remaining D && M && B term, where D is non-disabled
draft, M is nonzero minimum and B is any materialized BrimPlan. Keep the original
resolved.objects.len()>1 and every other guard term byte-identical. Do not add
a new guard, error, fallback, comparison rule or configuration restriction.
Removal is conditional on the complete newly admitted categorical
matrix below. Do not claim that the four public RED inputs alone prove it.

Existing source ports must be reused, not duplicated:
- loop_generation::make_loops / generate's per-group call: positive-only
  closed-length*initial-flow/filament-area budget; resets per group, not member,
  copy, brim E or physical export layer.
- occupied::items / BrimPlan::covered_hull: area-derived contours plus one
  print-object-origin shift before convex hull; no double instance shift or
  emitted-path-bound substitute.
- member_groups / SkirtPlan own range / layers::object_skirt done state:
  nominal fixed-point reach, stable membership, advancing Z and physical-height
  flow; own-range innermost selection, never global flat last().
- layers::brim_emit / brim::emit / motion::skirt_brim: independent first-layer
  ownership and current-position clipping; no new skirt/brim budget abstraction.
Compatibility shells stay temporary where they differ from cited upstream.
First raw difference determines a minimal source-cited interaction repair.

## Newly admitted categorical matrix and coverage

D=false, M=false or B=false are already admitted by this term; removing it
only changes D=true,M=true,B=true. Positive-loop per-object plus multiple source
objects is the existing outer gate. Legal draft enum states are Disabled/Enabled;
legal min_skirt_length is nonnegative (registry metadata min0; options.rs skirt
range parser); therefore legal M=true is positive, not negative/NaN policy.
No numeric/schema validation changes are included.

Other existing terms independently reject prime tower, sequential ordering,
multiple filaments, support/raft, tower corners, and any generated brim with
combine_brims=true, non-OuterOnly type or non-singleton instance ownership.
Those remain rejected; no duplicate coverage/admission claim for those cases.

New source branches to represent: full/single export; disjoint/shared skirt
groups; and each occupied item with or without brim areas while B=true globally.
The last dimension was NOT covered by the four initial both-brim inputs.
Supplemental discovery therefore exhausts all nonempty brim-owner masks for
the two-source representative scene: both owners, left only, right only.
This checks both a group first member with brim and first member without brim.

| New branch dimensions | Native evidence |
|---|---|
| both-brim × far/near × full/single | original4 inputs, official+seeded8/8 accepted |
| left-only/right-only × far/near × full/single | supplemental8 inputs, official+seeded16/16 accepted |
| no brim (B=false), no minimum (M=false) | existing composition/draft controls, unchanged admission |

This is categorical branch coverage, not enumeration of every real minimum,
brim width or mesh. Parameter magnitude and object/group cardinality remain
source-owned generic loops/convex hull/fixed-point algorithms already ported;
no numeric/scene-specific implementation or guard is included. If another
newly reachable source branch is identified, removal is blocked until
Native evidence covers it; do not silently declare it deferred while removing
its only rejection. Existing exclusions are only the preserved guard policies.

## Discovery facts and immutable public RED inputs

Original inputs differ from the existing composition brim inputs only by minimum20.
Enabled, loops2/height1/distance6; OuterOnly width8/gap0/combine0; singleton,
one filament/by-layer; uniform4mm objects/0.2mm layers, far and near layouts.
Native8/8 accepted, actual20 positive-motion skirt Zs0.2..4.0, far2 groups/near1.
First-layer per-group skirt E far20.10565 each, near23.31296; single later
far3.87567 / near5.71919 (<20), so minimum is generation budget, not a later
layer top-up. First layer all/single skirt+brim motion identical; every later
single has own smaller inner bounds/E. Two independent brim owners first only.
Existing Ares CLI4/4 exits1 UnsupportedProjectFeature("skirt_type per-object"),
no G-code: all four demonstrably diverge, with triple still deferred correctly.

Supplemental inputs: only one object's model_settings brim_type=no_brim/brim_width=0 override differs
from original triple inputs. Native16/16 accepted and honors override: one
first-layer brim, no later brims,20 skirt Zs. Far2 separate budgets produce
20.10565 and21.36385 first E (order swaps); near shared group first E20.62989/
20.62991. Single later far3.87567/2.41445, near5.04847/5.04851. Source/brim-owner
masks do not duplicate group emission or turn minimum into a brim budget.
Discovery is not public RED or strict parity. No Native output is committed.

| Four immutable public RED inputs | SHA256 |
|---|---|
| far-triple-all | `6527dadfd9dd928e0f0274d66d515b1f3e0c87e0fa0decb451daaf637ab2c741` |
| far-triple-single | `ac1f97bc9e5f19b77cde81f3cd936ef6e291af0f140d4fc01896aca54a88ce70` |
| near-triple-all | `93d9c5a3f78b1e9ee83d374304af75fce9b78cf878669fa7eb334d9fb266d41d` |
| near-triple-single | `9e932b03701e81fb8fcb469df0d9acb53619ea3a43f533c200372d7a44f2e158` |

## Exit gates

1. Public RED commits exactly the four original pure inputs above with strict
   Native-first API+CLI success assertions; expected4guardFAIL per mode, not
   rejection-as-success. Parent composition12 tests/controls unchanged.
2. Only existing draft term removed;
   first raw differences drive source-minimal repair, no speculative port rewrite.
3. GREEN: same-input complete ordered Native/API/CLI strict comparison and raw
   API/CLI equality; physical lifetime, local first-layer minimum, later own
   innermost/no-top-up, independent first-layer brims. Supplemental mixed-mask
   Native evidence must stay valid and mixed exports need strict same-input
   verification, not merely return0, before claiming whole-term removal ready.
4. Preserve all prior composition12/draft12/min+brim4/min8/brim4/group4 and
   combined/single/zero-loop controls/related option domains; raw control bytes
   unchanged from RED. Fixtures/assertions/comparator/classification immutable.
5. Directed legs, strict clippy, fmt and WASM pass with serial Native modes
   and fresh caches/artifacts. <=400LOC; mechanical split before growth.
