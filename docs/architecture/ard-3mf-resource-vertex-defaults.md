# ARD: Preserve ObjectImporter zero defaults for resource vertices

Status: **PROPOSED**, docs only. Associated
[SPEC](3mf-resource-vertex-defaults-spec.md). No execution or production
authorization.

Orca `bbs_3mf.cpp::ObjectImporter::_handle_object_start_vertex:5478-5490`
explicitly sends absent x/y/z through `bbs_get_attribute_value_float`, whose
documented caller behavior sets missing values to zero. Resource dispatch is
owned by `_handle_object_start_model_xml_element:5612-5649`; model units are
set at `_handle_object_start_model:5315-5319`.

Rust target is private `project/model_xml.rs::Vertex` within the role-aware
resource decode reached by `project/load/graph.rs`. Preserve the required root
model decoder, mesh finite/index checks, XML security checks and all explicit
coordinate values. Do not apply a global serde default without proving that
main-model behavior has the same upstream owner.

The proposed RED must be a target-equivalence assertion: Native complete output
versus successful Ares API/CLI with raw equality and the unchanged strict whole
generator-only comparator. A rejection is evidence of the defect and must make
the test fail, not pass. The control uses the same equality assertions.

Input, fixture, Native, and public execution remain deferred until SPEC approval;
M73 and unrelated model behavior remain deferred.
