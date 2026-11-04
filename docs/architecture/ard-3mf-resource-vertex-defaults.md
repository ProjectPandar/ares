# ARD: Preserve ObjectImporter zero defaults for resource vertices

Status: proposed. Docs only. Associated
[SPEC](3mf-resource-vertex-defaults-spec.md).

Orca `bbs_3mf.cpp::_BBS_3MF_Importer::_handle_start_vertex:3772-3782` and
`ObjectImporter::_handle_object_start_vertex:5485-5495` both send absent x/y/z
through `bbs_get_attribute_value_float`, whose documented caller behavior sets
missing values to zero. Main and resource dispatch therefore share this source
contract. Their model unit factors remain applied at their respective model
start handlers.

Rust target is the single private `project/model_xml.rs::Vertex` decoder used by
both root and resource decodes through `project/load/graph.rs`. Preserve mesh
finite/index checks, XML security checks and all explicit coordinate values. A
field-level zero default is source-backed for both roles; no role-specific DTO
partition is warranted. Triangle `v1/v2/v3` zero defaults at main `:3814` and
ObjectImporter `:5527` are explicitly deferred.

The proposed RED must be a target-equivalence assertion: Native complete output
versus successful Ares API/CLI with raw equality and the unchanged strict whole
generator-only comparator. A rejection is evidence of the defect and must make
the test fail, not pass. The control uses the same equality assertions.

Input, fixture, Native, and public execution follow the SPEC;
M73 and unrelated model behavior remain deferred.
