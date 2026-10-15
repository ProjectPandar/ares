# SPEC: ObjectImporter zero-defaults missing resource vertex coordinates

Status: **PROPOSED, docs only; no runtime or fixture authorization.** Follows
approved resource build/item GREEN `37ec261a4bf3dd95d1e1442aaffb8d3df48ea21b`
in stack-19. No Native/API/CLI/nextest runs until this SPEC is approved. M73
discovery remains deferred.

## Upstream boundary

OrcaSlicer `8500fcdccaa10b5099ac20d252af3a7c560046f1`,
`src/libslic3r/Format/bbs_3mf.cpp`:

- Active resource `ObjectImporter` workers are created by
  `_load_model_from_file:1780-1817`.
- Resource XML dispatch calls `_handle_object_start_vertex` from
  `_handle_object_start_model_xml_element:5612-5649` (vertex dispatch) and
  closes it through `:5651-5687`.
- `_handle_object_start_vertex:5478-5490` appends a vertex using
  `bbs_get_attribute_value_float` for `x`, `y`, and `z`; the source comment
  explicitly states missing values are set equal to ZERO.
- The same ObjectImporter path uses `object_unit_factor` set by
  `_handle_object_start_model:5315-5319`, so present coordinates remain scaled
  by the model unit. This proposal changes only absent coordinate defaults.

The Rust rewrite boundary is private `project/model_xml.rs::Vertex` and the
existing resource/main graph decoding. Current Rust requires all three `@x`,
`@y`, and `@z`, whereas the cited upstream resource parser supplies zero for a
missing attribute. This is a source-cited compatibility slice, not geometry
repair or a new Ares policy.

## Proposed exact behavior

For resource model parts only, decode absent `@x`, `@y`, or `@z` as `0.0` before
the existing unit scaling and mesh validation. Preserve explicit values,
non-finite rejection, triangle index checks, XML namespace/entity/security
limits, and main-model behavior unless the upstream boundary proves it shared.
Do not add fallback parsing, XML rewriting, filename heuristics, broad serde
defaults, diagnostic I/O, or a public API. Keep the resource/main role seam
introduced by the approved resource build-item port.

The current `Vertex` DTO is shared by both role decoders; the implementation
must not silently broaden required main-model fields without a source-cited
reason. If a minimal role-specific DTO is required, keep it private and explain
the conversion; otherwise stop/amend this proposal rather than use a global
`#[serde(default)]` that changes main behavior.

## Proposed input and gates (not frozen)

Use the exact current-main control-before50 pair from the approved resource
fixture family only after SPEC approval. Derive one positive by removing exactly
one vertex coordinate attribute from a resource model entry, preserving every
other ZIP entry and byte. Freeze complete entry SHA manifest and reproducer
before any test commit. No fixture or test is authorized by this document.

Native-first official and seeded, fresh `--slice 0 --outputdir`, positive,
repeat, control; CPU15 and canonical environments. Require Native exit0,
complete footer, and full generator-only normalized neutrality before public
calls. Then validated `slice_project_plate_validated` plus real bare CLI on the
same inputs. Public RED, if observed, must assert the target Native-equivalent
success and whole strict output/raw API-CLI equality, therefore fail until GREEN;
never encode `is_err` or `assert_ne` as a passing RED. Controls assert equality
too. If Native does not complete or differs, stop with NO-RED; do not substitute
inputs or retry.

After a separately authorized RED, a production GREEN must keep every complete
Native/API/CLI stream equal, preserve M73/config/motion/arcs/statistics/footer,
and pass targeted existing 3MF/load regressions, fmt, strict clippy, and WASM.
No full/workspace runs under an active coordinator freeze.

## Included / deferred

Included: the named ObjectImporter missing-coordinate zero default for resource
vertices, with existing unit and validation behavior retained.

Deferred: missing triangle attributes, malformed mesh repair, non-finite values,
main-model schema relaxation, component/relationship policy, object metadata,
colors, transforms, seams, configuration, and M73. Any residual G-code delta
after GREEN is STOP/report, not downstream scope growth.
