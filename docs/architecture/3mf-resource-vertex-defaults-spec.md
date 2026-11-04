# SPEC: 3MF importer zero-defaults missing vertex coordinates

Status: **proposed.** Follows the resource build/item port
`37ec261a4bf3dd95d1e1442aaffb8d3df48ea21b`. M73 discovery remains deferred.

## Upstream boundary

OrcaSlicer `8500fcdccaa10b5099ac20d252af3a7c560046f1`,
`src/libslic3r/Format/bbs_3mf.cpp`:

- Active resource `ObjectImporter` workers are created by
  `_load_model_from_file:1780-1817`.
- Resource XML dispatch calls `_handle_object_start_vertex` from
  `_handle_object_start_model_xml_element:5612-5649` (vertex dispatch) and
  closes it through `:5651-5687`.
- `_BBS_3MF_Importer::_handle_start_vertex:3772-3782` appends a main-model
  vertex using
  `bbs_get_attribute_value_float` for `x`, `y`, and `z`; the source comment
  explicitly states missing values are set equal to ZERO.
- `ObjectImporter::_handle_object_start_vertex:5485-5495` appends a resource
  vertex using
  `bbs_get_attribute_value_float` for `x`, `y`, and `z`; the source comment
  explicitly states missing values are set equal to ZERO.
- The same ObjectImporter path uses `object_unit_factor` set by
  `_handle_object_start_model:5315-5319`, so present coordinates remain scaled
  by the model unit. This proposal changes only absent coordinate defaults.

The Rust rewrite boundary is the single `project/model_xml.rs::Vertex` decoder
used by both main and resource model documents. Current Rust requires all three
`@x`, `@y`, and `@z`, whereas both cited upstream parsers supply zero for a
missing attribute. This is a source-cited compatibility slice, not geometry
repair or a new Ares policy. `unit_factor` continues to apply at each upstream
location before the resulting coordinate enters geometry validation; the Rust
existing unit conversion remains unchanged.

## Proposed exact behavior

For both main and resource model parts, decode absent `@x`, `@y`, or `@z` as
`0.0` before the existing unit scaling and mesh validation. Preserve explicit values,
non-finite rejection, triangle index checks, XML namespace/entity/security
limits, and main-model behavior unless the upstream boundary proves it shared.
Do not add fallback parsing, XML rewriting, filename heuristics, broad serde
defaults, diagnostic I/O, or a public API. Keep the resource/main role seam
introduced by the resource build-item port.

The current `Vertex` DTO is already the shared boundary; do not split it by
role. A field-level default is justified by both cited upstream handlers. The
same source family also zero-defaults missing triangle `v1/v2/v3` at main
`_handle_start_triangle:3814` and ObjectImporter `_handle_object_start_triangle:
5527`. Triangle defaults are explicitly **deferred** from this slice: retain
current triangle index requirements and propose them separately with their own
Native-first witness. This deferral does not narrow the vertex behavior to one
role.

## Proposed input and gates

Use the exact current-main control-before50 pair from the resource
fixture family. The proposed positive may remove exactly
one vertex coordinate attribute from a resource model entry; because the decoder
is shared, this same positive exercises the production code path used by both
roles, while Native confirms the resource-role behavior. A future main-role
positive is not required to justify a separate DTO: source ownership is shared.
Preserve every other ZIP entry and byte. Freeze complete entry SHA manifest and
reproducer before any test commit.

Native-first official and seeded, fresh `--slice 0 --outputdir`, positive,
repeat, control; canonical environments. Require Native exit0,
complete footer, and full generator-only normalized neutrality before public
calls. Then validated `slice_project_plate_validated` plus real bare CLI on the
same inputs. Public RED, if observed, must assert the target Native-equivalent
success and whole strict output/raw API-CLI equality, therefore fail until GREEN;
never encode `is_err` or `assert_ne` as a passing RED. Controls assert equality
too. If Native does not complete or differs, stop with NO-RED; do not substitute
inputs or retry.

After RED, a production GREEN must keep every complete
Native/API/CLI stream equal, preserve M73/config/motion/arcs/statistics/footer,
and pass targeted existing 3MF/load regressions, fmt, strict clippy, and WASM.

## Included / deferred

Included: the named ObjectImporter missing-coordinate zero default for resource
vertices, with existing unit and validation behavior retained.

Deferred: missing triangle attributes, malformed mesh repair, non-finite values,
main-model schema relaxation, component/relationship policy, object metadata,
colors, transforms, seams, configuration, and M73. Any residual G-code delta
after GREEN is STOP/report, not downstream scope growth.
