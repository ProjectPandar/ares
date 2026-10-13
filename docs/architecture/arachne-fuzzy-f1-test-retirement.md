# F1 obsolete rejection test retirement

Status: coordinator approved this exact retirement after public RED a31c6142c.
Delete only the named test, its sole helper and newly unused imports.
The closure debug-assert ruling is separate and applies in F1 production.

Existing test:
`crates/ares-core/src/project_slice/tests/perimeters/arachne/traverse.rs:320–327`,
`active_fuzzy_skin_fails_closed`. It constructs External/first-layer/Ripple
fuzzy settings, calls traversal for a closed wall and asserts
`UnsupportedProjectFeature("fuzzy_skin")`. The helper at90–103 only constructs
that test's active configuration.

Source contradicting that obsolete negative contract:
`PerimeterGenerator.cpp::traverse_extrusions:386–387` calls apply_fuzzy_skin;
`Feature/FuzzySkin/FuzzySkin.cpp::apply_fuzzy_skin:689–694` admits single-group
eligible junction fuzzification; `fuzzy_extrusion_line:348–352` dispatches Ripple.
Native preflight accepts the corresponding input in official and seeded.
F1 public Native/API/CLI RED will identify the existing guards before any
production edits, and replace this private guard expectation with full public
coverage for noise/modes, eligibility, variable width and task Context.

Approved concrete change, committed separately from F1 production:

- Delete only `active_fuzzy_skin_fails_closed` and its sole-used
  `active_fuzzy_skin` configuration helper.
- Remove only imports made unused by those deletions: OrcaBool, OrcaFloat,
  OrcaInt, ProcessFuzzySkinMode, ProcessFuzzySkinType, ProcessNoiseType,
  ProcessRegionSourceOptions, RegionOptions and SliceError.
- Keep Percent (used by the overhang context), FuzzySkinConfig (disabled
  context), all other fixtures, every other test body and assertion unchanged.
- Commit this retirement separately from F1 GREEN, with a message citing
  PerimeterGenerator.cpp:386–387, FuzzySkin.cpp:689–694 and the actual F1 RED hash.

Approval cites PerimeterGenerator.cpp:386–387, FuzzySkin.cpp:689–694 and
RED a31c6142c. The closure assertion ruling remains a separate production change.
