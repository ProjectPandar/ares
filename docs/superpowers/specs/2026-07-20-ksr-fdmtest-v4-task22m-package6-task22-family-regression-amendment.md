# Task 22M Package 6 Task22-Family Regression Amendment

## Authority And Regression RED

This amendment supplements the approved Task 22M specification and plan
(SHA-256
`5433110c60aa4aa7e72f193fbdecde07d8ca3556704320aae3d39a148a02e2ff` /
`b5dd487ebe277982e26365377173fa3ecafc5bd31d4c1c5c267835f77aecede8`)
and every approved Package 4-6 amendment. The fixed OrcaSlicer commit/tree
remain `8500fcdccaa10b5099ac20d252af3a7c560046f1` /
`b62d6017ba1ac7cb986f70fd6844353c7a776549`; the fixed Ares baseline
commit/tree remain `fcd2c5728f4c0529f28bfc43c636507d61e263d8` /
`19557e2e520e6b6d0e758740fd00f57397b6fd2a`.

The Rust 1.91 command `cargo nextest run -p ares-core task22` exposed one
stale earlier-slice assertion after 442 tests passed. The existing
`task22j_loaded_modifier_control_target_j_is_exact_and_public_stays_incomplete`
test still expects both its modifier and control archives to reach
`ProjectSlicingIncomplete`. Task 22M now runs after the unchanged J checkpoint.
The modifier archive produces valid retained multi-region layers and therefore
must return the approved
`UnsupportedProjectFeature("multi_region_layer_slices")`; the single-region
control must continue to return `ProjectSlicingIncomplete`.

This RED does not identify a production defect. The exact J bytes remain
correct for both archives, and the Task 22M specification explicitly requires
the later public multi-region gate before mutation. Weakening that gate or
changing either J identity would violate the approved boundary.

## Authorized Delta

Modify exactly:

- `crates/ares-core/src/project_slice/tests/region_fixture.rs`.

Rename the affected test so its name describes the later Task 22M region gate.
Keep the exact J byte, parser, and repeatability assertions unchanged. Extend
the existing case table with the expected public error: the modifier case uses
`UnsupportedProjectFeature("multi_region_layer_slices")`, and the control case
uses `ProjectSlicingIncomplete`. Compare the actual public error with that
case-specific value.

Do not change production code, fixture construction, checkpoint identities,
other tests, or source boundaries. The approved exact 68-path frame becomes an
exact 71-path frame: the prior 68 paths, this previously unchanged test path,
this specification, and its companion plan.

## Acceptance

- the focused renamed test goes from the recorded RED to GREEN on Rust 1.91;
- its modifier and control J identities remain exact;
- `cargo nextest run -p ares-core task22` completes all 509 tests;
- Task 22M remains 81/81 and Task 22L remains 53/53;
- the synthetic and KSR M identities remain unchanged;
- strict workspace clippy/check, rustfmt, WASM/browser, LOC, macro/unsafe,
  hardcoding, stale-L, and diff gates remain GREEN.

Any production change or broader test edit requires another approved
amendment.
