# ARD-0091: Lightning generator configuration ownership

## Upstream boundary

`PrintObject.cpp1010-1020::prepare_lightning_infill_data` activates a generator
when any printing region uses Lightning with positive sparse density.
`Fill/Lightning/Generator.cpp66-100::Generator` takes width, density, multiline
and angle parameters from `shared_regions()->all_regions.front()`, not the
activating region. `PrintObject.cpp2494-2498` separately retains a pattern-only
bridge flag.

## Decision and Rust destination

Rewrite these configuration operands in
`prepare_infill/bridge_over_infill/transaction/candidate_expansion.rs::build_lightning_generator`.
Keep activation distinct from ownership; the first stored region supplies
parameters even if it is not Lightning or has no layer geometry. Do not select
the first positive or first nonempty region. Keep the bridge flag unchanged.
The existing generator remains the consumed Rust rewrite of the upstream
Generator, not a second Ares pipeline or compatibility fallback.

## Included and deferred

Included: generator activation, configuration owner and tests consuming actual
generated polylines through the existing bridge transaction seam. Retain all
existing arithmetic and generator implementation. No wider production/test
visibility, new unsupported-feature guard or Normal/Tree changes.

Deferred but required: all-region outlines and bridge expansion, shared
multi-range ordering equivalence, public modifier output parity, all-printer
and option coverage, and Tree support completion. Current single-range region
storage supplies the first global configuration; broader mapping must receive
its own output evidence. Internal injection tests supplement, not replace,
public complete-output acceptance. Normal/Tree WIP remains held.
