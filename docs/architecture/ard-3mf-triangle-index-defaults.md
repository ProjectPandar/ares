# ARD: shared triangle defaults precede unchanged mesh validation

Status: proposed. Docs only, associated
[SPEC](3mf-triangle-index-defaults-spec.md).

## Source owner and destination

Orca8500fcdcc `Format/bbs_3mf.cpp::bbs_get_attribute_value_int:508-514` returns
zero for an absent attribute. Main `_handle_start_triangle:3813-3819` and
ObjectImporter `_handle_object_start_triangle:5526-5532` use it for v1/v2/v3.
Rust target is the single `project/model_xml.rs::Triangle` decoder consumed by
both graph roles. Field-level defaults only; no root/resource split or broad
serde policy. Existing DTOs remain transport shells for the named importers.

## Non-negotiable scope

- Native index zero is a vertex-table index, not a repair command. Active
  `_generate_volumes_new:4975-5007` validates bounds and constructs TriangleMesh;
  its constructor (`TriangleMesh.cpp:71-75`) initializes statistics, not the STL
  repair routine. Do not use the commented old `_generate_volumes` as evidence.
- Repeated/zero-area faces are not automatically proven repaired. The initial
  witness removes an already-zero v1 from `(0,1,2)`, keeping geometry identical.
  Broader repair, malformed integer conversion and degenerating omissions are
  deferred; missing v2/v3 use the same source-backed shared decode scope.
- Preserve explicit values, finite/index checks, XML security, annotation order,
  all other fields, main/root build semantics and byte-only Tier1 boundaries.
  No guard/fallback/reparse/I/O/new API or independent Ares pipeline.
- Native-first, same-input public Native/API/realCLI target equality is the RED
  seam. Native positive/control neutrality is expected, not NO-RED; Native vs
  Ares equality is NO-RED. Both public calls precede failing target assertions.
- Controls assert whole equality too. No inverted passing assertions, missing-
  output equality, comparator weakening or M73 filtering. Separate discovery,
  frozen public RED and separate production commit; any residual difference STOP.

## Alternatives rejected / deferred

A general mesh-repair port would obscure the scalar-default owner. A role DTO
split lacks upstream justification. Reindexing an input to synthesize v2/v3
zero cases changes more than the selected first pair. Those are not part of this
milestone. M73 discovery and all unrelated loader/config/geometry work stay deferred.
