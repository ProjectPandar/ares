# Model settings object-tag contract

## Source boundaries

The pinned upstream source is `OrcaSlicer/src/libslic3r/Format/bbs_3mf.cpp` at `_BBS_3MF_Exporter::_add_model_config_file_to_archive` (`8296`) and its object opening tag (`8312–8313`). Native writes an object element with an `id` attribute and the `AUTO_DROP_ATTR` attribute. The related part element remains a `part` element with its own subtype and id attributes.

The affected Rust boundary is the parity fixture-authoring helpers in `crates/ares-cli/tests/orca_parity/runner/compatible_regions/solid_pattern_fixture.rs`, `top_pattern_fixture.rs`, `bridge_anchor_length_projection.rs`, and `bridge_anchor_max_projection.rs`. Their object-scoped insertion currently locates an object with an id-only textual pattern, although the project writer emits additional object attributes.

## Destination

Replace only object-scoped textual matching with a small attribute parser for the opening `object` element. The parser must select an `object` element, require an `id` attribute, and require exactly one matching object start. It may accept the other attributes written on that same object element, but it must not match a different element or an arbitrary tag. Part-scoped matching remains unchanged and continues to require the existing part form.

## Included

- The shared object-scoped helper used by the solid-pattern fixture and its two bridge-anchor child cases.
- The independent object-scoped helper in the top-pattern fixture.
- Existing object/part scope selection, one-object cardinality, unchanged ZIP-member hashes, metadata absence and insertion checks, activated option checks, and all existing Native replay/API/CLI/full-output assertions.
- A source-cited parity test layer proving that the authoring helper reaches the intended metadata insertion point on the actual Native object tag shape.

## Deferred

- The Native importer, project loader, model settings representation, and production slicing code.
- Geometry, bridge, top-surface, anchor, routing, normalization, motion, FlushMode, and statistics behavior.
- Any change to the comparison oracle, output normalization, input fixtures, or assertion strength.
- Any claim that the downstream twenty cases become byte-equal after this authoring-contract repair.

## Tests

First run the existing twenty-case selection on the unchanged base to retain the exact tag-locator failure. Then run the modified selection in both official and seeded environments. The RED acceptance point is that the authoring helper passes its object-tag precondition and the original downstream parity assertions remain active and fail at their next real boundary where applicable. Run the four added bridge-anchor cases as part of the same selection. Verify API and CLI invocation paths remain present, and retain the exact one-object and part-scope checks.

## Failure modes

A match is invalid if it accepts a part element, a non-object tag, an object without `id`, or more than one object start. A parser that merely searches for `object` or accepts any tag would hide malformed fixture construction. A change that removes member hashes, metadata checks, activation checks, Native replay, API/CLI equality, or full-file comparisons is out of scope. A downstream parity change must not be attributed to this contract without a separate source boundary.
