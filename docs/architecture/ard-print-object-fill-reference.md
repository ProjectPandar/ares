# ARD: Use the upstream centered PrintObject fill reference

Status: accepted; bounded 52-case acceptance verified
on main `56e4da53`. See plan for retained failures
and lint warnings.

The non-negotiable boundary is `PrintObject::bounding_box` (OrcaSlicer2.4.2
`Print.hpp:347`) → `Layer::make_fills` (`Fill.cpp:1224`) →
`Fill::_infill_direction` (`FillBase.cpp:286-290`). The symmetric integer
object box has center0,0; quantized/compensated slice vertices do not redefine
that anchor. A one-unit discrepancy changes geometric tie outcomes and therefore
complete G-code, even if displayed extrusion coordinates initially match.

In Rust, change only reference selection inside
`project_slice/fill_entities/generate.rs::generate_layer`. The existing pattern
ports remain upstream compatibility shells; no new pipeline, configuration,
API, dependency or platform-specific runtime is added. This constant reference
eliminates the unnecessary all-layer bounds scan at this call site.

Do not repair this symptom in cooling/M73, relax ordered comparisons, add printer
special cases, or modify generic coordinate arithmetic. Do not change the
bridge-owner's retained `object_center` call or extended pattern-box sizing.
These adjacent scaffold approximations remain explicitly deferred, not asserted
to be upstream-equivalent.

Verification is the existing public project-bytes/options → G-code-bytes seam
against original Orca2.4.2, preserving generator-only normalization and all RED
assertions. No isolated unit tests or production instrumentation are introduced.
See the corresponding spec and vendor-stock triage plan for exact artifacts.
